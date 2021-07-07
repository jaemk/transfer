/*!
Database connection stuff
*/
use crate::config_dir;
use migrant_lib;
use postgres;
use r2d2;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

use crate::error::Result;

/// Postgres r2d2 pool
pub type Pool = r2d2::Pool<PostgresConnectionManager>;

/// Postgres r2d2 pooled connection (wrapped for `rocket`)
pub type DbConn = r2d2::PooledConnection<PostgresConnectionManager>;

/// Try to get the current db connection string
pub fn connect_str() -> Result<String> {
    let config_dir_ = config_dir()?;
    let config_path = migrant_lib::search_for_settings_file(&config_dir_)
        .ok_or("Unable to find `Migrant.toml` config file")?;
    let config = migrant_lib::Config::from_settings_file(&config_path)
        .map_err(|_| "Failed loading `Migrant.toml`")?;
    Ok(config
        .connect_string()
        .map_err(|_| "Failed creating a connection string")?)
}

/// Initialize a new r2d2 postgres connection pool
pub fn init_pool(n: u32) -> Pool {
    let conn_str = connect_str().expect("Failed to build connection string");
    let manager =
        PostgresConnectionManager::new(conn_str, TlsMode::None).expect("Failed to connect to db");
    r2d2::Pool::builder()
        .min_idle(Some(n))
        .build(manager)
        .expect("Failed to create db pool")
}

/// Initialize a single `postgres::Connection`
pub fn init_conn() -> Result<postgres::Connection> {
    let conn_str = connect_str()?;
    Ok(postgres::Connection::connect(
        conn_str,
        postgres::TlsMode::None,
    )?)
}
