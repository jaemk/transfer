/*!
Service initialization
*/
use std::env;
use std::thread;
use std::io;

use env_logger;
use chrono::Local;
use rouille;

use handlers::{self, ToResponse};
use sweep;
use db;
use models;
use errors::*;


/// Initialize the `status` database table if it doesn't already exist
fn init_status() -> Result<()> {
    let conn = db::init_conn()?;
    models::Status::init_load(&conn)?;
    Ok(())
}


/// Initialize things
/// - env logger
/// - database `status` table
/// - database connection pool
/// - cleaning thread
/// - server
/// - handle errors
pub fn start(host: &str, port: u16) -> Result<()> {
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

    // force a config load
    let _ = models::CONFIG;

    // make sure `status` record is initialized
    init_status()?;

    // spawn our cleaning thread
    let _ = thread::spawn(sweep::db_sweeper);

    let db_pool = db::init_pool();

    let addr = format!("{}:{}", host, port);
    info!("** Listening on {} **", addr);

    rouille::start_server(&addr, move |request| {
        let db_pool = db_pool.clone();

        rouille::log(request, io::stdout(), move || {
            { // static files
                let static_resp = rouille::match_assets(&request, "static");
                if static_resp.is_success() {
                    return static_resp;
                }
            }
            // dispatch and handle errors
            match route_request(request, db_pool) {
                Ok(resp) => resp,
                Err(e) => {
                    use ErrorKind::*;
                    error!("Handler Error: {}", e);
                    match *e {
                        BadRequest(ref s) => {
                            // bad request
                            let body = json!({"error": s});
                            body.to_resp().unwrap().with_status_code(400)
                        }
                        InvalidAuth(ref s) => {
                            // unauthorized
                            let body = json!({"error": s});
                            body.to_resp().unwrap().with_status_code(401)
                        }
                        DoesNotExist(ref s) => {
                            // not found
                            let body = json!({"error": s});
                            body.to_resp().unwrap().with_status_code(404)
                        }
                        UploadTooLarge(ref s) => {
                            // payload too large / request entity to large
                            let body = json!({"error": s});
                            body.to_resp().unwrap().with_status_code(413)
                        }
                        OutOfSpace(ref s) => {
                            // service unavailable
                            let body = json!({"error": s});
                            body.to_resp().unwrap().with_status_code(503)
                        }
                        _ => rouille::Response::text("Something went wrong").with_status_code(500),
                    }
                }
            }
        })
    });
}


/// Route the request to appropriate handler
fn route_request(request: &rouille::Request, db_pool: db::Pool) -> Result<rouille::Response> {
    Ok(router!(request,
        (GET) (/) => {
            rouille::Response::html("<html><body> <p> hello </p> <script src=\"/assets/js/app.js\"></script></body></html>")
        },

        (GET)   (/api/hello)    => { json!({"message": "hey!"}).to_resp()? },
        (POST)  (/api/bye)      => { json!({"message": "bye!"}).to_resp()? },

        (GET)   (/api/upload/defaults)  => { handlers::api_upload_defaults(request)? },
        (POST)  (/api/upload/init)      => { handlers::api_upload_init(request, db_pool.get()?)? },
        (POST)  (/api/upload)           => { handlers::api_upload_file(request, db_pool.get()?)? },
        (POST)  (/api/upload/delete)    => { handlers::api_upload_delete(request, db_pool.get()?)? },

        (POST)  (/api/download/init)    => { handlers::api_download_init(request, db_pool.get()?)? },
        (POST)  (/api/download)         => { handlers::api_download(request, db_pool.get()?)? },
        (POST)  (/api/download/confirm) => { handlers::api_download_confirm(request, db_pool.get()?)? },

        _ => bail_fmt!(ErrorKind::DoesNotExist, "nothing here")
    ))
}

