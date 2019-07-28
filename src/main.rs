#![recursion_limit = "1024"]
#[macro_use]
extern crate clap;
extern crate migrant_lib;
extern crate transfer;

use clap::{App, Arg, ArgMatches, SubCommand};
use migrant_lib::config::PostgresSettingsBuilder;
use migrant_lib::Config;
use std::env;

type Error = Box<std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

pub fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let matches = App::new(transfer::APPNAME)
        .version(crate_version!())
        .about("Secure Transfer Sever")
        .subcommand(
            SubCommand::with_name("serve")
                .about("Initialize Server")
                .arg(
                    Arg::with_name("debug")
                        .long("debug")
                        .help("Output debug logging info. Shortcut for setting env-var LOG=debug"),
                ),
        )
        .subcommand(
            SubCommand::with_name("admin")
                .about("admin functions")
                .subcommand(
                    SubCommand::with_name("config-dir")
                        .about("print the xdg config directory being used"),
                )
                .subcommand(
                    SubCommand::with_name("database")
                        .about("database functions")
                        .subcommand(
                            SubCommand::with_name("setup").about("Setup database migration table"),
                        )
                        .subcommand(
                            SubCommand::with_name("migrate")
                                .about("Look for and apply any available un-applied migrations"),
                        )
                        .subcommand(SubCommand::with_name("shell").about("Open a database shell")),
                )
                .subcommand(SubCommand::with_name("sweep-files").about(
                    "Sweep up orphaned files that are no longer referenced in the database",
                )),
        )
        .get_matches();

    match matches.subcommand() {
        ("admin", Some(admin_matches)) => {
            admin(&admin_matches)?;
        }
        ("serve", Some(serve_matches)) => {
            env::set_var("LOG", "info");
            if serve_matches.is_present("debug") {
                env::set_var("LOG", "debug");
            }
            transfer::service::start()?;
        }
        _ => {
            eprintln!("{}: see `--help`", transfer::APPNAME);
        }
    }
    Ok(())
}

pub fn admin(matches: &ArgMatches) -> Result<()> {
    if let Some(_) = matches.subcommand_matches("config-dir") {
        println!("{}", transfer::config_dir()?.display());
        return Ok(());
    }
    if let Some(db_matches) = matches.subcommand_matches("database") {
        let proj_dir = env::current_dir()?;
        let config_dir = transfer::config_dir()?;
        let config_path = match migrant_lib::search_for_settings_file(&config_dir) {
            None => {
                Config::init_in(&config_dir)
                    .with_postgres_options(
                        PostgresSettingsBuilder::empty()
                            .database_name("transfer")
                            .database_user("transfer")
                            .database_password("transfer")
                            .database_host("localhost")
                            .database_port(5432)
                            .migration_location(proj_dir.join("migrations"))?,
                    )
                    .initialize()?;
                match migrant_lib::search_for_settings_file(&config_dir) {
                    None => Err("Unable to find `Migrant.toml` even though it was just saved.")?,
                    Some(p) => p,
                }
            }
            Some(p) => p,
        };

        // don't check database migration table since it may not be setup yet
        let mut config = Config::from_settings_file(&config_path)?;
        config.use_cli_compatible_tags(true);

        if db_matches.is_present("setup") {
            config.setup()?;
            return Ok(());
        }

        // load applied migrations from the database
        let config = config.reload()?;

        match db_matches.subcommand() {
            ("shell", _) => {
                migrant_lib::shell(&config)?;
            }
            ("migrate", _) => {
                let res = migrant_lib::Migrator::with_config(&config)
                    .direction(migrant_lib::Direction::Up)
                    .all(true)
                    .apply();
                if let Err(ref err) = res {
                    if err.is_migration_complete() {
                        println!("Database is up-to-date!");
                        return Ok(());
                    }
                }
                // propagate other errors
                let _ = res?;
                return Ok(());
            }
            _ => println!("see `--help`"),
        }
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("sweep-files") {
        transfer::admin::sweep_files()?;
        return Ok(());
    }

    println!("See: {} admin --help", transfer::APPNAME);
    Ok(())
}
