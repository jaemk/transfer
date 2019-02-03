/*!
Service initialization
*/
use std::thread;
use std::io::Write;

use env_logger;
use chrono::Local;
use warp::{self, Filter};
use warp::http::StatusCode;
use num_cpus;
use futures_cpupool::CpuPool;
use futures_fs::FsPool;

use handlers;
use sweep;
use db;
use models;
use error::{self, Result};
use std::net::SocketAddr;


#[derive(Clone)]
pub struct Ctx {
    pub cpu: CpuPool,
    pub db: db::Pool,
    pub fs: FsPool,
}


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
    let mut logger = env_logger::Builder::from_env("LOG");
    logger.format(|buf, record| {
        writeln!(buf, "{} [{}] - [{}] -> {}",
                 Local::now().format("%Y-%m-%d_%H:%M:%S"),
                 record.level(),
                 record.target(),
                 record.args()
        )
    })
        .init();

    // force a config load
    let _ = models::CONFIG;

    // make sure `status` record is initialized
    init_status()?;

    // spawn our cleaning thread
    let _ = thread::spawn(sweep::db_sweeper);

    let cpus = num_cpus::get();
    let db_pool = db::init_pool(cpus as u32);
    let cpu_pool = CpuPool::new(cpus * 2);
    let fs_pool = FsPool::new(cpus);
    let ctx = Ctx { cpu: cpu_pool, db: db_pool, fs: fs_pool };

    let addr = format!("{}:{}", host, port).parse::<SocketAddr>()?;
    route_and_serve(addr, ctx);
    Ok(())
}


fn route_and_serve(addr: SocketAddr, ctx: Ctx) {
    // `/api`
    let api_root = warp::path("api");

    // `/api/upload`
    let api_upload = api_root.and(warp::path("upload"));

    // `/api/download`
    let api_download = api_root.and(warp::path("download"));

    let with_ctx = warp::any().map(move || ctx.clone());
    let with_body_stream = warp::body::content_length_limit(models::CONFIG.upload_limit_bytes as u64)
        .and(warp::body::stream());
    let with_body_limit = warp::body::content_length_limit(1_000_000);

    // `/`
    let index = warp::get2()
        .and(warp::path::end())
        .and(warp::fs::file("assets/main.html"));

    // `/status`
    let status = warp::get2()
        .and(warp::path("status").and(warp::path::end()))
        .map(|| {
            let body = json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")});
            warp::reply::json(&body)
        });

    // `/api/hello`
    let api_hello = warp::get2()
        .and(api_root)
        .and(warp::path("hello"))
        .and(warp::path::end())
        .map(|| warp::reply::json(&json!({"message": "hello!"})));

    // `/api/upload/defaults`
    let api_defaults = warp::get2()
        .and(api_upload)
        .and(warp::path("defaults"))
        .and(warp::path::end())
        .map(handlers::api_upload_defaults);

    let api_upload_init = warp::post2()
        .and(api_upload)
        .and(warp::path("init"))
        .and(warp::path::end())
        .and(with_ctx.clone())
        .and(with_body_limit)
        .and(warp::body::json())
        .and_then(handlers::api_upload_init);

    let api_upload_file = warp::post2()
        .and(api_upload)
        .and(warp::path::end())
        .and(with_ctx.clone())
        .and(warp::query())
        .and(with_body_stream)
        .and_then(handlers::api_upload_file);

    let api_upload_delete = warp::post2()
        .and(api_upload)
        .and(warp::path("delete"))
        .and(warp::path::end())
        .and(with_ctx.clone())
        .and(with_body_limit)
        .and(warp::body::json())
        .and_then(handlers::api_upload_delete);

    let api_download_init = warp::post2()
        .and(api_download)
        .and(warp::path("init"))
        .and(warp::path::end())
        .and(with_ctx.clone())
        .and(with_body_limit)
        .and(warp::body::json())
        .and_then(handlers::api_download_init);

    let api_download_file = warp::post2()
        .and(api_download)
        .and(warp::path::end())
        .and(with_ctx.clone())
        .and(with_body_limit)
        .and(warp::body::json())
        .and_then(handlers::api_download);

    let api_download_confirm = warp::post2()
        .and(api_download)
        .and(warp::path("confirm"))
        .and(warp::path::end())
        .and(with_ctx.clone())
        .and(with_body_limit)
        .and(warp::body::json())
        .and_then(handlers::api_download_confirm);

    // match everything else as a static file
    let static_file = warp::get2()
        .and(warp::fs::dir("assets"));

    let not_found = warp::any()
        .map(|| {
            warp::http::Response::builder()
                .status(404)
                .body("not found")
        });

    let api = index
        .or(status)
        .or(api_hello)
        .or(api_defaults)
        .or(api_upload_init)
        .or(api_upload_file)
        .or(api_upload_delete)
        .or(api_download_init)
        .or(api_download_file)
        .or(api_download_confirm)
        .or(static_file)
        .or(not_found);

    let routes = api
        .with(warp::log("transfer"))
        .recover(handle_error);

    warp::serve(routes)
        .run(addr);
}


fn handle_error(err: warp::Rejection) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    {
        let inner = err.find_cause::<error::Error>();
        if inner.is_some() {
            let inner = inner.unwrap();
            use error::ErrorKind::*;
            type S = StatusCode;
            error!("Handler error: {}", inner);
            return Ok(match inner.kind() {
                BadRequest(ref s) => {
                    // 400
                    let body = json!({"error": s});
                    warp::reply::with_status(warp::reply::json(&body), S::BAD_REQUEST)
                }
               InvalidAuth(ref s) => {
                   // 401
                   let body = json!({"error": s});
                   warp::reply::with_status(warp::reply::json(&body), S::UNAUTHORIZED)
               }
               DoesNotExist(ref s) => {
                   // 404
                   let body = json!({"error": s});
                   warp::reply::with_status(warp::reply::json(&body), S::NOT_FOUND)
               }
                UploadTooLarge(ref s) => {
                    // 413
                    let body = json!({"error": s});
                    warp::reply::with_status(warp::reply::json(&body), S::PAYLOAD_TOO_LARGE)
                }
                OutOfSpace(ref s) => {
                    // 503
                    let body = json!({"error": s});
                    warp::reply::with_status(warp::reply::json(&body), S::SERVICE_UNAVAILABLE)
                }
                _ => {
                    // 500
                    let body = json!({"error": "something went wrong"});
                    warp::reply::with_status(warp::reply::json(&body), S::INTERNAL_SERVER_ERROR)
                },
            })
        }
    }

    error!("Handler error: {:?}", err.cause());
    match err.status() {
        e @ StatusCode::NOT_FOUND | e @ StatusCode::METHOD_NOT_ALLOWED => {
            error!("Not found: {}", e);
            Ok(warp::reply::with_status(warp::reply::json(&json!({"error": "not found"})), StatusCode::NOT_FOUND))
        },
        e @ StatusCode::INTERNAL_SERVER_ERROR => {
            error!("Internal error: {}", e);
            Ok(warp::reply::with_status(warp::reply::json(&json!({"error": "internal error"})), StatusCode::INTERNAL_SERVER_ERROR))
        },
        _ => Err(err),
    }
}



