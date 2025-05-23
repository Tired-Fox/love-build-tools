use std::collections::BTreeMap;

use clap::{ArgAction, Parser, Subcommand};
use llam::{Addon, Error, Manager};
use log::LevelFilter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(long, short, action = ArgAction::SetTrue)]
    global: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    Add {
        name: String,
    },
    #[clap(alias = "rm")]
    Remove {
        name: String,
    },
    Init,
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct Workspace {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    addons: BTreeMap<String, Addon>,
}

fn process(cli: Cli) -> Result<(), Error> {
    let manager = Manager::new(std::env::current_dir()?);

    match cli.command {
        Command::Init => manager.init()?,
        Command::Add { name } => manager.add(name)?,
        Command::Remove { name } => manager.remove(name)?,
    }

    Ok(())
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .format_timestamp(None)
        .format_module_path(false)
        .format_source_path(false)
        .format_target(false)
        .init();

    let cli = Cli::parse();

    if let Err(err) = process(cli) {
        log::error!("{err}");
    }

    std::process::exit(1);
}
