use std::path;
use std::io;

use rocket;
use rocket::response;
use rocket::http;
use rocket_contrib::{Json, Value as JsonValue};
use hex::{FromHex, ToHex};
use uuid::Uuid;
use chrono::{Utc, Duration};


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
    filename: String,
    access_password: String,
    encrypt_password: String,
}
struct UploadInfo {
    iv: Vec<u8>,
    filename: String,
    access_password: Vec<u8>,
    encrypt_password: Vec<u8>,
}
impl UploadInfoPost {
    fn decode_hex(&self) -> Result<UploadInfo, &'static str> {
        macro_rules! bytes_from_hex {
            ($hex:expr) => {
                match Vec::from_hex($hex) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        error!("Failed decoding initialization vector: {}", e);
                        return Err("Failed parsing init vec");
                    }
                }
            }
        }
        Ok(UploadInfo {
            iv: bytes_from_hex!(&self.iv),
            filename: self.filename.to_owned(),
            access_password: bytes_from_hex!(&self.access_password),
            encrypt_password: bytes_from_hex!(&self.encrypt_password),
        })
    }
}

#[post("/api/upload/init", data = "<info>")]
fn api_upload_init(info: Json<UploadInfoPost>) -> Json<JsonValue> {
    let info = info.decode_hex().expect("bad upload info");
    let uuid = Uuid::new_v4();
    let uuid_hex = uuid.as_bytes().to_hex();
    let date_initialized = Utc::now();
    debug!("date-initialized: {:?}", date_initialized);
    debug!("iv: {:?}", info.iv);
    debug!("filename: {:?}", info.filename);
    debug!("access-pass: {:?}", ::std::str::from_utf8(&*info.access_password).unwrap());
    debug!("encrypt-pass: {:?}", ::std::str::from_utf8(&*info.encrypt_password).unwrap());
    let resp = json!({
        "uuid": &uuid_hex,
        "responseUrl": "/api/upload",
    });
    Json(resp)
}


#[derive(FromForm)]
struct UploadId {
    uuid: String,
    hash: String,
}


const UPLOAD_TIMEOUT: i64 = 30;  // seconds

#[post("/api/upload?<upload_id>", format = "text/plain", data = "<data>")]
fn api_upload_file(upload_id: UploadId, data: rocket::Data) -> Result<Json<JsonValue>, &'static str> {
    use std::io::Read;
    let then = Utc::now();  // replace
    let now = Utc::now();
    if now.signed_duration_since(then) > Duration::seconds(UPLOAD_TIMEOUT) {
        error!("Upload request came too late");
        return Err("Upload request came to late");
    }
    debug!("uuid-hex: {}", upload_id.uuid);
    debug!("hash-hex: {}", upload_id.hash);

    // update this to just stream directly to file
    let mut buf = String::new();
    data.open().read_to_string(&mut buf).unwrap();
    let bytes = Vec::from_hex(&buf).unwrap();
    debug!("{:?}", bytes);

    let resp = json!({"ok": "ok"});
    Ok(Json(resp))
}
