/*!
Route handlers
*/
use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use futures::{self, Future};
use hex::FromHex;
use hyper;
use tokio;
use uuid::Uuid;
use warp;

use auth;
use error;
use models::{self, CONFIG};
use service::Ctx;

/// Return the default configurable upload constraints
pub fn api_upload_defaults() -> impl warp::Reply {
    let defaults = json!({
        "upload_limit_bytes": CONFIG.upload_limit_bytes,
        "upload_lifespan_secs_default": CONFIG.upload_lifespan_secs_default,
        "download_limit_default": CONFIG.download_limit_default,
    });
    warp::reply::json(&defaults)
}

/// Upload Initialize post info (in transport formatting)
#[derive(Deserialize)]
pub struct UploadInitPost {
    nonce: String,
    file_name_hash: String,
    size: u64,
    content_hash: String,
    access_password: String,
    deletion_password: Option<String>,
    download_limit: Option<u32>,
    lifespan: Option<i64>,
}
impl UploadInitPost {
    fn decode_hex(&self) -> error::Result<UploadInit> {
        let lifespan = Duration::seconds(
            self.lifespan
                .unwrap_or(models::CONFIG.upload_lifespan_secs_default),
        );
        let expire_date = Utc::now()
            .checked_add_signed(lifespan)
            .ok_or_else(|| "lifespan too large")?;
        let deletion_password = match self.deletion_password {
            Some(ref hex) => Some(Vec::from_hex(hex)?),
            None => None,
        };
        Ok(UploadInit {
            nonce: Vec::from_hex(&self.nonce)?,
            file_name_hash: Vec::from_hex(&self.file_name_hash)?,
            size: self.size as i64,
            content_hash: Vec::from_hex(&self.content_hash)?,
            access_password: Vec::from_hex(&self.access_password)?,
            deletion_password: deletion_password,
            download_limit: self
                .download_limit
                .map(|n| n as i32)
                .or(CONFIG.download_limit_default),
            expire_date: expire_date,
        })
    }
}

/// Upload post info converted/decoded
#[derive(Debug)]
struct UploadInit {
    nonce: Vec<u8>,
    file_name_hash: Vec<u8>,
    size: i64,
    content_hash: Vec<u8>,
    access_password: Vec<u8>,
    deletion_password: Option<Vec<u8>>,
    download_limit: Option<i32>,
    expire_date: DateTime<Utc>,
}

/// Initialize a new upload
///
/// Supply all meta-data about an upload. Returning a unique key
/// e.g.)
///   format!("{}/api/upload?key={}", "http://localhost:3000", "...long-key...")
///
pub fn api_upload_init(
    ctx: Ctx,
    info: UploadInitPost,
) -> impl Future<Item = impl warp::Reply, Error = warp::Rejection> {
    let info =
        futures::future::result::<UploadInit, error::Error>(|| -> error::Result<UploadInit> {
            let info = info
                .decode_hex()
                .map_err(|_| error::helpers::bad_request("malformed info"))?;
            if info.size > models::CONFIG.upload_limit_bytes {
                return Err(error::helpers::too_large(format!(
                    "Upload too large, max bytes: {}",
                    models::CONFIG.upload_limit_bytes
                )));
            }
            Ok(info)
        }());

    let db = ctx.db;
    let cpu = ctx.cpu;
    info.and_then(move |info: UploadInit| {
        cpu.spawn_fn(move || -> error::Result<String> {
            let uuid = Uuid::new_v4();
            let uuid_hex = hex::encode(uuid.as_bytes());

            let conn = db.get()?;
            let trans = conn.transaction()?;
            trans.set_commit();

            if !models::Status::can_fit(&trans, info.size)? {
                return Err(error::helpers::out_of_space("Server out of storage space"));
            }
            let access_auth =
                models::NewAuth::from_pass_bytes(&info.access_password)?.insert(&trans)?;
            let deletion_auth = match info.deletion_password {
                Some(ref bytes) => {
                    let auth = models::NewAuth::from_pass_bytes(bytes)?.insert(&trans)?;
                    Some(auth.id)
                }
                None => None,
            };
            let new_init_upload = models::NewInitUpload {
                uuid: uuid,
                file_name_hash: info.file_name_hash,
                content_hash: info.content_hash,
                size: info.size,
                nonce: info.nonce,
                access_password: access_auth.id,
                deletion_password: deletion_auth,
                download_limit: info.download_limit,
                expire_date: info.expire_date,
            };
            new_init_upload.insert(&trans)?;
            Ok(uuid_hex)
        })
    })
    .map(move |uuid_hex| {
        let resp = json!({ "key": &uuid_hex });
        warp::reply::json(&resp)
    })
    .map_err(|e| error::helpers::reject(e))
}

