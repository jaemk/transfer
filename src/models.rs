use postgres::{self, GenericConnection};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use errors::*;


pub struct NewInitUpload {
    pub uuid: Uuid,
    pub file_name: String,
    pub access_password: i32,
    pub encrypt_password: i32,
}
impl NewInitUpload {
    pub fn insert<T: GenericConnection>(self, conn: &T) -> Result<InitUpload> {
        unimplemented!()
    }
}


pub struct InitUpload {
    pub id: i32,
    pub uuid: Uuid,
    pub file_name: String,
    pub access_password: i32,
    pub encrypt_password: i32,
    pub date_created: DateTime<Utc>,
}
impl InitUpload {
    pub fn into_upload(self, content_hash: Vec<u8>, file_path: String) -> NewUpload {
        NewUpload {
            uuid: self.uuid,
            content_hash: content_hash,
            file_name: self.file_name,
            file_path: file_path,
            access_password: self.access_password,
            encrypt_password: self.encrypt_password,
        }
    }
}


pub struct NewUpload {
    pub uuid: Uuid,
    pub content_hash: Vec<u8>,
    pub file_name: String,
    pub file_path: String,
    pub access_password: i32,
    pub encrypt_password: i32,
}
impl NewUpload {
    pub fn insert<T: GenericConnection>(self, conn: &T) -> Result<Upload> {
        unimplemented!()
    }
}


pub struct Upload {
    pub id: i32,
    pub uuid: Uuid,
    pub content_hash: Vec<u8>,
    pub file_name: String,
    pub file_path: String,
    pub access_password: i32,
    pub encrypt_password: i32,
    pub date_created: DateTime<Utc>,
}
