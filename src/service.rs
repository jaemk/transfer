/*!
Service initialization
*/
use std::env;
use std::thread;

use env_logger;
use chrono::Local;
use rocket;
use rocket::config::{Config, LoggingLevel};

use handlers;
use sweep;
use db;
use models;
use errors::*;


fn init_status() -> Result<()> {
    let conn = db::init_conn()?;
    models::Status::init_load(&conn)?;
    Ok(())
}


pub fn start(host: &str, port: u16, workers: u16, log: bool) -> Result<()> {
    // Set a custom logging format & change the env-var to "LOG"
    // e.g. LOG=info chatbot serve
    env_logger::LogBuilder::new()
        .format(|record| {
            format!("{} [{}] - [{}] -> {}",
                Local::now().format("%Y-%m-%d_%H:%M:%S"),
                record.level(),
                record.location().module_path(),
                record.args()
                )
            })
        .parse(&env::var("LOG").unwrap_or_default())
        .init()?;

    // make sure `status` record is initialized
    init_status()?;

    // spawn our cleaning thread
    let _ = thread::spawn(sweep::db_sweeper);

    info!("** Listening on {} **", host);
    let mut config = Config::production()?;
    config.set_address(host)?;
    config.set_port(port);
    if workers > 0 { config.set_workers(workers); }
    if log { config.set_log_level(LoggingLevel::Normal); }

    rocket::custom(config, log)
        .manage(db::init_pool())
        .mount("/static/",  routes![handlers::static_files])
        .mount("/",
                routes![
                    handlers::index,
                    handlers::api_hello,
                    handlers::api_bye,
                    handlers::api_upload_init,
                    handlers::api_upload_file,
                    handlers::api_upload_delete,
                    handlers::api_download_init,
                    handlers::api_download,
                    handlers::api_download_confirm,
                ]
            )
        .launch();
    Ok(())
}

