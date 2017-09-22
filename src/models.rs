use std::env;
use std::path::{Path, PathBuf};
use postgres::{self, GenericConnection};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use auth;
use errors::*;


pub const UPLOAD_LIMIT_BYTES: i64 = 100_000_000;
pub const UPLOAD_TIMEOUT_SECS: i64 = 30;
pub const UPLOAD_MAX_LIFE_SECS: i64 = 60 * 60 * 24;


pub trait FromRow {
    fn table_name() -> &'static str;
    fn from_row(row: postgres::rows::Row) -> Self;
}



pub struct NewAuth {
    pub salt: Vec<u8>,
    pub hash: Vec<u8>,
}
impl NewAuth {
    pub fn from_bytes(pass: &[u8]) -> Result<Self> {
        let salt = auth::new_salt()?;
        let hash = auth::bcrypt_hash(pass, &salt)?;
        Ok(Self {
            salt, hash
        })
    }

    pub fn insert<T: GenericConnection>(self, conn: &T) -> Result<Auth> {
        let stmt = "insert into auth (salt, hash) values ($1, $2) \
                    returning id, date_created";
        try_insert_to_model!(conn.query(stmt, &[&self.salt, &self.hash]);
                             Auth;
                             id: 0, date_created: 1;
                             salt: self.salt, hash: self.hash)
    }
}


pub struct Auth {
    pub id: i32,
    pub salt: Vec<u8>,
    pub hash: Vec<u8>,
    pub date_created: DateTime<Utc>,
}
impl FromRow for Auth {
    fn table_name() -> &'static str {
        "auth"
    }
    fn from_row(row: postgres::rows::Row) -> Self {
        Self {
            id:             row.get(0),
            salt:           row.get(1),
            hash:           row.get(2),
            date_created:   row.get(3),
        }
    }
}
impl Auth {
    pub fn find<T: GenericConnection>(conn: &T, id: &i32) -> Result<Self> {
        let stmt = "select id, salt, hash, date_created from auth \
                    where id = $1 \
                    limit 1";
        try_query_one!(conn.query(stmt, &[id]), Auth)
    }

    pub fn verify(&self, other_bytes: &[u8]) -> Result<()> {
        let other_hash = auth::bcrypt_hash(other_bytes, &self.salt)?;
        auth::eq(&self.hash, &other_hash)
            .map_err(|_| format_err!(ErrorKind::InvalidAuth, "Invalid authentication"))?;
        Ok(())
    }
}


pub struct NewInitUpload {
    pub uuid: Uuid,
    pub file_name: String,
    pub content_hash: Vec<u8>,
    pub file_size: i64,
    pub nonce: Vec<u8>,
    pub access_password: i32,
}
impl NewInitUpload {
    pub fn insert<T: GenericConnection>(self, conn: &T) -> Result<InitUpload> {
        let stmt = "insert into init_upload (uuid_, file_name, content_hash, file_size, nonce, access_password) \
                    values ($1, $2, $3, $4, $5, $6) \
                    returning id, date_created";
        try_insert_to_model!(conn.query(stmt, &[&self.uuid, &self.file_name, &self.content_hash, &self.file_size,
                                        &self.nonce, &self.access_password]);
                            InitUpload;
                            id: 0, date_created: 1;
                            uuid: self.uuid, file_name: self.file_name, content_hash: self.content_hash,
                            file_size: self.file_size, nonce: self.nonce, access_password: self.access_password)
    }
}


pub struct InitUpload {
    pub id: i32,
    pub uuid: Uuid,
    pub file_name: String,
    pub content_hash: Vec<u8>,
    pub file_size: i64,
    pub nonce: Vec<u8>,
    pub access_password: i32,
    pub date_created: DateTime<Utc>,
}
impl FromRow for InitUpload {
    fn table_name() -> &'static str {
        "init_upload"
    }
    fn from_row(row: postgres::rows::Row) -> Self {
        Self {
            id:                 row.get(0),
            uuid:               row.get(1),
            file_name:          row.get(2),
            content_hash:       row.get(3),
            file_size:          row.get(4),
            nonce:              row.get(5),
            access_password:    row.get(6),
            date_created:       row.get(7),
        }
    }
}
impl InitUpload {
    pub fn find<T: GenericConnection>(conn: &T, uuid: &Uuid) -> Result<Self> {
        let stmt = "select id, uuid_, file_name, content_hash, file_size, nonce, access_password, date_created \
                    from init_upload \
                    where uuid_ = $1 \
                    limit 1";
        try_query_one!(conn.query(stmt, &[uuid]), InitUpload)
    }

    pub fn delete<T: GenericConnection>(&self, conn: &T) -> Result<i64> {
        let stmt = "with deleted as (delete from init_upload where id = $1 returning 1) \
                    select count(*) from deleted";
        try_query_aggregate!(conn.query(stmt, &[&self.id]), i64)
    }

