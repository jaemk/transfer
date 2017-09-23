/*!
Route handlers
*/
use std::path;
use std::io;
use std::fs;

use rocket;
use rocket::response;
use rocket::http;
use rocket_contrib::{Json, Value as JsonValue};
use hex::{FromHex, ToHex};
use uuid::Uuid;
use chrono::{Utc, Duration};

use db;
use auth;
use models;
use errors::*;


/// Static file handler
#[get("/<file..>")]
fn static_files(file: path::PathBuf) -> Option<response::NamedFile> {
    response::NamedFile::open(path::Path::new("static/").join(file)).ok()
}


/// Index page with static files test
#[get("/")]
fn index<'a>() -> response::Response<'a> {
    let resp = response::Response::build()
        .header(http::ContentType::HTML)
        .sized_body(io::Cursor::new("<html><body> <p> hello </p> <script src=\"/static/js/app.js\"></script></body></html>"))
        .finalize();
    resp
}


#[get("/api/hello")]
fn api_hello() -> Json<JsonValue> {
    Json(json!({"message": "hello!"}))
}

#[post("/api/bye")]
fn api_bye<'a>() -> Json<JsonValue> {
    Json(json!({"message": "bye!"}))
}


/// Upload Initialize post info (in transport formatting)
#[derive(Deserialize)]
struct UploadInitPost {
    nonce: String,
    file_name: String,
    size: u64,
    content_hash: String,
    access_password: String,
}
impl UploadInitPost {
    fn decode_hex(&self) -> Result<UploadInit> {
        Ok(UploadInit {
            nonce: Vec::from_hex(&self.nonce)?,
            file_name: self.file_name.to_owned(),
            size: self.size as i64,
            content_hash: Vec::from_hex(&self.content_hash)?,
            access_password: Vec::from_hex(&self.access_password)?,
        })
    }
}

/// Upload post info converted/decoded
struct UploadInit {
    nonce: Vec<u8>,
    file_name: String,
    size: i64,
    content_hash: Vec<u8>,
    access_password: Vec<u8>,
}


/// Initialize a new upload
///
/// Supply all meta-data about an upload. Returning a unique key and a response-url
/// fragment where the actual content should be posted
/// e.g.)
///   format!("{}{}?key={}", "http://localhost:3000", "/api/upload", "...long-key...")
///
#[post("/api/upload/init", data = "<info>")]
fn api_upload_init(info: Json<UploadInitPost>, conn: db::DbConn) -> Result<Json<JsonValue>> {
    let info = info.decode_hex().expect("bad upload info");
    if info.size > models::UPLOAD_LIMIT_BYTES {
        error!("Upload too large");
        bail_fmt!(ErrorKind::BadRequest, "Upload too large, max bytes: {}", models::UPLOAD_LIMIT_BYTES)
    }
    let uuid = Uuid::new_v4();
    let uuid_hex = uuid.as_bytes().to_hex();

    let access_auth = models::NewAuth::from_bytes(&info.access_password)?.insert(&**conn)?;
    let new_init_upload = models::NewInitUpload {
        uuid: uuid,
        file_name: info.file_name,
        content_hash: info.content_hash,
        size: info.size,
        nonce: info.nonce,
        access_password: access_auth.id,
    };
    new_init_upload.insert(&**conn)?;

    let resp = json!({
        "key": &uuid_hex,
        "response_url": "/api/upload",
    });
    Ok(Json(resp))
}


/// Upload identifier
#[derive(FromForm)]
struct UploadKey{
    key: String,
}


