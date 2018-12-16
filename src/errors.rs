/*!
Project Error type and conversions
*/
use std;
use log;
use uuid;
use hex;
use serde_json;
use postgres;
use r2d2;
use ring;
use migrant_lib;
use xdg;


error_chain! {
    foreign_links {
        Io(std::io::Error);
        LogInit(log::SetLoggerError) #[doc = "Error initializing env_logger"];
        ConnError(r2d2::Error);
        ParseInt(std::num::ParseIntError);
        Json(serde_json::Error);
        Uuid(uuid::ParseError);
        InvalidHex(hex::FromHexError);
        Postgres(postgres::error::Error);
        RingUnspecified(ring::error::Unspecified);
        MigrantLib(migrant_lib::Error);
        Xdg(xdg::BaseDirectoriesError);
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

