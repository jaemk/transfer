/*!
Database models

*/
use std::env;
use std::path::{Path, PathBuf};
use postgres::{self, GenericConnection};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use auth;
use errors::*;


pub const UPLOAD_LIMIT_BYTES: i64 = 200_000_000;  // 200mb
pub const UPLOAD_TIMEOUT_SECS: i64 = 30;
pub const UPLOAD_LIFESPAN_SECS_DEFAULT: i64 = 60 * 60 * 24;  // 1 day
pub const MAX_COMBINED_UPLOAD_BYTES: i64 = 5_000_000_000;  // 5gb


pub trait FromRow {
    /// Return the associated database table name
    fn table_name() -> &'static str;
    /// Convert a `postgres::row::Row` into an instance
    fn from_row(row: postgres::rows::Row) -> Self;
}


/// For inserting a new `Auth` record
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
        try_query_to_model!(conn.query(stmt, &[&self.salt, &self.hash]);
                            Auth;
                            id: 0, date_created: 1;
                            salt: self.salt, hash: self.hash)
    }
}


/// Maps to db table `auth`
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
    /// Return the `auth` record for the given `id` or `ErrorKind::DoesNotExist`
    pub fn find<T: GenericConnection>(conn: &T, id: &i32) -> Result<Self> {
        let stmt = "select id, salt, hash, date_created from auth \
                    where id = $1";
        try_query_one!(conn.query(stmt, &[id]), Auth)
    }

    /// Try verifying the current `auth` record against a set of bytes, returning
    /// `Ok` if verification passes or `ErrorKind::InvalidAuth`
    pub fn verify(&self, other_bytes: &[u8]) -> Result<()> {
        let other_hash = auth::bcrypt_hash(other_bytes, &self.salt)?;
        auth::eq(&self.hash, &other_hash)
            .map_err(|_| format_err!(ErrorKind::InvalidAuth, "Invalid authentication"))?;
        Ok(())
    }
}


/// For initializing a new `InitUpload` record
pub struct NewInitUpload {
    pub uuid: Uuid,
    pub file_name: String,
    pub content_hash: Vec<u8>,
    pub size: i64,
    pub nonce: Vec<u8>,
    pub access_password: i32,
    pub download_limit: Option<i32>,
    pub expire_date: DateTime<Utc>,
}
impl NewInitUpload {
    pub fn insert<T: GenericConnection>(self, conn: &T) -> Result<InitUpload> {
        let stmt = "insert into init_upload \
                    (uuid_, file_name, content_hash, size_, nonce, access_password, download_limit, expire_date) \
                    values ($1, $2, $3, $4, $5, $6, $7, $8) \
                    returning id, date_created";
        try_query_to_model!(conn.query(stmt, &[&self.uuid, &self.file_name, &self.content_hash, &self.size,
                                        &self.nonce, &self.access_password, &self.download_limit, &self.expire_date]);
                            InitUpload;
                            id: 0, date_created: 1;
                            uuid: self.uuid, file_name: self.file_name, content_hash: self.content_hash,
                            size: self.size, nonce: self.nonce, access_password: self.access_password,
                            download_limit: self.download_limit, expire_date: self.expire_date)
    }
}


/// Maps to db table `init_upload`
pub struct InitUpload {
    pub id: i32,
    pub uuid: Uuid,
    pub file_name: String,
    pub content_hash: Vec<u8>,
    pub size: i64,
    pub nonce: Vec<u8>,
    pub access_password: i32,
    pub download_limit: Option<i32>,
    pub expire_date: DateTime<Utc>,
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
            size:               row.get(4),
            nonce:              row.get(5),
            access_password:    row.get(6),
            download_limit:     row.get(7),
            expire_date:        row.get(8),
            date_created:       row.get(9),
        }
    }
}
impl InitUpload {
    /// Return the `init_upload` record for the given `uuid` or `ErrorKind::DoesNotExist`
    pub fn find<T: GenericConnection>(conn: &T, uuid: &Uuid) -> Result<Self> {
        let stmt = "select * \
                    from init_upload \
                    where uuid_ = $1";
        try_query_one!(conn.query(stmt, &[uuid]), InitUpload)
    }

