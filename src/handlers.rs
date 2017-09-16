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


#[get("/<file..>")]
fn static_files(file: path::PathBuf) -> Option<response::NamedFile> {
    response::NamedFile::open(path::Path::new("static/").join(file)).ok()
}


#[get("/")]
fn index<'a>() -> response::Response<'a> {
    let resp = response::Response::build()
        .header(http::ContentType::HTML)
        .sized_body(io::Cursor::new("<html><body> <p> hello </p> <script src=\"/static/js/app.js\"></script></body></html>"))
        .finalize();
    resp
}


#[derive(Serialize, Deserialize)]
struct Message {
    message: String,
}


#[get("/api/hello", format = "application/json")]
fn api_hello() -> Json<JsonValue> {
    Json(json!({"message": "hey"}))
}

#[post("/api/bye", data = "<msg>")]
fn api_bye<'a>(msg: Json<Message>) -> Json<JsonValue> {
    debug!("msg: {}", msg.message);
    Json(json!({"message": "bye!"}))
}


#[derive(Deserialize)]
struct UploadInfoPost {
    iv: String,
    file_name: String,
    file_size: u32,
    content_hash: String,
    access_password: String,
}
struct UploadInfo {
    iv: Vec<u8>,
    file_name: String,
    file_size: u32,
    content_hash: Vec<u8>,
    access_password: Vec<u8>,
}
impl UploadInfoPost {
    fn decode_hex(&self) -> Result<UploadInfo> {
        Ok(UploadInfo {
            iv: Vec::from_hex(&self.iv)?,
            file_name: self.file_name.to_owned(),
            file_size: self.file_size,
            content_hash: Vec::from_hex(&self.content_hash)?,
            access_password: Vec::from_hex(&self.access_password)?,
        })
    }
}

const UPLOAD_LIMIT_BYTES: u32 = 100_000_000;

#[post("/api/upload/init", data = "<info>")]
fn api_upload_init(info: Json<UploadInfoPost>, conn: db::DbConn) -> Result<Json<JsonValue>> {
    let info = info.decode_hex().expect("bad upload info");
    if info.file_size > UPLOAD_LIMIT_BYTES {
        bail_fmt!(ErrorKind::BadRequest, "Upload too large, max bytes: {}", UPLOAD_LIMIT_BYTES)
    }
    let uuid = Uuid::new_v4();
    let uuid_hex = uuid.as_bytes().to_hex();

    let access_auth = models::NewAuth::from_bytes(&info.access_password)?.insert(&**conn)?;
    let new_init_upload = models::NewInitUpload {
        uuid: uuid,
        file_name: info.file_name,
        content_hash: info.content_hash,
        iv: info.iv,
        access_password: access_auth.id,
    };
    new_init_upload.insert(&**conn)?;

    let resp = json!({
        "key": &uuid_hex,
        "responseUrl": "/api/upload",
    });
    Ok(Json(resp))
}


#[derive(FromForm)]
struct UploadId{
    key: String,
}


const UPLOAD_TIMEOUT: i64 = 30;  // seconds

#[post("/api/upload?<upload_info>", format = "text/plain", data = "<data>")]
fn api_upload_file(upload_info: UploadId, data: rocket::Data, conn: db::DbConn) -> Result<Json<JsonValue>> {
    use std::str::FromStr;
    let upload = {
        let trans = conn.transaction()?;
        let uuid = Uuid::from_str(&upload_info.key)?;
        let init_upload = models::InitUpload::find(&trans, &uuid)?;
        let file_path = models::Upload::new_file_path(&init_upload.uuid)?;
        init_upload.delete(&trans)?;
        debug!("Deleted `init_upload` id={}", init_upload.id);
        // assert within timespan
        let now = Utc::now();
        if now.signed_duration_since(init_upload.date_created) > Duration::seconds(UPLOAD_TIMEOUT) {
            error!("Upload request came too late");
            bail_fmt!(ErrorKind::BadRequest, "Upload request came to late");
        }
        let new_upload = init_upload.into_upload(&file_path)?;
        let upload = new_upload.insert(&trans)?;
        debug!("Created `upload` id={}", upload.id);
        trans.commit()?;
        upload
    };

    data.stream_to_file(&upload.file_path)?;

    let resp = json!({"ok": "ok"});
    Ok(Json(resp))
}


#[derive(Deserialize)]
struct DownloadIdAccess {
    key: String,
    access_password: String,
}


#[post("/api/download/iv", data = "<download_info>")]
fn api_download_iv(download_info: Json<DownloadIdAccess>, conn: db::DbConn) -> Result<Json<JsonValue>> {
    let uuid_bytes = Vec::from_hex(&download_info.key)?;
    let uuid = Uuid::from_bytes(&uuid_bytes)?;
    let upload = models::Upload::find(&**conn, &uuid)?;
    let access_auth = models::Auth::find(&**conn, &upload.access_password)?;
    let access_pass_bytes = Vec::from_hex(&download_info.access_password)?;
    access_auth.verify(&access_pass_bytes)?;
    Ok(Json(json!({"iv": upload.iv.as_slice().to_hex()})))
}


#[post("/api/download", data = "<download_info>")]
fn api_download(download_info: Json<DownloadIdAccess>, conn: db::DbConn) -> Result<response::Stream<fs::File>> {
    let uuid_bytes = Vec::from_hex(&download_info.key)?;
    let uuid = Uuid::from_bytes(&uuid_bytes)?;
    let upload = models::Upload::find(&**conn, &uuid)?;
    let access_auth = models::Auth::find(&**conn, &upload.access_password)?;
    let access_pass_bytes = Vec::from_hex(&download_info.access_password)?;
    access_auth.verify(&access_pass_bytes)?;
    let file = fs::File::open(upload.file_path)?;
    Ok(response::Stream::from(file))
}


#[derive(Deserialize)]
struct DownloadIdHash {
    key: String,
    hash: String,
}


#[post("/api/download/name", data = "<download_info>")]
fn api_download_name(download_info: Json<DownloadIdHash>, conn: db::DbConn) -> Result<Json<JsonValue>> {
    let uuid_bytes = Vec::from_hex(&download_info.key)?;
    let uuid = Uuid::from_bytes(&uuid_bytes)?;
    let upload = models::Upload::find(&**conn, &uuid)?;
    let hash_bytes = Vec::from_hex(&download_info.hash)?;
    auth::eq(&hash_bytes, &upload.content_hash)?;
    Ok(Json(json!({"file_name": &upload.file_name})))
}
