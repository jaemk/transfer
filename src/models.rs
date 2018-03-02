/*!
Database models

*/
use std::env;
use std::path::{Path, PathBuf};
use std::fs;
use postgres::{self, GenericConnection};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use ron;

use auth;
use errors::*;
use {config_dir};


lazy_static! {
    pub static ref CONFIG: Config = {
        let config_dir_ = config_dir().expect("Failed getting xdg config dir");
        let f = match fs::File::open(config_dir_.join("config.ron")) {
            Err(_) => {
                let cwd = env::current_dir().expect("current dir error");
                fs::File::open(cwd.join("config.ron")).expect("Failed opening config file")
            }
            Ok(f) => f,
        };
        ron::de::from_reader(f).expect("Failed parsing config file")
    };
}


#[derive(Debug, Deserialize)]
pub struct Config {
    pub upload_limit_bytes: i64,
    pub upload_timeout_secs: i64,
    pub upload_lifespan_secs_default: i64,
    pub max_combined_upload_bytes: i64,
    pub download_timeout_secs: i64,
    pub download_limit_default: Option<i32>,
    pub expired_cleanup_interval_secs: u64,
}


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
    pub fn from_pass_bytes(pass: &[u8]) -> Result<Self> {
        let salt = auth::new_salt()?;
        let sha = auth::sha256(pass);
        let hash = auth::bcrypt_hash(&sha, &salt)?;
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
    pub fn verify(&self, other_pass_bytes: &[u8]) -> Result<()> {
        let other_sha = auth::sha256(other_pass_bytes);
        let other_hash = auth::bcrypt_hash(&other_sha, &self.salt)?;
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
    pub deletion_password: Option<i32>,
    pub download_limit: Option<i32>,
    pub expire_date: DateTime<Utc>,
}
impl NewInitUpload {
    pub fn insert<T: GenericConnection>(self, conn: &T) -> Result<InitUpload> {
        let stmt = "insert into init_upload \
                    (uuid_, file_name, content_hash, size_, nonce, access_password, deletion_password, download_limit, expire_date) \
                    values ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
                    returning id, date_created";
        try_query_to_model!(conn.query(stmt, &[&self.uuid, &self.file_name, &self.content_hash, &self.size,
                                        &self.nonce, &self.access_password, &self.deletion_password,
                                        &self.download_limit, &self.expire_date]);
                            InitUpload;
                            id: 0, date_created: 1;
                            uuid: self.uuid, file_name: self.file_name, content_hash: self.content_hash,
                            size: self.size, nonce: self.nonce, access_password: self.access_password,
                            deletion_password: self.deletion_password, download_limit: self.download_limit,
                            expire_date: self.expire_date)
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
    pub deletion_password: Option<i32>,
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
            deletion_password:  row.get(7),
            download_limit:     row.get(8),
            expire_date:        row.get(9),
            date_created:       row.get(10),
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
            deletion_password: self.deletion_password,
            download_limit: self.download_limit,
            expire_date: self.expire_date,
        })
    }

    pub fn still_valid(&self, dt: &DateTime<Utc>) -> bool {
        dt.signed_duration_since(self.date_created) <= Duration::seconds(CONFIG.upload_timeout_secs)
    }

    /// Try deleting all `init_upload` records that are older than the current `CONFIG.upload_timeout_secs`
    pub fn clear_outdated<T: GenericConnection>(conn: &T) -> Result<i64> {
        let stmt = "with deleted as (delete from init_upload where date_created < $1 returning 1) \
                    select count(*) from deleted";
        let timeout = Duration::seconds(CONFIG.upload_timeout_secs);
        let now = Utc::now();
        let cutoff = now.checked_sub_signed(timeout)
            .ok_or_else(|| format_err!(ErrorKind::InvalidDateTimeMathOffset, "Error subtracting {} secs from {:?}",
                                       CONFIG.upload_timeout_secs, now))?;
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
    pub deletion_password: Option<i32>,
    pub download_limit: Option<i32>,
    pub expire_date: DateTime<Utc>,
}
impl NewUpload {
    pub fn insert<T: GenericConnection>(self, conn: &T) -> Result<Upload> {
        let stmt = "insert into upload \
                    (uuid_, content_hash, size_, file_name, file_path, nonce, access_password, deletion_password, download_limit, expire_date) \
                    values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
                    returning id, deleted, date_created";
        try_query_to_model!(conn.query(stmt, &[&self.uuid, &self.content_hash, &self.size, &self.file_name,
                                                &self.file_path, &self.nonce, &self.access_password, &self.deletion_password,
                                                &self.download_limit, &self.expire_date]);
                            Upload;
                            id: 0, deleted: 1, date_created: 2;
                            uuid: self.uuid, content_hash: self.content_hash, size: self.size, file_name: self.file_name,
                            file_path: self.file_path, nonce: self.nonce, access_password: self.access_password,
                            deletion_password: self.deletion_password, download_limit: self.download_limit, expire_date: self.expire_date)
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
    pub deletion_password: Option<i32>,
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
            deletion_password:  row.get(8),
            download_limit:     row.get(9),
            expire_date:        row.get(10),
            deleted:            row.get(11),
            date_created:       row.get(12),
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

    pub fn get_access_auth<T: GenericConnection>(&self, conn: &T) -> Result<Auth> {
        Auth::find(conn, &self.access_password)
    }

    pub fn get_deletion_auth<T: GenericConnection>(&self, conn: &T) -> Result<Option<Auth>> {
        Ok(match self.deletion_password {
            Some(ref id) => Some(Auth::find(conn, id)?),
            None => None,
        })
    }

    /// Check if an `upload` record with the given `file_name` exists and is available for download
    pub fn uuid_exists_available<T: GenericConnection>(conn: &T, uuid: &Uuid) -> Result<bool> {
        let stmt = "select exists(select 1 from upload where uuid_ = $1 and deleted = false)";
        try_query_aggregate!(conn.query(stmt, &[&uuid]), bool)
    }

    /// Return a collection of `Upload` instances that are older than `UPLOAD_MAX_LIFE_SECS`
    /// or are over their download limit
    pub fn select_outdated<T: GenericConnection>(conn: &T) -> Result<Vec<Self>> {
        let stmt = "select * \
                    from upload \
                    where (expire_date <= $1 and deleted = false) \
                    or id in \
                        (with dl_counts as \
                            (select upload, min(download_limit) as download_limit, count(*) \
                                from download join upload on (upload.id = download.upload) \
                                where deleted = false \
                                group by upload) \
                            select upload from dl_counts where count >= download_limit)";
        let now = Utc::now();
        try_query_vec!(conn.query(stmt, &[&now]), Upload)
    }

    /// Try marking the current instance deleted, returning the number of items marked
    pub fn delete<T: GenericConnection>(&self, conn: &T) -> Result<i64> {
        let stmt = "with deleted as (update upload set deleted = true where id = $1 returning 1) \
                    select count(*) from deleted";
        try_query_aggregate!(conn.query(stmt, &[&self.id]), i64)
    }

    pub fn download_count<T: GenericConnection>(&self, conn: &T) -> Result<i64> {
        let stmt = "select count(*) from download where upload = $1";
        try_query_aggregate!(conn.query(stmt, &[&self.id]), i64)
    }
}


/// Download type (usage) for `InitDownload`s
#[derive(Debug, Eq, PartialEq)]
pub enum DownloadType {
    Content,
    Confirm,
}
impl DownloadType {
    pub fn as_str(&self) -> &'static str {
        use self::DownloadType::*;
        match *self {
            Content => "content",
            Confirm => "confirm",
        }
    }
}