    /// Try deleting the current record from the database, returning the number of items deleted
    pub fn delete<T: GenericConnection>(&self, conn: &T) -> Result<i64> {
        let stmt = "with deleted as (delete from init_upload where id = $1 returning 1) \
                    select count(*) from deleted";
        try_query_aggregate!(conn.query(stmt, &[&self.id]), i64)
    }

    /// Convert the current `InitUpload` into a `NewUpload`
    ///
    /// Converts current instance with a given `file_path` where the associated upload data
    /// will be saved. Note, the current `InitUpload` should be deleted before being converted.
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
            size: self.size,
            file_name: self.file_name,
            file_path: pb,
            nonce: self.nonce,
            access_password: self.access_password,
            download_limit: self.download_limit,
            expire_date: self.expire_date,
        })
    }

    /// Try deleting all `init_upload` records that are older than the current `UPLOAD_TIMEOUT_SECS`
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


/// For initializing a new `Upload` record
pub struct NewUpload {
    pub uuid: Uuid,
    pub content_hash: Vec<u8>,
    pub size: i64,
    pub file_name: String,
    pub file_path: String,
    pub nonce: Vec<u8>,
    pub access_password: i32,
    pub download_limit: Option<i32>,
    pub expire_date: DateTime<Utc>,
}
impl NewUpload {
    pub fn insert<T: GenericConnection>(self, conn: &T) -> Result<Upload> {
        let stmt = "insert into upload \
                    (uuid_, content_hash, size_, file_name, file_path, nonce, access_password, download_limit, expire_date) \
                    values ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
                    returning id, deleted, date_created";
        try_query_to_model!(conn.query(stmt, &[&self.uuid, &self.content_hash, &self.size, &self.file_name,
                                                &self.file_path, &self.nonce, &self.access_password,
                                                &self.download_limit, &self.expire_date]);
                            Upload;
                            id: 0, deleted: 1, date_created: 2;
                            uuid: self.uuid, content_hash: self.content_hash, size: self.size, file_name: self.file_name,
                            file_path: self.file_path, nonce: self.nonce, access_password: self.access_password,
                            download_limit: self.download_limit, expire_date: self.expire_date)
    }
}


/// Maps to db table `upload`
pub struct Upload {
    pub id: i32,
    pub uuid: Uuid,
    pub content_hash: Vec<u8>,
    pub size: i64,
    pub file_name: String,
    pub file_path: String,
    pub nonce: Vec<u8>,
    pub access_password: i32,
    pub download_limit: Option<i32>,
    pub expire_date: DateTime<Utc>,
    pub deleted: bool,
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
            size:               row.get(3),
            file_name:          row.get(4),
            file_path:          row.get(5),
            nonce:              row.get(6),
            access_password:    row.get(7),
            download_limit:     row.get(8),
            expire_date:        row.get(9),
            deleted:            row.get(10),
            date_created:       row.get(11),
        }
    }
}
impl Upload {
    /// Convert an `Upload`s `uuid` into a valid upload file-path
    pub fn new_file_path(uuid: &Uuid) -> Result<PathBuf> {
        use hex::ToHex;
        let base_dir = env::current_dir()?;
        Ok(base_dir.join("uploads").join(uuid.as_bytes().to_hex()))
    }

    /// Return the `upload` record for the given `uuid` or `ErrorKind::DoesNotExist`
    pub fn find<T: GenericConnection>(conn: &T, uuid: &Uuid) -> Result<Self> {
        let stmt = "select * \
                    from upload \
                    where uuid_ = $1 and deleted = false";
        try_query_one!(conn.query(stmt, &[uuid]), Upload)
    }

