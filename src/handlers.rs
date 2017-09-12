use std::path;
use std::io;

use rocket;
use rocket::response;
use rocket::http;
use rocket_contrib::{Json, Value as JsonValue};
use hex::FromHex;


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
    info!("msg: {}", msg.message);
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
    fn decode(&self) -> Result<UploadInfo, &'static str> {
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
fn api_upload(info: Json<UploadInfoPost>) -> Json<JsonValue> {
    let info = info.decode().expect("bad upload info");
    info!("iv: {:?}", info.iv);
    info!("filename: {:?}", info.filename);
    info!("access-pass: {:?}", ::std::str::from_utf8(&*info.access_password).unwrap());
    info!("encrypt-pass: {:?}", ::std::str::from_utf8(&*info.encrypt_password).unwrap());
    let resp = json!({
        "uuid": "1234",
        "responseUrl": "/response/url/uuid",
    });
    Json(resp)
}


#[derive(FromForm)]
struct UploadId {
    uuid: String,
    hash: String,
}


#[post("/api/upload?<upload_id>", format = "text/plain", data = "<data>")]
fn api_upload_file(upload_id: UploadId, data: rocket::Data) -> Json<JsonValue> {
    use std::io::Read;
    info!("uuid: {}", upload_id.uuid);
    info!("hash: {}", upload_id.hash);
    let mut buf = String::new();
    data.open().read_to_string(&mut buf).unwrap();
    let bytes = Vec::from_hex(&buf).unwrap();
    println!("{:?}", bytes);
    let resp = json!({"ok": "ok"});
    Json(resp)
}
