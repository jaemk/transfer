#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
#![recursion_limit = "1024"]

#[macro_use] extern crate error_chain;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
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

mod service;
mod handlers;
mod db;
mod models;

use std::env;
use clap::{Arg, App, SubCommand};

mod errors {
    use super::*;
    error_chain! {
        foreign_links {
            LogInit(log::SetLoggerError) #[doc = "Error initializing env_logger"];
            ParseInt(std::num::ParseIntError);
            RocketConfig(rocket::config::ConfigError) #[doc = "Error finalizing rocket config"];
        }
    }
}
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
        .get_matches();

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
