/*!
Route handlers
*/
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::path;
use std::fs;

use rouille::{Request, Response};
use hex::{FromHex, ToHex};
use uuid::Uuid;
use chrono::{Utc, Duration, DateTime};

use db;
use auth;
use models::{self, CONFIG};
use {FromRequestBody, FromRequestQuery, ToResponse};
use errors::*;


/// Return a `rouille::Response` for a given file
pub fn serve_file<T: AsRef<path::Path>>(mime: &'static str, path: T) -> Result<Response> {
    let file = fs::File::open(path.as_ref())?;
    Ok(Response::from_file(mime, file))
}


/// Return the default configurable upload constraints
pub fn api_upload_defaults(_: &Request) -> Result<Response> {
    let defaults = json!({
        "upload_limit_bytes": CONFIG.upload_limit_bytes,
        "upload_lifespan_secs_default": CONFIG.upload_lifespan_secs_default,
        "download_limit_default": CONFIG.download_limit_default,
    }).to_resp()?;
    Ok(defaults)
}


/// Upload Initialize post info (in transport formatting)
#[derive(Deserialize)]
struct UploadInitPost {
    nonce: String,
    file_name: String,
    size: u64,
    content_hash: String,
    access_password: String,
    deletion_password: Option<String>,
    download_limit: Option<u32>,
    lifespan: Option<i64>,
}
impl UploadInitPost {
    fn decode_hex(&self) -> Result<UploadInit> {
        let lifespan = Duration::seconds(self.lifespan.unwrap_or(models::CONFIG.upload_lifespan_secs_default));
        let expire_date = Utc::now().checked_add_signed(lifespan)
            .ok_or_else(|| format_err!(ErrorKind::BadRequest, "Lifespan (seconds) too large"))?;
        let deletion_password = match self.deletion_password {
            Some(ref hex) => Some(Vec::from_hex(hex)?),
            None => None,
        };
        Ok(UploadInit {
            nonce: Vec::from_hex(&self.nonce)?,
            file_name: self.file_name.to_owned(),
            size: self.size as i64,
            content_hash: Vec::from_hex(&self.content_hash)?,
            access_password: Vec::from_hex(&self.access_password)?,
            deletion_password: deletion_password,
            download_limit: self.download_limit.map(|n| n as i32).or(CONFIG.download_limit_default),
            expire_date: expire_date,
        })
    }
}

