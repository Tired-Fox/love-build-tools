use std::{collections::HashSet, fmt::Display, io::Write, path::{Path, PathBuf}, slice::Iter};

use bytes::Bytes;
use clap::Parser;
use indoc::indoc;
use reqwest::{IntoUrl, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use llam::{cache::Addons, git::{self, client::content::{Contents, Links}, Client}, Error};

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
        #[arg()]
        addon: String
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new("love-build-tools");

    let addons = Addons::new(".addons", &client).await?;
    println!("{:#?}", addons);

    let llam = LLAM::parse();

    match llam.command {
        Subcommand::Add { mut addon } => {
            let (addon, version) = if addon.contains("@") {
                let (a, v) = addon.rsplit_once("@").unwrap();
                (a.to_string(), Some(v))
            } else {
                (addon, None)
            };

            println!("Addon: {} Version: {:?}", addon, version);

            //let info_path = branch_cache.join(format!("{addon}.json"));
            //println!("[{addon}:{}:{}]", addons.contains(addon.as_str()), if info_path.exists() {
            //    "exists"
            //} else {
            //    "does not exist"
            //});
            //
            //// TODO: Lock file for versioning with addon's latest commit sha
            //
            //let addon_path = PathBuf::from(".addons").join(&addon);
            //if !addon_path.exists() {
            //    let info = load_json::<Addon>(&info_path)?;
            //
            //    match info.submodule {
            //        Some(submodule) => {
            //            println!("SHA1: {}", submodule.sha);
            //            println!("{:#?}", GitClient::branches("LuaCATS", "love2d").await);
            //            let output = std::process::Command::new("git")
            //                .args([ "clone", submodule.url.as_str(), addon_path.display().to_string().as_str() ])
            //                .output()?;
            //
            //            match version {
            //                None | Some("latest") => {},
            //                Some(sha1) => {
            //                    let output = std::process::Command::new("git")
            //                        .args([ "reset", "--hard", sha1 ])
            //                        .output()?;
            //
            //                    if !output.status.success() {
            //                        eprintln!("ERROR: Failed to clone addon {addon}");
            //                        std::fs::remove_dir_all(&addon_path)?;
            //                    }
            //                },
            //            }
            //
            //            println!("Cloning Results: {}", output.status);
            //        },
            //        None => eprintln!("unknown addon {addon}: cannot find addon repository") 
            //    }
            //}
        }
    }

    Ok(())
}
