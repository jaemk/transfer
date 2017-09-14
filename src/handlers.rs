use std::path;
use std::io;

use rocket;
use rocket::response;
use rocket::http;
use rocket_contrib::{Json, Value as JsonValue};
use hex::{FromHex, ToHex};
use uuid::Uuid;
//use chrono::{Utc, Duration};

use db;
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
    access_password: String,
    encrypt_password: String,
}
struct UploadInfo {
    iv: Vec<u8>,
    file_name: String,
    access_password: Vec<u8>,
    encrypt_password: Vec<u8>,
}
impl UploadInfoPost {
    fn decode_hex(&self) -> Result<UploadInfo> {
        Ok(UploadInfo {
            iv: Vec::from_hex(&self.iv)?,
            file_name: self.file_name.to_owned(),
            access_password: Vec::from_hex(&self.access_password)?,
            encrypt_password: Vec::from_hex(&self.encrypt_password)?,
        })
    }
}

#[post("/api/upload/init", data = "<info>")]
fn api_upload_init(info: Json<UploadInfoPost>, conn: db::DbConn) -> Result<Json<JsonValue>> {
    let info = info.decode_hex().expect("bad upload info");
    let uuid = Uuid::new_v4();
    let uuid_hex = uuid.as_bytes().to_hex();

    let access_auth = models::NewAuth::from_bytes(&info.access_password)?.insert(&**conn)?;
    let encrypt_auth = models::NewAuth::from_bytes(&info.encrypt_password)?.insert(&**conn)?;
    let new_init_upload = models::NewInitUpload {
        uuid: uuid,
        file_name: info.file_name,
        access_password: access_auth.id,
        encrypt_password: encrypt_auth.id,
    };
    let new_init_upload = new_init_upload.insert(&**conn)?;

    let resp = json!({
        "uuid": &uuid_hex,
        "responseUrl": "/api/upload",
    });
    Ok(Json(resp))
}


#[derive(FromForm)]
struct UploadId{
    uuid: String,
    hash: String,
}


const UPLOAD_TIMEOUT: i64 = 30;  // seconds

#[post("/api/upload?<upload_info>", format = "text/plain", data = "<data>")]
fn api_upload_file(upload_info: UploadId, data: rocket::Data, conn: db::DbConn) -> Result<Json<JsonValue>> {
    use std::str::FromStr;
    let trans = conn.transaction()?;
    let uuid = Uuid::from_str(&upload_info.uuid)?;
    let file_path = models::Upload::new_file_path(&uuid)?;
    let init_upload = models::InitUpload::find(&trans, &uuid)?;
    assert_eq!(init_upload.delete(&trans)?, 1);
    // assert within timespan
    //let now = Utc::now();
    //if now.signed_duration_since(then) > Duration::seconds(UPLOAD_TIMEOUT) {
    //    error!("Upload request came too late");
    //    bail!("Upload request came to late");
    //}
    let hash_hex = Vec::from_hex(upload_info.hash)?;
    let new_upload = init_upload.into_upload(hash_hex, &file_path)?;
    let upload = new_upload.insert(&trans)?;

    data.stream_to_file(&file_path)?;

    let resp = json!({"ok": "ok"});
    Ok(Json(resp))
}
