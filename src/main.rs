use clap::Parser;

use llam::{Addon, Error, Manager};

/// Lua Language Addon Manager
/// 
/// Used to install and manage lua language server addons. The idea being that it installs them to a set location
/// then adds a `.luarc.json` file to the current location to expose the addons.
#[derive(Debug, Parser)]
#[command(name = "llam", version, about, long_about = None)]
struct LLAM {
    #[command(subcommand)]
    command: Subcommand,     
}

#[derive(Debug, clap::Subcommand)]
enum Subcommand {
    Add {
        addons: Vec<Addon>
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut manager = Manager::new("examples/blocks", "love-build-tools")?;

    let llam = LLAM::parse();

    match llam.command {
        Subcommand::Add { addons } => {
            manager.add(addons)?;
        }
    }

    Ok(())
}