    pub fn into_upload<T: AsRef<Path>>(self, file_path: T) -> Result<NewUpload> {
        let pb = Path::to_str(file_path.as_ref())
            .map(str::to_string)
            .ok_or_else(|| {
                let pb = Path::to_owned(file_path.as_ref());
                ErrorKind::PathRepr(pb)
            })?;
        Ok(NewUpload {
            uuid: self.uuid,
            content_hash: self.content_hash,
            file_size: self.file_size,
            file_name: self.file_name,
            file_path: pb,
            nonce: self.nonce,
            access_password: self.access_password,
        })
    }

    pub fn clear_outdated<T: GenericConnection>(conn: &T) -> Result<i64> {
        let stmt = "with deleted as (delete from init_upload where date_created < $1 returning 1) \
                    select count(*) from deleted";
        let timeout = Duration::seconds(UPLOAD_TIMEOUT_SECS);
        let now = Utc::now();
        let cutoff = now.checked_sub_signed(timeout)
            .ok_or_else(|| format_err!(ErrorKind::InvalidDateTimeMathOffset, "Error subtracting {} secs from {:?}",
                                       UPLOAD_TIMEOUT_SECS, now))?;
        try_query_aggregate!(conn.query(stmt, &[&cutoff]), i64)
    }
}


pub struct NewUpload {
    pub uuid: Uuid,
    pub content_hash: Vec<u8>,
    pub file_size: i64,
    pub file_name: String,
    pub file_path: String,
    pub nonce: Vec<u8>,
    pub access_password: i32,
}
impl NewUpload {
    pub fn insert<T: GenericConnection>(self, conn: &T) -> Result<Upload> {
        let stmt = "insert into upload (uuid_, content_hash, file_size, file_name, file_path, nonce, access_password) \
                    values ($1, $2, $3, $4, $5, $6, $7) \
                    returning id, date_created";
        try_insert_to_model!(conn.query(stmt, &[&self.uuid, &self.content_hash, &self.file_size, &self.file_name,
                                                &self.file_path, &self.nonce, &self.access_password]);
                            Upload;
                            id: 0, date_created: 1;
                            uuid: self.uuid, content_hash: self.content_hash, file_size: self.file_size, file_name: self.file_name,
                            file_path: self.file_path, nonce: self.nonce, access_password: self.access_password)
    }
}


pub struct Upload {
    pub id: i32,
    pub uuid: Uuid,
    pub content_hash: Vec<u8>,
    pub file_size: i64,
    pub file_name: String,
    pub file_path: String,
    pub nonce: Vec<u8>,
    pub access_password: i32,
    pub date_created: DateTime<Utc>,
}
impl FromRow for Upload {
    fn table_name() -> &'static str {
        "upload"
    }
    fn from_row(row: postgres::rows::Row) -> Self {
        Self {
            id:                 row.get(0),
            uuid:               row.get(1),
            content_hash:       row.get(2),
            file_size:          row.get(3),
            file_name:          row.get(4),
            file_path:          row.get(5),
            nonce:              row.get(6),
            access_password:    row.get(7),
            date_created:       row.get(8),
        }
    }
}
impl Upload {
    pub fn new_file_path(uuid: &Uuid) -> Result<PathBuf> {
        use hex::ToHex;
        let base_dir = env::current_dir()?;
        Ok(base_dir.join("uploads").join(uuid.as_bytes().to_hex()))
    }

    pub fn find<T: GenericConnection>(conn: &T, uuid: &Uuid) -> Result<Self> {
        let stmt = "select id, uuid_, content_hash, file_size, file_name, file_path, nonce, access_password, date_created \
                    from upload \
                    where uuid_ = $1 \
                    limit 1";
        try_query_one!(conn.query(stmt, &[uuid]), Upload)
    }

    pub fn file_name_exists<T: GenericConnection>(conn: &T, file_name: &str) -> Result<bool> {
        let stmt = "select exists(select 1 from upload where file_name = $1)";
        try_query_aggregate!(conn.query(stmt, &[&file_name]), bool)
    }

    pub fn select_outdated<T: GenericConnection>(conn: &T) -> Result<Vec<Self>> {
        let stmt = "select id, uuid_, content_hash, file_size, file_name, file_path, nonce, access_password, date_created \
                    from upload \
                    where date_created < $1";
        let max_life = Duration::seconds(UPLOAD_MAX_LIFE_SECS);
        let now = Utc::now();
        let cutoff = now.checked_sub_signed(max_life)
            .ok_or_else(|| format_err!(ErrorKind::InvalidDateTimeMathOffset, "Error subtracting {} secs from {:?}",
                                       UPLOAD_MAX_LIFE_SECS, now))?;
        try_query_vec!(conn.query(stmt, &[&cutoff]), Upload)
    }

    pub fn delete<T: GenericConnection>(self, conn: &T) -> Result<i64> {
        let stmt = "with deleted as (delete from upload where id = $1 returning 1) \
                    select count(*) from deleted";
        try_query_aggregate!(conn.query(stmt, &[&self.id]), i64)
    }
}