/// Upload identifier
#[derive(Deserialize, Debug)]
pub struct UploadKey {
    key: String,
}

/// Upload encrypted bytes to a specified upload-key
///
/// Before accepting upload:
///     - Make sure the server has enough space available (using the previously reported file size).
///     - Make sure the upload came within the upload time-out
///     - While reading the uploaded bytes, keep count and make sure the number of bytes <= state size
pub fn api_upload_file(
    ctx: Ctx,
    upload_key: UploadKey,
    body: warp::body::BodyStream,
) -> impl Future<Item = impl warp::Reply, Error = warp::Rejection> {
    info!("upload started, key: {}", upload_key.key);
    struct Info {
        uuid: Uuid,
        now: DateTime<Utc>,
        upload: Option<models::Upload>,
    }

    let info = futures::future::result::<Info, error::Error>(move || -> error::Result<Info> {
        let now = Utc::now();
        let uuid = Uuid::from_str(&upload_key.key)
            .map_err(|_| error::helpers::does_not_exist("upload not found"))?;
        Ok(Info {
            uuid: uuid,
            now: now,
            upload: None,
        })
    }());

    let db_create = ctx.db.clone();
    let cpu_create = ctx.cpu.clone();
    let db_delete = ctx.db.clone();
    let cpu_delete = ctx.cpu.clone();

    info.and_then(move |mut info: Info| {
        cpu_create.spawn_fn(move || -> error::Result<Info> {
            let conn = db_create.get()?;
            let trans = conn.transaction()?;
            trans.set_commit();

            let init_upload = models::InitUpload::find(&trans, &info.uuid)?;
            if !models::Status::can_fit(&trans, init_upload.size)? {
                return Err(error::helpers::out_of_space("Server out of storage space"));
            }

            models::Status::inc_upload(&trans, init_upload.size)?;
            let file_path = models::Upload::new_file_path(&init_upload.uuid)?;
            init_upload.delete(&trans)?;
            if !init_upload.still_valid(&info.now) {
                return Err(error::helpers::bad_request("Upload request came too late"));
            }
            let new_upload = init_upload.into_upload(&file_path)?;
            let upload = new_upload.insert(&trans)?;
            info.upload = Some(upload);
            Ok(info)
        })
    })
    // Convert error to match the next future's unnameable return type.
    // The tuple Error is required so the error branch can cleanup oversized uploads
    .map_err(|e| (e, None))
    .and_then(move |info: Info| {
        // -> `impl Future<Item=usize, Error=(error::Error, Option<models::Upload>)>`
        use warp::Buf;
        use warp::Stream;

        let max_bytes = models::CONFIG.upload_limit_bytes as usize;
        let upload = info.upload.expect("No upload present");

        tokio::fs::File::create(upload.file_path.clone())
            .map_err(|e| (error::Error::from(e), None))
            .and_then(move |file| {
                body.map_err(|e| (error::Error::from(e), None))
                    .fold((file, 0), move |(file, byte_count), buf| {
                        futures::future::result::<Vec<u8>, (error::Error, Option<models::Upload>)>(
                            {
                                let bytes = buf.bytes();
                                let size = byte_count + bytes.len();
                                if size > max_bytes {
                                    Err((
                                        error::helpers::too_large("upload too large"),
                                        Some(upload.clone()),
                                    ))
                                } else {
                                    Ok(bytes.to_vec())
                                }
                            },
                        )
                        .and_then(move |bytes| {
                            tokio::io::write_all(file, bytes)
                                .map(move |(file, bytes_written)| {
                                    (file, byte_count + bytes_written.len())
                                })
                                .map_err(|e| (error::Error::from(e), None))
                        })
                    })
                    .map(|(_file, size)| size)
            })
    })
    .map(move |size| {
        let resp = json!({"ok": "ok", "bytes": size});
        warp::reply::json(&resp)
    })
    .or_else(move |(err, maybe_upload)| {
        // mark the upload deleted and pass along the upload-file_path to delete
        cpu_delete
            .spawn_fn(move || -> error::Result<String> {
                match err.kind() {
                    error::ErrorKind::UploadTooLarge(_) => match maybe_upload {
                        Some(upload) => {
                            let conn = db_delete.get()?;
                            upload.delete(&*conn)?;
                            return Ok(upload.file_path);
                        }
                        _ => unreachable!("Found UploadTooLarge error, but no Upload"),
                    },
                    _ => (),
                }
                Err(err)
            })
            .and_then(|path| {
                // delete the file we just created and then convert this to an error-future
                tokio::fs::remove_file(path)
                    .map_err(|e| error::Error::from(e))
                    .and_then(|_| {
                        futures::future::err(error::helpers::too_large("upload too large"))
                    })
            })
            // this future chain should only consist of errors at this point
            .map(|()| unreachable!("Future chain should contain only errors"))
            .map_err(|e| error::helpers::reject(e))
    })
}

