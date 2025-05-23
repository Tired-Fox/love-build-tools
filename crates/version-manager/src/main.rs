use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

use clap::Parser;
use love_version_manager::{DEFAULT_PLATFORM, Manager, Target, Version, git::release::AssetType};

struct CommaSeparated<T>(Vec<T>);
impl<T> CommaSeparated<T> {
    pub fn unwrap(self) -> Vec<T> {
        self.0
    }
}
impl<T> Default for CommaSeparated<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
impl<T: Clone> Clone for CommaSeparated<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: FromStr<Err = String>> FromStr for CommaSeparated<T> {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s
            .split(',')
            .map(T::from_str)
            .collect::<Result<Vec<T>, String>>()?;

        Ok(Self(items))
    }
}
impl<T> Deref for CommaSeparated<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for CommaSeparated<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(clap::Parser)]
#[clap(about = "Love Version Manager")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    #[clap(
        alias = "i",
        about = "Install love platform bundles for the specified version"
    )]
    Install {
        #[arg(help = "Targeted release version [examples: latest, 11.5] [default: latest]")]
        target: Option<Target>,
        #[arg(long, default_value = DEFAULT_PLATFORM, help="[possible: win64,win32,linux,macos,android,web]")]
        platforms: CommaSeparated<AssetType>,
    },
    #[clap(
        alias = "rm",
        about = "Remove love platform bundles for the specified version"
    )]
    Remove {
        #[arg(help = "Targeted release version [examples: latest, 11.5] [default: latest]")]
        target: Version,
        #[arg(long, help = "[possible: win64,win32,linux,macos,android,web]")]
        platforms: Option<CommaSeparated<AssetType>>,
    },
    #[clap(
        alias = "ls",
        about = "List information about love version and platform bundles"
    )]
    List { target: Option<Version> },
    #[clap(
        about = "List the available bundles (assets) from the Tired-Fox/love-bundles repository"
    )]
    Assets {
        #[arg(help = "Targeted release version [examples: latest, 11.5] [default: latest]")]
        target: Option<Target>,
    },
}

async fn process() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::List { target } => {
            if target.is_none() {
                println!("[Installed]");
                for version in Manager::installed()? {
                    let bundles = Manager::bundles(&version)?;
                    if !bundles.is_empty() {
                        println!(
                            "  [{version}] {}",
                            bundles
                                .iter()
                                .map(|b| b.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                }
            } else if let Some(target) = target {
                println!("[{target} Installed Bundles]");
                println!(
                    "  {}",
                    Manager::bundles(target.to_string())?
                        .iter()
                        .map(|b| b.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        Command::Install { target, platforms } => {
            Manager::install_release(target, platforms.unwrap()).await?
        }
        Command::Remove { target, platforms } => {
            Manager::remove_release(target, platforms.map(|v| v.unwrap())).await?
        }
        Command::Assets { target } => {
            let mut release = Manager::get_release(target).await?;
            release.assets.sort_by(|a, b| a.name.ty.cmp(&b.name.ty));

            println!("[{} Assets]", release.tag);
            for asset in release.assets.iter() {
                println!(
                    "  [{}] {: <7} - {}",
                    asset.id, asset.name.ty, asset.browser_download_url
                );
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = process().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
