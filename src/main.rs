#![recursion_limit = "1024"]
#[macro_use] extern crate error_chain;
extern crate clap;
extern crate migrant_lib;
extern crate transfer;

use clap::{Arg, ArgMatches, App, SubCommand};
use migrant_lib::Config;
use std::env;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Migrant(migrant_lib::Error);
        Transfer(transfer::errors::Error);
    }
}


quick_main!(run);


fn run() -> Result<()> {
    let matches = App::new(transfer::APPNAME)
        .version(&*transfer::CONFIG.app_version)
        .about("Secure Transfer Sever")
        .subcommand(SubCommand::with_name("serve")
            .about("Initialize Server")
            .arg(Arg::with_name("port")
                .long("port")
                .short("p")
                .takes_value(true)
                .default_value("3002")
                .help("Port to listen on."))
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
            admin(&admin_matches)?;
        }
        ("serve", Some(serve_matches)) => {
            env::set_var("LOG", "info");
            if serve_matches.is_present("debug") { env::set_var("LOG", "debug"); }
            let port = serve_matches.value_of("port")
                .expect("default port should be set by clap")
                .parse::<u16>()
                .chain_err(|| "`--port` expects an integer")?;
            let host = if serve_matches.is_present("public") { "0.0.0.0" } else { "localhost" };
            transfer::service::start(&host, port)?;
        }
        _ => {
            eprintln!("{}: see `--help`", transfer::APPNAME);
        }
    }
    Ok(())
}


pub fn admin(matches: &ArgMatches) -> Result<()> {
    if let Some(db_matches) = matches.subcommand_matches("database") {
        let dir = env::current_dir()?;
        let config_path = match migrant_lib::search_for_config(&dir) {
            None => {
                Config::init_in(&dir)
                    .for_database(Some("postgres"))?
                    .initialize()?;
                match migrant_lib::search_for_config(&dir) {
                    None => bail!("Unable to find `.migrant.toml` even though it was just saved."),
                    Some(p) => p,
                }
            }
            Some(p) => p,
        };

        // don't check database migration table since it may not be setup yet
        let config = Config::load_file_only(&config_path)?;

        if db_matches.is_present("setup") {
            config.setup()?;
            return Ok(())
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
                    if let migrant_lib::Error::MigrationComplete(_) = *err {
                        println!("Database is up-to-date!");
                        return Ok(());
                    }
                }
                // propagate other errors
                let _ = res?;
                return Ok(())
            }
            _ => println!("see `--help`"),
        }
        return Ok(())
    }

    if let Some(_) = matches.subcommand_matches("sweep-files") {
        transfer::admin::sweep_files()?;
        return Ok(())
    }

    println!("See: {} admin --help", transfer::APPNAME);
    Ok(())
}