#[derive(Deserialize)]
pub struct DeleteKeyAccessPost {
    key: String,
    deletion_password: String,
}
impl DeleteKeyAccessPost {
    fn decode_hex(&self) -> error::Result<DeleteKeyAccess> {
        Ok(DeleteKeyAccess {
            uuid: Uuid::from_str(&self.key)
                .map_err(|_| error::helpers::does_not_exist("upload not found"))?,
            deletion_password: Vec::from_hex(&self.deletion_password)
                .map_err(|_| error::helpers::bad_request("malformed info"))?,
        })
    }
}

struct DeleteKeyAccess {
    uuid: Uuid,
    deletion_password: Vec<u8>,
}

/// Deletes an upload by key. Only uploads that were created with a deletion password can be deleted.
/// Deletion password must be present.
pub fn api_upload_delete(
    ctx: Ctx,
    delete_key: DeleteKeyAccessPost,
) -> impl Future<Item = impl warp::Reply, Error = warp::Rejection> {
    let cpu = ctx.cpu;
    let db = ctx.db;
    futures::future::result::<DeleteKeyAccess, error::Error>(delete_key.decode_hex())
        .and_then(move |delete_key| {
            cpu.spawn_fn(move || -> error::Result<String> {
                let conn = db.get()?;
                let trans = conn.transaction()?;
                trans.set_commit();

                let upload = models::Upload::find(&trans, &delete_key.uuid)?;
                let deletion_auth = upload.get_deletion_auth(&trans)?;
                match deletion_auth {
                    None => Err(error::helpers::bad_request("cannot delete")),
                    Some(auth) => {
                        auth.verify(&delete_key.deletion_password)?;
                        let id = upload.id;
                        match upload.delete(&trans) {
                            Ok(_) => {
                                models::Status::dec_upload(&trans, upload.size)?;
                            }
                            Err(e) => {
                                error!("Error deleting upload with id={}, {}", id, e);
                                return Err(error::Error::from(e));
                            }
                        }
                        Ok(upload.file_path)
                    }
                }
            })
        })
        .and_then(|file_path| {
            tokio::fs::remove_file(file_path.clone()).or_else(move |e| {
                error!(
                    "Error deleting upload file {}, {} continuing...",
                    file_path, e
                );
                futures::future::ok(())
            })
        })
        .map(|_| {
            let resp = json!({"ok": "ok"});
            warp::reply::json(&resp)
        })
        .map_err(|e| error::helpers::reject(e))
}

