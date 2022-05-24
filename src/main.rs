mod audio_clip;
mod commands;
mod config;
mod db;

use audio_clip::AudioClip;
use chrono::prelude::*;
use clap::Parser;
use color_eyre::eyre::Result;
use commands::{Cli, Commands};
use config::Config;
use db::Db;
use std::fs;

fn main() -> Result<()> {
    // config
    let config_file = fs::File::open("./config/config.yaml")?;
    let config_de: Config = serde_yaml::from_reader(&config_file)?;

    // setup
    color_eyre::install()?;
    let args = Cli::parse();
    let db = Db::open(&config_de)?;

    match args.command {
        Commands::Record { name } => {
            let name =
                name.unwrap_or_else(|| Local::now().format(&config_de.formats["date"]).to_string());
            let mut clip = AudioClip::record(name)?;
            db.save(&mut clip)?;

            Ok(())
        }
        Commands::List {} => {
            println!("{:5} | {:30} | {:30}", "Id", "Name", "Date");
            for entry in db.list()? {
                println!(
                    "{:5} | {:30} | {:30}",
                    entry.id,
                    entry.name,
                    entry
                        .date
                        .with_timezone(&Local)
                        .format(&config_de.formats["date"])
                        .to_string()
                )
            }

            Ok(())
        }
        Commands::Play { name } => {
            if let Ok(Some(clip)) = db.load(&name) {
                eprintln!("Play {:?}", name);
                clip.play()?;
            } else {
                eprintln!("No such clip found");
            }

            Ok(())
        }
        Commands::Delete { name } => {
            db.delete(&name)?;
            eprintln!("Delete {:?}", name);

            Ok(())
        }
    }
}
