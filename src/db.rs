/*!
Database connection stuff
*/
use std::env;
use std::ops::Deref;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use r2d2;
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use postgres;
use migrant_lib;

use super::errors::*;


/// Postgres r2d2 pool
pub type Pool = r2d2::Pool<PostgresConnectionManager>;


/// Postgres r2d2 pooled connection (wrapped for `rocket`)
pub struct DbConn(pub r2d2::PooledConnection<PostgresConnectionManager>);


impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for DbConn {
    type Target = r2d2::PooledConnection<PostgresConnectionManager>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


/// Try to get the current db connection string
pub fn connect_str() -> Result<String> {
    let dir = env::current_dir().chain_err(|| "Unable to retrieve current working dir")?;
    let config_path = migrant_lib::search_for_config(&dir).chain_err(|| "Unable to find `migrant.toml` config file")?;
    let config = migrant_lib::Config::load_file_only(&config_path).chain_err(|| "Failed loading `migrant.toml`")?;
    Ok(config.connect_string().chain_err(|| "Failed creating a connection string")?)
}


/// Initialize a new r2d2 postgres connection pool
pub fn init_pool() -> Pool {
    let config = r2d2::Config::default();
    let conn_str = connect_str().expect("Failed to build connection string");
    let manager = PostgresConnectionManager::new(conn_str, TlsMode::None)
        .expect("Failed to connect to db");
    r2d2::Pool::new(config, manager)
        .expect("Failed to create db pool")
}


pub fn init_conn() -> Result<postgres::Connection> {
    let conn_str = connect_str()?;
    Ok(postgres::Connection::connect(conn_str, postgres::TlsMode::None)?)
}