/// Download identifier and access/auth password
#[derive(Deserialize)]
pub struct DownloadKeyAccessPost {
    key: String,
    access_password: String,
}
impl DownloadKeyAccessPost {
    fn decode_hex(&self) -> error::Result<DownloadKeyAccess> {
        Ok(DownloadKeyAccess {
            uuid: Uuid::from_str(&self.key)
                .map_err(|_| error::helpers::does_not_exist("upload not found"))?,
            access_password: Vec::from_hex(&self.access_password)
                .map_err(|_| error::helpers::bad_request("malformed info"))?,
        })
    }
}

struct DownloadKeyAccess {
    uuid: Uuid,
    access_password: Vec<u8>,
}

/// Initialize a download
///
/// Using a key and access-password, obtain the download meta-data (stuff
/// needed for decryption).
pub fn api_download_init(
    ctx: Ctx,
    download_key: DownloadKeyAccessPost,
) -> impl Future<Item = impl warp::Reply, Error = warp::Rejection> {
    struct Info {
        now: DateTime<Utc>,
        download_key: DownloadKeyAccess,
    }

    struct Data {
        upload: models::Upload,
        init_download_content: models::InitDownload,
        init_download_confirm: models::InitDownload,
    }

    let cpu = ctx.cpu;
    let db = ctx.db;
    futures::future::result::<Info, error::Error>((|| {
        let now = Utc::now();
        let download_key = download_key.decode_hex()?;
        Ok(Info { now, download_key })
    })())
    .and_then(move |info: Info| {
        cpu.spawn_fn(move || -> error::Result<Data> {
            let conn = db.get()?;
            let trans = conn.transaction()?;
            trans.set_commit();

            let upload = models::Upload::find(&trans, &info.download_key.uuid)?;
            let access_auth = upload.get_access_auth(&trans)?;
            access_auth.verify(&info.download_key.access_password)?;
            let n_downloads = upload.download_count(&trans)? as i32;
            if let Some(limit) = upload.download_limit {
                if n_downloads >= limit {
                    return Err(error::helpers::does_not_exist("upload not found"));
                }
            }
            if info.now >= upload.expire_date {
                return Err(error::helpers::does_not_exist("upload not found"));
            }
            let init_download_content = models::NewInitDownload {
                uuid: Uuid::new_v4(),
                usage: String::from("content"),
                upload: upload.id,
            }
            .insert(&trans)?;
            let init_download_confirm = models::NewInitDownload {
                uuid: Uuid::new_v4(),
                usage: String::from("confirm"),
                upload: upload.id,
            }
            .insert(&trans)?;
            Ok(Data {
                upload: upload,
                init_download_content,
                init_download_confirm,
            })
        })
    })
    .map(|data| {
        let resp = json!({
            "nonce": hex::encode(&data.upload.nonce),
            "size": data.upload.size,
            "download_key": hex::encode(data.init_download_content.uuid.as_bytes()),
            "confirm_key": hex::encode(data.init_download_confirm.uuid.as_bytes()),
        });
        warp::reply::json(&resp)
    })
    .map_err(|e| error::helpers::reject(e))
}

