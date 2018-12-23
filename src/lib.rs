#![recursion_limit = "1024"]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate hex;
extern crate uuid;
extern crate ring;
extern crate crypto;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate migrant_lib;
extern crate ron;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate serde_urlencoded;
extern crate xdg;
extern crate warp;
extern crate futures;
extern crate futures_cpupool;
extern crate futures_fs;
extern crate num_cpus;
extern crate tokio;
extern crate hyper;

#[macro_use] pub mod macros;
pub mod service;
pub mod sweep;
pub mod handlers;
pub mod db;
pub mod models;
pub mod auth;
pub mod error;
pub mod admin;


use error::{Result};
pub use models::CONFIG;

pub static APPNAME: &'static str = "Transfer";


pub fn config_dir() -> Result<std::path::PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("transfer")?;
    let config_dir = xdg_dirs.create_config_directory("")?;
    Ok(config_dir)
}
