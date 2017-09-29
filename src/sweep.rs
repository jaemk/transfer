/*!
Databse/filesystem cleanup routines
*/
use std::fs;
use std::path;
use std::thread;
use std::time::Duration;

use postgres;
use uuid::Uuid;

use db;
use models;
use errors::*;


const SWEEP_TIMEOUT_SECS: u64 = 60;


/// Cleanup `init_upload` table, deleting expired items
fn sweep_init_upload(conn: &postgres::Connection) -> Result<i64> {
    models::InitUpload::clear_outdated(conn)
}


/// Cleanup `init_download` table, deleting expire items
fn sweep_init_download(conn: &postgres::Connection) -> Result<i64> {
    models::InitDownload::clear_outdated(conn)
}


/// Cleanup `upload` table, deleting expired items
fn sweep_upload(conn: &postgres::Connection) -> Result<i64> {
    let uploads = models::Upload::select_outdated(conn)?;
    let mut sum = 0;
    for upload in uploads.into_iter() {
        match fs::remove_file(&upload.file_path) {
            Ok(_) => (),
            Err(e) => error!("Error deleting {}, {}, continuing...", upload.file_path, e),
        }
        let id = upload.id;
        match upload.delete(conn) {
            Ok(n) => {
                sum += n;
                models::Status::dec_upload(conn, upload.size)?;
            }
            Err(e) => error!("Error deleting upload with id={}, {}, continuing...", id, e),
        }
    }
    Ok(sum)
}


/// Periodically check/clean the database
pub fn db_sweeper() {
    loop {
        {
            match db::init_conn() {
                Err(e) => error!("Unable to acquire db connection: {}", e),
                Ok(conn) => {
                    match sweep_init_upload(&conn) {
                        Err(e) => error!("InitUpload Sweeper Error: {}", e),
                        Ok(n) => info!("Sweeper cleaned out {} old `init_upload` items", n),
                    };
                    match sweep_init_download(&conn) {
                        Err(e) => error!("InitDownload Sweeper Error: {}", e),
                        Ok(n) => info!("Sweeper cleaned out {} old `init_download` items", n),
                    };
                    match sweep_upload(&conn) {
                        Err(e) => error!("Upload Sweeper Error: {}", e),
                        Ok(n) => info!("Sweeper cleaned out {} old `upload` items", n),
                    };
                }
            }
        }
        thread::sleep(Duration::from_secs(SWEEP_TIMEOUT_SECS));
    }
}


/// Cleanup upload files that've been orphaned, no longer have an associated db record
fn sweep_files(conn: &postgres::Connection, upload_dir: &path::Path) -> Result<u64> {
    use std::ffi::OsStr;
    use std::str::FromStr;
    let mut count = 0;
    for file in fs::read_dir(upload_dir)? {
        let path = file?.path();
        if path.is_dir() { continue; }
        if let Some(file_name) = path.file_name().and_then(OsStr::to_str) {
            if file_name.starts_with(".") { continue; }
            let uuid = Uuid::from_str(file_name)?;
            if ! models::Upload::uuid_exists_available(conn, &uuid)? {
                fs::remove_file(&path)?;
                count += 1;
            }
        }
    }
    Ok(count)
}


/// Cleanup orphaned upload files
pub fn sweep_fs<P: AsRef<path::Path>>(upload_dir: P) -> Result<u64> {
    let conn = db::init_conn()?;
    sweep_files(&conn, upload_dir.as_ref())
}