/// Download encrypted bytes
pub fn api_download(
    ctx: Ctx,
    download_key: DownloadKeyAccessPost,
) -> impl Future<Item = impl warp::Reply, Error = warp::Rejection> {
    info!("upload started, key: {}", download_key.key);
    struct Info {
        now: DateTime<Utc>,
        download_key: DownloadKeyAccess,
    }

    let cpu = ctx.cpu;
    let db = ctx.db;
    let fs_pool = ctx.fs;
    futures::future::result::<Info, error::Error>((|| {
        let now = Utc::now();
        let download_key = download_key.decode_hex()?;
        Ok(Info { now, download_key })
    })())
    .and_then(move |info: Info| {
        cpu.spawn_fn(move || -> error::Result<models::Upload> {
            let conn = db.get()?;
            let trans = conn.transaction()?;
            trans.set_commit();

            let init_download = models::InitDownload::find(
                &trans,
                &info.download_key.uuid,
                models::DownloadType::Content,
            )?;
            let upload = init_download.get_upload(&trans)?;
            let access_auth = upload.get_access_auth(&trans)?;
            access_auth.verify(&info.download_key.access_password)?;
            let n_downloads = upload.download_count(&trans)? as i32;
            if let Some(limit) = upload.download_limit {
                if n_downloads >= limit {
                    return Err(error::helpers::does_not_exist("upload not found"));
                }
            }
            if info.now >= upload.expire_date {
                return Err(error::helpers::does_not_exist("upload not found"));
            }
            let new_download = models::NewDownload { upload: upload.id };
            new_download.insert(&trans)?;
            init_download.delete(&trans)?;
            Ok(upload)
        })
    })
    .map(move |upload| {
        let stream = fs_pool.read(upload.file_path, Default::default());
        let body = hyper::Body::wrap_stream(stream);
        warp::http::Response::builder().body(body)
    })
    .map_err(|e| error::helpers::reject(e))
    // if request.header("x-proxy-nginx").unwrap_or("") == "true" {
    //     let upload_path = format!("/private/{}", hex::encode(upload.uuid.as_bytes()));
    //     let resp = Response::empty_400()
    //         .with_status_code(200)
    //         .with_additional_header("x-accel-redirect", upload_path)
    //         .with_additional_header("content-type", "application/octet-stream");
    //     Ok(resp)
    // }
}

/// Download identifier and corresponding decrypted content hash
#[derive(Deserialize)]
pub struct DownloadKeyHash {
    key: String,
    hash: String,
}

/// Obtain the decrypted file's name
///
/// Upload identifier and a matching hash of the decrypted content are required
pub fn api_download_confirm(
    ctx: Ctx,
    download_key: DownloadKeyHash,
) -> impl Future<Item = impl warp::Reply, Error = warp::Rejection> {
    struct Info {
        hash_bytes: Vec<u8>,
        uuid: Uuid,
    }

    let cpu = ctx.cpu;
    let db = ctx.db;
    futures::future::result::<Info, error::Error>((|| {
        let hash_bytes = Vec::from_hex(&download_key.hash)
            .map_err(|_| error::helpers::bad_request("malformed info"))?;
        let uuid_bytes = Vec::from_hex(&download_key.key)?;
        let uuid = Uuid::from_bytes(&uuid_bytes)
            .map_err(|_| error::helpers::does_not_exist("upload not found"))?;
        Ok(Info { hash_bytes, uuid })
    })())
    .and_then(move |info| {
        cpu.spawn_fn(move || -> error::Result<models::Upload> {
            let conn = db.get()?;
            let trans = conn.transaction()?;
            trans.set_commit();

            let init_download =
                models::InitDownload::find(&*conn, &info.uuid, models::DownloadType::Confirm)?;
            let upload = init_download.get_upload(&trans)?;
            auth::eq(&info.hash_bytes, &upload.content_hash)?;
            init_download.delete(&trans)?;
            Ok(upload)
        })
    })
    .map(|upload| {
        let resp = json!({"file_name_hash": hex::encode(&upload.file_name_hash)});
        warp::reply::json(&resp)
    })
    .map_err(|e| error::helpers::reject(e))
}
