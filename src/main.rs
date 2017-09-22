#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
#![recursion_limit = "1024"]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate hex;
extern crate uuid;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate migrant_lib;
extern crate ring;
extern crate crypto;

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
            .arg(Arg::with_name("log")
                .long("log")
                .help("Output logging info. Shortcut for setting env-var LOG=info"))
            .arg(Arg::with_name("debug")
                .long("debug")
                .help("Output debug logging info. Shortcut for setting env-var LOG=debug"))
            .arg(Arg::with_name("workers")
                .long("workers")
                .short("w")
                .takes_value(true)
                .help("Number of workers to use")))
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

    if let Some(admin_matches) = matches.subcommand_matches("admin") {
        admin::handle(&admin_matches)?;
        return Ok(())
    }

    if let Some(serve_matches) = matches.subcommand_matches("serve") {
        let log_info = serve_matches.is_present("log");
        let log_debug = serve_matches.is_present("debug");
        if log_info { env::set_var("LOG", "info"); }
        if log_debug { env::set_var("LOG", "debug"); }
        let log = if log_info || log_debug { true } else { false };
        let port = serve_matches.value_of("port").unwrap_or("3000").parse::<u16>().chain_err(|| "`--port` expects an integer")?;
        let host = if serve_matches.is_present("public") { "0.0.0.0" } else { "localhost" };
        let workers = serve_matches.value_of("workers").unwrap_or("0").parse::<u16>().chain_err(|| "`--workers` expects an integer")?;
        service::start(&host, port, workers, log)?;
        return Ok(());
    }

    eprintln!("{}: see `--help`", APPNAME);
    Ok(())
}

quick_main!(run);