/// For initializing a new `InitDownload` record
pub struct NewInitDownload {
    pub uuid: Uuid,
    pub usage: String,
    pub upload: i32,
}
impl NewInitDownload {
    pub fn insert<T: GenericConnection>(self, conn: &T) -> Result<InitDownload> {
        let stmt = "insert into init_download \
                    (uuid_, usage, upload) \
                    values ($1, $2, $3) \
                    returning id, date_created";
        try_query_to_model!(conn.query(stmt, &[&self.uuid, &self.usage, &self.upload]);
                            InitDownload;
                            id: 0, date_created: 1;
                            uuid: self.uuid, usage: self.usage, upload: self.upload)
    }
}


/// Maps to db table `init_download`
pub struct InitDownload {
    pub id: i32,
    pub uuid: Uuid,
    pub usage: String,
    pub upload: i32,
    pub date_created: DateTime<Utc>,
}
impl FromRow for InitDownload {
    fn table_name() -> &'static str {
        "init_download"
    }

    fn from_row(row: postgres::rows::Row) -> Self {
        Self {
            id:             row.get(0),
            uuid:           row.get(1),
            usage:          row.get(2),
            upload:         row.get(3),
            date_created:   row.get(4),
        }
    }
}
impl InitDownload {
    /// Return the `init_download` record for the given `uuid` or `ErrorKind::DoesNotExist`
    pub fn find<T: GenericConnection>(conn: &T, uuid: &Uuid, usage: DownloadType) -> Result<Self> {
        let stmt = "select * \
                    from init_download \
                    where uuid_ = $1 and usage = $2";
        let usage = usage.as_str();
        try_query_one!(conn.query(stmt, &[uuid, &usage]), InitDownload)
    }