/// Upload encrypted bytes to a specified upload-key
///
/// TODO: Add another upload size check
#[post("/api/upload?<upload_key>", format = "application/octet-stream", data = "<data>")]
fn api_upload_file(upload_key: UploadKey, data: rocket::Data, conn: db::DbConn) -> Result<Json<JsonValue>> {
    use std::str::FromStr;
    use std::io::{Write, BufRead};
    let upload = {
        let now = Utc::now();
        let uuid = Uuid::from_str(&upload_key.key)?;

        let trans = conn.transaction()?;
        let init_upload = models::InitUpload::find(&trans, &uuid)?;
        let file_path = models::Upload::new_file_path(&init_upload.uuid)?;
        init_upload.delete(&trans)?;
        if now.signed_duration_since(init_upload.date_created) > Duration::seconds(models::UPLOAD_TIMEOUT_SECS) {
            error!("Upload request came too late");
            bail_fmt!(ErrorKind::BadRequest, "Upload request came to late");
        }
        let new_upload = init_upload.into_upload(&file_path)?;
        let upload = new_upload.insert(&trans)?;
        trans.commit()?;
        upload
    };

    // In case they lied about the upload size twice...
    let mut byte_count = 0;
    let mut file = fs::File::create(&upload.file_path)?;
    let mut stream = io::BufReader::new(data.open());
    loop {
        let n = {
            let mut buf = stream.fill_buf()?;
            file.write_all(&mut buf)?;
            buf.len()
        };
        stream.consume(n);
        if n == 0 { break; }
        byte_count += n;
        if byte_count as i64 > models::UPLOAD_LIMIT_BYTES {
            error!("Upload too large");
            // if the file deletion fails, the file will eventually be cleaned up
            // by the `admin sweep-files` command
            fs::remove_file(&upload.file_path).ok();
            // drain the rest of the stream
            loop {
                let n = {
                    let buf = stream.fill_buf()?;
                    buf.len()
                };
                stream.consume(n);
                if n == 0 { break; }
            }
            // delete the entry we just made
            upload.delete(&**conn)?;
            bail_fmt!(ErrorKind::BadRequest, "Upload too large, max bytes: {}", models::UPLOAD_LIMIT_BYTES)
        }
    }

    let resp = json!({"ok": "ok"});
    Ok(Json(resp))
}


/// Download identifier and access/auth password
#[derive(Deserialize)]
struct DownloadKeyAccess {
    key: String,
    access_password: String,
}


/// Initialize a download
///
/// Using a key and access-password, obtain the download meta-data (stuff
/// needed for decryption).
#[post("/api/download/init", data = "<download_key>")]
fn api_download_init(download_key: Json<DownloadKeyAccess>, conn: db::DbConn) -> Result<Json<JsonValue>> {
    let uuid_bytes = Vec::from_hex(&download_key.key)?;
    let access_pass_bytes = Vec::from_hex(&download_key.access_password)?;
    let uuid = Uuid::from_bytes(&uuid_bytes)?;
    let upload = models::Upload::find(&**conn, &uuid)?;
    let access_auth = models::Auth::find(&**conn, &upload.access_password)?;
    access_auth.verify(&access_pass_bytes)?;
    Ok(Json(json!({
        "nonce": upload.nonce.to_hex(),
        "size": upload.size,
    })))
}


/// Download encrypted bytes
#[post("/api/download", data = "<download_key>")]
fn api_download(download_key: Json<DownloadKeyAccess>, conn: db::DbConn) -> Result<response::Stream<fs::File>> {
    let uuid_bytes = Vec::from_hex(&download_key.key)?;
    let access_pass_bytes = Vec::from_hex(&download_key.access_password)?;
    let uuid = Uuid::from_bytes(&uuid_bytes)?;
    let upload = models::Upload::find(&**conn, &uuid)?;
    let access_auth = models::Auth::find(&**conn, &upload.access_password)?;
    access_auth.verify(&access_pass_bytes)?;
    let file = fs::File::open(upload.file_path)?;
    Ok(response::Stream::from(file))
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
#[post("/api/download/name", data = "<download_key>")]
fn api_download_name(download_key: Json<DownloadKeyHash>, conn: db::DbConn) -> Result<Json<JsonValue>> {
    let uuid_bytes = Vec::from_hex(&download_key.key)?;
    let hash_bytes = Vec::from_hex(&download_key.hash)?;
    let uuid = Uuid::from_bytes(&uuid_bytes)?;
    let upload = models::Upload::find(&**conn, &uuid)?;
    auth::eq(&hash_bytes, &upload.content_hash)?;
    Ok(Json(json!({"file_name": &upload.file_name})))
}

