use std::str::FromStr;

use clap::Parser;
use lbt::git::{self, Version};

#[derive(Parser)]
pub struct LBT {
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    Install(Install),
    Build,
    Run,
}

#[derive(Debug, Clone)]
pub enum Package {
    Love { version: Option<Version> },
    Lovr { version: Option<Version> },
}

impl Package {
    pub fn owner(&self) -> &str {
        match self {
            Self::Love { .. } => "love2d",
            Self::Lovr { .. } => "bjornbytes",
        }
    }

    pub fn repo(&self) -> &str {
        match self {
            Self::Love { .. } => "love",
            Self::Lovr { .. } => "lovr",
        }
    }

    pub fn tag(&self) -> String {
        let version = match self {
            Self::Love { version } => version.as_ref(),
            Self::Lovr { version } => version.as_ref(),
        };

        match version {
            Some(version) => format!("{}", version),
            None => String::from("latest"),
        }
    }
}

impl FromStr for Package {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, version) = if s.contains('@') {
            s.split_once('@').unwrap()
        } else {
            (s, "latest")
        };

        let version = match version {
            "latest" => None,
            other => Some(Version::from_str(other)?),
        };

        match name {
            "love" => Ok(Self::Love { version }),
            "lovr" => Ok(Self::Lovr { version }),
            _ => Err("expected love or lovr".to_string()),
        }
    }
}

#[derive(clap::Args)]
//#[group(multiple = false, required = true)]
pub struct Install {
    #[arg(long, short)]
    list: bool,
    /// Install a version of love or lovr
    package: Package,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = git::Client::new("love-build-tools");

    // TODO: Convert from install command to pull from a config
    #[allow(clippy::single_match)]
    match LBT::parse().command {
        Subcommand::Install(Install { list, package }) => {
            match package {
                Package::Love { version } => {
                    let releases = client.releases("love2d", "love").await?;
                    if list {
                        println!("Love Versions");
                        for version in releases.iter().map(|r| format!("{}", r.tag)) {
                            println!(" - {version}");
                        }
                    }

                    let release = match version {
                        Some(version) => {
                            if version < Version::min_love_version() {
                                return Err(anyhow::anyhow!("minimum supported love version is {}", Version::min_love_version()))
                            }

                            match releases.iter().find(|r| r.tag == version) {
                                Some(release) => release,
                                None => return Err(anyhow::anyhow!("release version {version} for love was not found"))
                            }
                        },
                        None => releases.first().unwrap()
                    };

                    release.install("love").await?;
                },
                Package::Lovr { version } => {
                    let releases = client.releases("bjornbytes", "lovr").await?;
                    if list {
                        println!("Lovr Versions");
                        for version in releases.iter().map(|r| format!("{}", r.tag)) {
                            println!(" - {version}");
                        }
                    }

                    let release = match version {
                        Some(version) => {
                            if version < Version::min_lovr_version() {
                                return Err(anyhow::anyhow!("minimum supported love version is {}", Version::min_love_version()))
                            }

                            match releases.iter().find(|r| r.tag == version) {
                                Some(release) => release,
                                None => return Err(anyhow::anyhow!("release version {version} for love was not found"))
                            }
                        },
                        None => releases.first().unwrap()
                    };

                    release.install("lovr").await?;
                }
            }
        }
        _ => {}
    }

    Ok(())
}