    /// Try deleting the current record from the database, returning the number of items deleted
    pub fn delete<T: GenericConnection>(&self, conn: &T) -> Result<i64> {
        let stmt = "with deleted as (delete from init_download where id = $1 returning 1) \
                    select count(*) from deleted";
        try_query_aggregate!(conn.query(stmt, &[&self.id]), i64)
    }

    /// Try fetching the associated `Upload`
    pub fn get_upload<T: GenericConnection>(&self, conn: &T) -> Result<Upload> {
        let stmt = "select * from upload where id = $1";
        try_query_one!(conn.query(stmt, &[&self.upload]), Upload)
    }

    /// Check if download initializer is still valid
    pub fn still_valid(&self, dt: &DateTime<Utc>) -> bool {
        dt.signed_duration_since(self.date_created) <= Duration::seconds(CONFIG.download_timeout_secs)
    }

    /// Try deleting all `init_download` records that are older than the current `CONFIG.download_timeout_secs`
    pub fn clear_outdated<T: GenericConnection>(conn: &T) -> Result<i64> {
        let stmt = "with deleted as (delete from init_download where date_created < $1 returning 1) \
                    select count(*) from deleted";
        let timeout = Duration::seconds(CONFIG.download_timeout_secs);
        let now = Utc::now();
        let cutoff = now.checked_sub_signed(timeout)
            .ok_or_else(|| format_err!(ErrorKind::InvalidDateTimeMathOffset, "Error subtracting {} secs from {:?}",
                                       CONFIG.download_timeout_secs, now))?;
        try_query_aggregate!(conn.query(stmt, &[&cutoff]), i64)
    }
}


/// For initializing a new `Download` record
pub struct NewDownload {
    pub upload: i32,
}
impl NewDownload {
    pub fn insert<T: GenericConnection>(self, conn: &T) -> Result<Download> {
        let stmt = "insert into download (upload) values ($1) returning id, date_created";
        try_query_to_model!(conn.query(stmt, &[&self.upload]);
                            Download;
                            id: 0, date_created: 1;
                            upload: self.upload)
    }
}


/// Maps to db table `download`
pub struct Download {
    pub id: i32,
    pub upload: i32,
    pub date_created: DateTime<Utc>,
}
impl FromRow for Download {
    fn table_name() -> &'static str {
        "download"
    }

    fn from_row(row: postgres::rows::Row) -> Self {
        Self {
            id:             row.get(0),
            upload:         row.get(1),
            date_created:   row.get(2),
        }
    }
}


/// Maps to db table `status`
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
    /// Fetch or initialize the single `status` table record
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

    /// Check if we can hold `n` more bytes, staying under `CONFIG.max_combined_upload_bytes`
    pub fn can_fit<T: GenericConnection>(conn: &T, n_bytes: i64) -> Result<bool> {
        let status = Self::load(conn)?;
        Ok((status.total_bytes + n_bytes) < CONFIG.max_combined_upload_bytes)
    }

    /// Increment `status` record count and running total of uploaded bytes
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

    /// Decrement `status` record count and running total of uploaded bytes
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

