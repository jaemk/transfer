/*!
Database connection stuff
*/
use r2d2;
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use postgres;
use migrant_lib;
use {config_dir};

use super::errors::*;


/// Postgres r2d2 pool
pub type Pool = r2d2::Pool<PostgresConnectionManager>;


/// Postgres r2d2 pooled connection (wrapped for `rocket`)
pub type DbConn = r2d2::PooledConnection<PostgresConnectionManager>;


/// Try to get the current db connection string
pub fn connect_str() -> Result<String> {
    let config_dir_ = config_dir()?;
    let config_path = migrant_lib::search_for_settings_file(&config_dir_)
        .chain_err(|| "Unable to find `Migrant.toml` config file")?;
    let config = migrant_lib::Config::from_settings_file(&config_path)
        .chain_err(|| "Failed loading `Migrant.toml`")?;
    Ok(config.connect_string().chain_err(|| "Failed creating a connection string")?)
}


/// Initialize a new r2d2 postgres connection pool
pub fn init_pool() -> Pool {
//    let config = r2d2::Config::default();
    let conn_str = connect_str().expect("Failed to build connection string");
    let manager = PostgresConnectionManager::new(conn_str, TlsMode::None)
        .expect("Failed to connect to db");
    r2d2::Pool::builder()
        .min_idle(Some(3))
        .build(manager)
        .expect("Failed to create db pool")
}


/// Initialize a single `postgres::Connection`
pub fn init_conn() -> Result<postgres::Connection> {
    let conn_str = connect_str()?;
    Ok(postgres::Connection::connect(conn_str, postgres::TlsMode::None)?)
}

