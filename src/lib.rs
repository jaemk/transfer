#![recursion_limit = "1024"]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate crypto;
extern crate env_logger;
extern crate hex;
extern crate migrant_lib;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate ring;
extern crate ron;
extern crate serde;
extern crate uuid;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate futures;
extern crate futures_cpupool;
extern crate futures_fs;
extern crate hyper;
extern crate num_cpus;
extern crate serde_urlencoded;
extern crate tokio;
extern crate warp;
extern crate xdg;

#[macro_use]
pub mod macros;
pub mod admin;
pub mod auth;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod service;
pub mod sweep;

use error::Result;
pub use models::CONFIG;

pub static APPNAME: &str = "Transfer";

pub fn config_dir() -> Result<std::path::PathBuf> {
    Ok(std::env::var("CONFIG_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().expect("unable to get current_dir")))
}
