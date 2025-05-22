use std::str::FromStr;

use clap::{ArgAction, Parser};
use love_version_manager::{git::{self, release::AssetType, ReleaseRef}, Version};

#[derive(clap::Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command
}

#[derive(clap::Subcommand)]
enum Command {
    Love {
        target: Target,
        #[arg(action = ArgAction::Append, required=true)]
        platforms: Vec<LovePlatform>
    },
    Lovr {
        target: Target,
        #[arg(action = ArgAction::Append, required=true)]
        platforms: Vec<LovrPlatform>
    },
}

#[derive(Debug, Clone, PartialEq)]
enum Target {
    Latest,
    Tag(Version),
}

impl FromStr for Target {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("latest") {
            Ok(Self::Latest)
        } else {
            Ok(Self::Tag(Version::from_str(s)?))
        }
    }
}

#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
enum LovrPlatform {
    Win64,
    Linux,
    Macos
}

impl PartialEq<AssetType> for LovrPlatform {
    fn eq(&self, other: &AssetType) -> bool {
        match other {
            AssetType::Macos => self == &Self::Macos,
            AssetType::Linux => self == &Self::Linux,
            AssetType::Win64 => self == &Self::Win64,
            _ => false
        }
    }
}

#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
enum LovePlatform {
    Win64,
    Win32,
    Linux,
    Macos,
    Ios,
    Android
}

impl PartialEq<AssetType> for LovePlatform {
    fn eq(&self, other: &AssetType) -> bool {
        match other {
            AssetType::Macos => self == &Self::Macos,
            AssetType::Linux => self == &Self::Linux,
            AssetType::Win64 => self == &Self::Win64,
            AssetType::Win32 => self == &Self::Win32,
            AssetType::Ios => self == &Self::Ios,
            AssetType::Android => self == &Self::Android,
            _ => false
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Love { target, platforms } => {
            let client = git::Client::new("love-build-tools");
            let release = client.release_by("love2d", "love", match target {
                Target::Tag(mut tag) => {
                    if tag.patch == Some(0) {
                        _ = tag.patch.take();
                    }
                    ReleaseRef::Tag(tag.to_string())
                },
                Target::Latest => ReleaseRef::Latest,
            }).await?;

            println!("[Assets]");
            for asset in release.assets.iter().filter(|a| platforms.iter().any(|p| *p == a.name.ty)) {
                println!("  - [{}] \x1b[4m\x1b]8;;{}\x1b\\{} @ {}\x1b]8;;\x1b\\\x1b[24m", asset.id, asset.browser_download_url, asset.name.name, asset.name.ty);
            }
        },
        Command::Lovr { target, platforms } => {
            let client = git::Client::new("love-build-tools");
            let release = client.release_by("bjornbytes", "lovr", match target {
                Target::Tag(mut tag) => {
                    if tag.patch.is_none() {
                        tag.patch = Some(0);
                    }
                    ReleaseRef::Tag(format!("v{tag}"))
                },
                Target::Latest => ReleaseRef::Latest,
            }).await?;

            println!("[Assets]");
            for asset in release.assets.iter().filter(|a| platforms.iter().any(|p| *p == a.name.ty)) {
                println!("  - [{}] \x1b[4m\x1b]8;;{}\x1b\\{} @ {}\x1b]8;;\x1b\\\x1b[24m", asset.id, asset.browser_download_url, asset.name.name, asset.name.ty);
            }
        },
    }

    Ok(())
}
