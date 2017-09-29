#![recursion_limit = "1024"]

#[macro_use] extern crate error_chain;
#[macro_use] extern crate clap;
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
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate serde_urlencoded;
#[macro_use] extern crate rouille;

#[macro_use] pub mod macros;
pub mod service;
pub mod sweep;
pub mod handlers;
pub mod db;
pub mod models;
pub mod auth;
pub mod errors;
pub mod admin;

use std::env;
use clap::{Arg, App, SubCommand};

use errors::*;


static APPNAME: &'static str = "Transfer";


fn run() -> Result<()> {
    let matches = App::new(APPNAME)
        .version(crate_version!())
        .about("Secure Transfer Sever")
        .subcommand(SubCommand::with_name("serve")
            .about("Initialize Server")
            .arg(Arg::with_name("port")
                .long("port")
                .short("p")
                .takes_value(true)
                .help("Port to listen on. Defaults to 3000"))
            .arg(Arg::with_name("public")
                .long("public")
                .help("Serve on '0.0.0.0' instead of 'localhost'"))
            .arg(Arg::with_name("debug")
                .long("debug")
                .help("Output debug logging info. Shortcut for setting env-var LOG=debug")))
        .subcommand(SubCommand::with_name("admin")
            .about("admin functions")
            .subcommand(SubCommand::with_name("database")
                .about("database functions")
                .subcommand(SubCommand::with_name("setup")
                    .about("Setup database migration table"))
                .subcommand(SubCommand::with_name("migrate")
                    .about("Look for and apply any available un-applied migrations"))
                .subcommand(SubCommand::with_name("shell")
                    .about("Open a database shell")))
            .subcommand(SubCommand::with_name("sweep-files")
                .about("Sweep up orphaned files that are no longer referenced in the database")))
        .get_matches();

    match matches.subcommand() {
        ("admin", Some(admin_matches)) => {
            admin::handle(&admin_matches)?;
        }
        ("serve", Some(serve_matches)) => {
            env::set_var("LOG", "info");
            let log_debug = serve_matches.is_present("debug");
            if log_debug { env::set_var("LOG", "debug"); }
            let port = serve_matches.value_of("port").unwrap_or("3000").parse::<u16>().chain_err(|| "`--port` expects an integer")?;
            let host = if serve_matches.is_present("public") { "0.0.0.0" } else { "localhost" };
            service::start(&host, port)?;
        }
        _ => {
            eprintln!("{}: see `--help`", APPNAME);
        }
    }
    Ok(())
}

quick_main!(run);

