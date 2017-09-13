
use std::env;
use std::ops::Deref;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use r2d2;
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use migrant_lib;

pub type Pool = r2d2::Pool<PostgresConnectionManager>;


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


pub fn connect_str() -> String {
    let dir = env::current_dir().expect("Unable to retrieve current working dir");
    let config_path = migrant_lib::search_for_config(&dir).expect("Unable to find `migrant.toml` config file");
    let config = migrant_lib::Config::load_file_only(&config_path).expect("Failed loading `migrant.toml`");
    config.connect_string().expect("Failed creating a connection string")
}


pub fn init_pool() -> Pool {
    let config = r2d2::Config::default();
    let conn_str = connect_str();
    let manager = PostgresConnectionManager::new(conn_str, TlsMode::None)
        .expect("Failed to connect to db");
    r2d2::Pool::new(config, manager)
        .expect("Failed to create db pool")
}
