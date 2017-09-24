/*!
Project Error type and conversions
*/
use std;
use log;
use uuid;
use hex;
use rocket;
use postgres;
use ring;
use migrant_lib;


error_chain! {
    foreign_links {
        Io(std::io::Error);
        LogInit(log::SetLoggerError) #[doc = "Error initializing env_logger"];
        ParseInt(std::num::ParseIntError);
        Uuid(uuid::ParseError);
        Hex(hex::FromHexError);
        RocketConfig(rocket::config::ConfigError) #[doc = "Error finalizing rocket config"];
        Postgres(postgres::error::Error);
        RingUnspecified(ring::error::Unspecified);
        MigrantLib(migrant_lib::Error);
    }
    errors {
        DoesNotExist(s: String) {
            description("Query result does not exist")
            display("DoesNotExist Error: {}", s)
        }
        MultipleRecords(s: String) {
            description("Query returned multiple records, expected one")
            display("MultipleRecords Error: {}", s)
        }
        InvalidHashArgs(s: String) {
            description("Hash arguments have invalid number of bytes")
            display("InvalidHashArgs Error: {}", s)
        }
        PathRepr(p: std::path::PathBuf) {
            description("Unable to convert Path to String")
            display("PathRepr Error: Unable to convert Path to String: {:?}", p)
        }
        BadRequest(s: String) {
            description("Bad request")
            display("BadRequest: {}", s)
        }
        UploadTooLarge(s: String) {
            description("Upload too large")
            display("UploadTooLarge: {}", s)
        }
        OutOfSpace(s: String) {
            description("Out of storage space")
            display("OutOfSpace: {}", s)
        }
        UnequalBytes(s: String) {
            description("Unequal bytes")
            display("UnequalBytes Error: {}", s)
        }
        InvalidAuth(s: String) {
            description("Invalid auth")
            display("InvalidAuth Error: {}", s)
        }
        InvalidDateTimeMathOffset(s: String) {
            description("Invalid DateTime Math")
            display("InvalidDateTimeMathOffset Error: {}", s)
        }
        ConfirmationError(s: String) {
            description("Confirmation error")
            display("ConfirmationError: {}", s)
        }
    }
}

impl Error {
    pub fn does_not_exist(&self) -> bool {
        match *self.kind() {
            ErrorKind::DoesNotExist(_) => true,
            _ => false,
        }
    }
}

impl<'r> rocket::response::Responder<'r> for Error {
    fn respond_to(self, _: &rocket::request::Request) -> rocket::response::Result<'r> {
        use ErrorKind::*;
        match *self.kind() {
            BadRequest(ref s) => {
                let body = json!({"error": s}).to_string();
                rocket::Response::build()
                    .status(rocket::http::Status::BadRequest)
                    .header(rocket::http::ContentType::JSON)
                    .sized_body(std::io::Cursor::new(body))
                    .ok()
            }
            InvalidAuth(ref s) => {
                let body = json!({"error": s}).to_string();
                rocket::Response::build()
                    .status(rocket::http::Status::Unauthorized)
                    .header(rocket::http::ContentType::JSON)
                    .sized_body(std::io::Cursor::new(body))
                    .ok()
            }
            OutOfSpace(ref s) => {
                let body = json!({"error": s}).to_string();
                rocket::Response::build()
                    .status(rocket::http::Status::ServiceUnavailable)
                    .header(rocket::http::ContentType::JSON)
                    .sized_body(std::io::Cursor::new(body))
                    .ok()
            }
            UploadTooLarge(ref s) => {
                let body = json!({"error": s}).to_string();
                rocket::Response::build()
                    .status(rocket::http::Status::PayloadTooLarge)
                    .header(rocket::http::ContentType::JSON)
                    .sized_body(std::io::Cursor::new(body))
                    .ok()
            }
            DoesNotExist(ref s) => {
                let body = json!({"error": s}).to_string();
                rocket::Response::build()
                    .status(rocket::http::Status::NotFound)
                    .header(rocket::http::ContentType::JSON)
                    .sized_body(std::io::Cursor::new(body))
                    .ok()
            }
            _ => Err(rocket::http::Status::InternalServerError),
        }
    }
}

