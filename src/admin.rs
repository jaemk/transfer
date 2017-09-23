/*!
General Admin commands
*/
use std::env;

use clap::ArgMatches;
use migrant_lib::{self, Config};

use sweep;
use errors::*;


fn sweep_files() -> Result<()> {
    let dir = env::current_dir()?;
    let upload_dir = dir.join("uploads");
    if upload_dir.is_dir() && upload_dir.exists() {
        let n = sweep::sweep_fs(&upload_dir)?;
        info!("** Cleaned up {} orphaned files **", n);
    } else {
        bail!("Provided upload dir is invalid: {:?}", upload_dir);
    }
    Ok(())
}


pub fn handle(matches: &ArgMatches) -> Result<()> {
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
                let _ = res?;
                return Ok(())
            }
            _ => println!("see `--help`"),
        }

        return Ok(())
    }

    if let Some(_) = matches.subcommand_matches("sweep-files") {
        sweep_files()?;
        return Ok(())
    }

    println!("See: {} admin --help", super::APPNAME);
    Ok(())
}