/// Upload post info converted/decoded
#[derive(Debug)]
struct UploadInit {
    nonce: Vec<u8>,
    file_name: String,
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
pub fn api_upload_init(request: &Request, conn: db::DbConn) -> Result<Response> {
    let info = request.parse_json_body::<UploadInitPost>()?;
    let info = info.decode_hex()
        .map_err(|_| format_err!(ErrorKind::BadRequest, "malformed info"))?;
    if info.size > models::CONFIG.upload_limit_bytes {
        bail_fmt!(ErrorKind::UploadTooLarge, "Upload too large, max bytes: {}", models::CONFIG.upload_limit_bytes)
    }
    let uuid = Uuid::new_v4();
    let uuid_hex = uuid.as_bytes().to_hex();

    {
        let trans = conn.transaction()?;
        if ! models::Status::can_fit(&trans, info.size)? {
            bail_fmt!(ErrorKind::OutOfSpace, "Server out of storage space");
        }
        let access_auth = models::NewAuth::from_pass_bytes(&info.access_password)?.insert(&trans)?;
        let deletion_auth = match info.deletion_password {
            Some(ref bytes) => {
                let auth = models::NewAuth::from_pass_bytes(bytes)?.insert(&trans)?;
                Some(auth.id)
            }
            None => None,
        };
        let new_init_upload = models::NewInitUpload {
            uuid: uuid,
            file_name: info.file_name,
            content_hash: info.content_hash,
            size: info.size,
            nonce: info.nonce,
            access_password: access_auth.id,
            deletion_password: deletion_auth,
            download_limit: info.download_limit,
            expire_date: info.expire_date,
        };
        new_init_upload.insert(&trans)?;
        trans.commit()?;
    }

    let resp = json!({"key": &uuid_hex}).to_resp()?;
    Ok(resp)
}


/// Upload identifier
#[derive(Deserialize, Debug)]
struct UploadKey{
    key: String,
}


/// Upload encrypted bytes to a specified upload-key
///
/// Before accepting upload:
///     - Make sure the server has enough space available (using the previously reported file size).
///     - Make sure the upload came within the upload time-out
///     - While reading the uploaded bytes, keep count and make sure the number of bytes <= state size
pub fn api_upload_file(request: &Request, conn: db::DbConn) -> Result<Response> {
    let upload_key = request.parse_query_params::<UploadKey>()?;
    let upload = {
        let now = Utc::now();
        let uuid = Uuid::from_str(&upload_key.key)
            .map_err(|_| format_err!(ErrorKind::DoesNotExist, "upload not found"))?;

        let trans = conn.transaction()?;
        let init_upload = models::InitUpload::find(&trans, &uuid)?;
        if ! models::Status::can_fit(&trans, init_upload.size)? {
            bail_fmt!(ErrorKind::OutOfSpace, "Server out of storage space");
        }
        models::Status::inc_upload(&trans, init_upload.size)?;
        let file_path = models::Upload::new_file_path(&init_upload.uuid)?;
        init_upload.delete(&trans)?;
        if ! init_upload.still_valid(&now) {
            bail_fmt!(ErrorKind::BadRequest, "Upload request came to late");
        }
        let new_upload = init_upload.into_upload(&file_path)?;
        let upload = new_upload.insert(&trans)?;
        trans.commit()?;
        upload
    };

    // In case they lied about the upload size...
    // Note: when living behind a proxy, nginx will deal with rejecting
    //       requests larger than `client_max_body_size`
    let mut byte_count = 0;
    let mut file = fs::File::create(&upload.file_path)?;
    let mut stream = io::BufReader::new(request.data().expect("body already read"));
    let stated_upload_size = upload.size as usize;  // db stores i64
    loop {
        let n = {
            let buf = stream.fill_buf()?;
            file.write_all(&buf)?;
            buf.len()
        };
        stream.consume(n);
        if n == 0 { break; }

        byte_count += n;
        if byte_count > stated_upload_size {
            error!("Upload larger than previously stated");
            // delete the entry we just made
            upload.delete(&*conn)?;
            // if the file deletion fails, the file will eventually be cleaned up
            // by the `admin sweep-files` command
            fs::remove_file(&upload.file_path).ok();
            // See if we can drain the rest of the stream and send a real response
            // before we kill the connection
            let mut over = 0;
            loop {
                let n = {
                    let buf = stream.fill_buf()?;
                    buf.len()
                };
                stream.consume(n);
                over += n;
                if n == 0 || over >= 10_000 { break; }
            }
            bail_fmt!(ErrorKind::UploadTooLarge, "Upload larger than previously stated: {}", stated_upload_size)
        }
    }

    let resp = json!({"ok": "ok"}).to_resp()?;
    Ok(resp)
}


#[derive(Deserialize)]
struct DeleteKeyAccessPost {
    key: String,
    deletion_password: String,
}
impl DeleteKeyAccessPost {
    fn decode_hex(&self) -> Result<DeleteKeyAccess> {
        Ok(DeleteKeyAccess {
            uuid: Uuid::from_str(&self.key).map_err(|_| format_err!(ErrorKind::DoesNotExist, "upload not found"))?,
            deletion_password: Vec::from_hex(&self.deletion_password).map_err(|_| format_err!(ErrorKind::BadRequest, "malformed info"))?,
        })
    }
}

struct DeleteKeyAccess {
    uuid: Uuid,
    deletion_password: Vec<u8>,
}


/// Deletes an upload by key. Only uploads that were created with a deletion password can be deleted.
/// Deletion password must be present.
pub fn api_upload_delete(request: &Request, conn: db::DbConn) -> Result<Response> {
    let delete_key = request.parse_json_body::<DeleteKeyAccessPost>()?;
    let delete_key = delete_key.decode_hex()?;
    {
        let trans = conn.transaction()?;
        let upload = models::Upload::find(&trans, &delete_key.uuid)?;
        let deletion_auth = upload.get_deletion_auth(&trans)?;
        match deletion_auth {
            None => bail_fmt!(ErrorKind::BadRequest, "cannot delete"),
            Some(auth) => {
                auth.verify(&delete_key.deletion_password)?;
                match fs::remove_file(&upload.file_path) {
                    Ok(_) => (),
                    Err(e) => error!("Error deleting {}, {}, continuing...", upload.file_path, e),
                }
                let id = upload.id;
                match upload.delete(&trans) {
                    Ok(_) => {
                        models::Status::dec_upload(&trans, upload.size)?;
                    }
                    Err(e) => error!("Error deleting upload with id={}, {}, continuing...", id, e),
                }
            }
        }
        trans.commit()?;
    }
    Ok(json!({"ok": "ok"}).to_resp()?)
}


/// Download identifier and access/auth password
#[derive(Deserialize)]
struct DownloadKeyAccessPost {
    key: String,
    access_password: String,
}
impl DownloadKeyAccessPost {
    fn decode_hex(&self) -> Result<DownloadKeyAccess> {
        Ok(DownloadKeyAccess{
            uuid: Uuid::from_str(&self.key).map_err(|_| format_err!(ErrorKind::DoesNotExist, "upload not found"))?,
            access_password: Vec::from_hex(&self.access_password).map_err(|_| format_err!(ErrorKind::BadRequest, "malformed info"))?,
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
pub fn api_download_init(request: &Request, conn: db::DbConn) -> Result<Response> {
    let now = Utc::now();
    let download_key = request.parse_json_body::<DownloadKeyAccessPost>()?;
    let download_key = download_key.decode_hex()?;

    let (upload, init_download_content, init_download_confirm) = {
        let trans = conn.transaction()?;
        let upload = models::Upload::find(&trans, &download_key.uuid)?;
        let access_auth = upload.get_access_auth(&trans)?;
        access_auth.verify(&download_key.access_password)?;
        let n_downloads = upload.download_count(&trans)? as i32;
        if let Some(limit) = upload.download_limit {
            if n_downloads >= limit {
                bail_fmt!(ErrorKind::DoesNotExist, "upload not found");
            }
        }
        if now >= upload.expire_date {
            bail_fmt!(ErrorKind::DoesNotExist, "upload not found");
        }
        let init_download_content = models::NewInitDownload {
            uuid: Uuid::new_v4(),
            usage: String::from("content"),
            upload: upload.id,
        }.insert(&trans)?;
        let init_download_confirm = models::NewInitDownload {
            uuid: Uuid::new_v4(),
            usage: String::from("confirm"),
            upload: upload.id,
        }.insert(&trans)?;
        trans.commit()?;
        (upload, init_download_content, init_download_confirm)
    };
    let resp = json!({
        "nonce": upload.nonce.to_hex(),
        "size": upload.size,
        "download_key": init_download_content.uuid.as_bytes().to_hex(),
        "confirm_key": init_download_confirm.uuid.as_bytes().to_hex(),
    }).to_resp()?;
    Ok(resp)
}


/// Download encrypted bytes
pub fn api_download(request: &Request, conn: db::DbConn) -> Result<Response> {
    let now = Utc::now();
    let download_key = request.parse_json_body::<DownloadKeyAccessPost>()?;
    let download_key = download_key.decode_hex()?;
    let upload = {
        let trans = conn.transaction()?;
        let init_download = models::InitDownload::find(&trans, &download_key.uuid, models::DownloadType::Content)?;
        let upload = init_download.get_upload(&trans)?;
        let access_auth = upload.get_access_auth(&trans)?;
        access_auth.verify(&download_key.access_password)?;
        let n_downloads = upload.download_count(&trans)? as i32;
        if let Some(limit) = upload.download_limit {
            if n_downloads >= limit {
                bail_fmt!(ErrorKind::DoesNotExist, "upload not found");
            }
        }
        if now >= upload.expire_date {
            bail_fmt!(ErrorKind::DoesNotExist, "upload not found");
        }
        let new_download = models::NewDownload { upload: upload.id };
        new_download.insert(&trans)?;
        init_download.delete(&trans)?;
        trans.commit()?;
        upload
    };
    serve_file("application/octet-stream", upload.file_path)
}


/// Download identifier and corresponding decrypted content hash
#[derive(Deserialize)]
struct DownloadKeyHash {
    key: String,
    hash: String,
}


/// Obtain the decrypted file's name
///
/// Upload identifier and a matching hash of the decrypted content are required
pub fn api_download_confirm(request: &Request, conn: db::DbConn) -> Result<Response> {
    let download_key = request.parse_json_body::<DownloadKeyHash>()?;
    let hash_bytes = Vec::from_hex(&download_key.hash)
        .map_err(|_| format_err!(ErrorKind::BadRequest, "malformed info"))?;
    let uuid_bytes = Vec::from_hex(&download_key.key)?;
    let uuid = Uuid::from_bytes(&uuid_bytes)
        .map_err(|_| format_err!(ErrorKind::DoesNotExist, "upload not found"))?;
    let upload = {
        let trans = conn.transaction()?;
        let init_download = models::InitDownload::find(&*conn, &uuid, models::DownloadType::Confirm)?;
        let upload = init_download.get_upload(&trans)?;
        auth::eq(&hash_bytes, &upload.content_hash)?;
        init_download.delete(&trans)?;
        trans.commit()?;
        upload
    };
    Ok(json!({"file_name": &upload.file_name}).to_resp()?)
}