    /// Check if an `upload` record with the given `file_name` exists and is available for download
    pub fn uuid_exists_available<T: GenericConnection>(conn: &T, uuid: &Uuid) -> Result<bool> {
        let stmt = "select exists(select 1 from upload where uuid_ = $1 and deleted = false)";
        try_query_aggregate!(conn.query(stmt, &[&uuid]), bool)
    }

    /// Return a collection of `Upload` instances that are older than `UPLOAD_MAX_LIFE_SECS`
    /// TODO: Also return uploads that are over their download limit
    pub fn select_outdated<T: GenericConnection>(conn: &T) -> Result<Vec<Self>> {
        let stmt = "select * \
                    from upload \
                    where expire_date < $1";
        let now = Utc::now();
        try_query_vec!(conn.query(stmt, &[&now]), Upload)
    }

    /// Try marking the current instance deleted, returning the number of items marked
    pub fn delete<T: GenericConnection>(&self, conn: &T) -> Result<i64> {
        let stmt = "with deleted as (update upload where id = $1 set deleted = true returning 1) \
                    select count(*) from deleted";
        try_query_aggregate!(conn.query(stmt, &[&self.id]), i64)
    }
}


#[allow(dead_code)]
pub struct Status {
    id: i32,
    upload_count: i64,
    total_bytes: i64,
    date_modified: DateTime<Utc>,
}
impl FromRow for Status {
    fn table_name() -> &'static str {
        "status"
    }
    fn from_row(row: postgres::rows::Row) -> Self {
        Self {
            id:             row.get(0),
            upload_count:   row.get(1),
            total_bytes:    row.get(2),
            date_modified:  row.get(3),
        }
    }
}
impl Status {
    pub fn init_load<T: GenericConnection>(conn: &T) -> Result<Self> {
        let trans = conn.transaction()?;
        let status = Self::load(&trans);
        let status = match status {
            Err(ref e) if e.does_not_exist() => Self::init(&trans),
            status => status,
        };
        trans.commit()?;
        status
    }

    pub fn load<T: GenericConnection>(conn: &T) -> Result<Self> {
        let stmt = "select id, upload_count, total_bytes, date_modified from status";
        try_query_one!(conn.query(stmt, &[]), Status)
    }

    pub fn init<T: GenericConnection>(conn: &T) -> Result<Self> {
        let stmt = "insert into status (upload_count, total_bytes, date_modified) \
                    values ($1, $2, $3) \
                    returning id";
        let now = Utc::now();
        try_query_to_model!(conn.query(stmt, &[&0i64, &0i64, &now]);
                            Status;
                            id: 0;
                            upload_count: 0, total_bytes: 0, date_modified: now)
    }

    pub fn can_fit<T: GenericConnection>(conn: &T, n_bytes: i64) -> Result<bool> {
        let status = Self::load(conn)?;
        Ok((status.total_bytes + n_bytes) < MAX_COMBINED_UPLOAD_BYTES)
    }

    pub fn inc_upload<T: GenericConnection>(conn: &T, n_bytes: i64) -> Result<Self> {
        let stmt = "with updated as (update status set \
                                        upload_count = upload_count + 1, \
                                        total_bytes = total_bytes + $1, \
                                        date_modified = $2 \
                                        returning id, upload_count, total_bytes) \
                    select * from updated";
        let now = Utc::now();
        try_query_to_model!(conn.query(stmt, &[&n_bytes, &now]);
                            Status;
                            id: 0, upload_count: 1, total_bytes: 2;
                            date_modified: now)
    }

    pub fn dec_upload<T: GenericConnection>(conn: &T, n_bytes: i64) -> Result<Self> {
        let stmt = "with updated as (update status set \
                                        upload_count = upload_count - 1, \
                                        total_bytes = total_bytes - $1, \
                                        date_modified = $2 \
                                        returning id, upload_count, total_bytes) \
                    select * from updated";
        let now = Utc::now();
        try_query_to_model!(conn.query(stmt, &[&n_bytes, &now]);
                            Status;
                            id: 0, upload_count: 1, total_bytes: 2;
                            date_modified: now)
    }
}

