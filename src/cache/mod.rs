mod lock;
use std::{collections::BTreeMap, path::Path};

use lock::Lock;
use serde::{Deserialize, Serialize};

use crate::{git::{client::tree::Mode, Client}, Error};

static LUA_LS: &str = "LuaLS";
static LLS_ADDONS: &str = "LLS-Addons";
static CACHE_DIR: &str = "lua-language-addon-manager";
static CACHE_FILE_NAME: &str = ".addons.json";

#[derive(Debug, Deserialize, Serialize)]
pub struct Addon {
    description: String,
    #[serde(rename="hasPlugin")]
    plugin: bool,
    submodule: Option<String>,
}

#[derive(Debug)]
pub struct Addons {
    pub(crate) lock: Lock,
    pub(crate) addons: BTreeMap<String, Addon>
}

impl Addons {
    pub async fn new(dir: impl AsRef<Path>, client: &Client) -> Result<Self, Error> {
        let cache_dir = dirs::cache_dir().unwrap().join(CACHE_DIR);
        let branch = client.branch(LUA_LS, LLS_ADDONS, "main").await?;

        let lock = Lock::detect(dir, &branch).await?;

        // If cache is missing or versions are different between
        let addons = if !cache_dir.join(CACHE_FILE_NAME).exists() || branch.commit.sha.as_str() != lock.lls_addon.as_ref() {
            Self::generate(&cache_dir, branch.commit.sha.as_str(), client).await?
        } else {
            let bytes = std::fs::read(cache_dir.join(CACHE_FILE_NAME))?;
            let jd = &mut serde_json::Deserializer::from_slice(&bytes);
            serde_path_to_error::deserialize(jd).map_err(|e| Error::context("Addons::BTreeMap<String, Addon>", e))?
        };

        Ok(Self {
            lock,
            addons
        })
    }

    pub async fn generate(path: &Path, sha: &str, client: &Client) -> Result<BTreeMap<String, Addon>, Error> {
        // TODO: Add progress bars and output information so the user knows that it is grabbing the
        // latest changes
        std::fs::create_dir_all(path)?;

        let mut addons = BTreeMap::new();
        for addon in client.tree(LUA_LS, LLS_ADDONS, "main", Some(sha)).await?.iter().filter(|e| e.path.starts_with("addons/") && e.path.ends_with("info.json")) {
            let name = addon.path.parent().unwrap().strip_prefix("addons/").unwrap().display().to_string();

            match (
                client.raw_blob(LUA_LS, LLS_ADDONS, "main", format!("addons/{name}/info.json")).await,
                client.contents(LUA_LS, LLS_ADDONS, format!("addons/{name}/module")).await
            ) {
                (Ok(addon), contents) => {
                    let jd = &mut serde_json::Deserializer::from_slice(&addon);
                    let mut addon: Addon = serde_path_to_error::deserialize(jd).map_err(|e| Error::context("Addon", e))?;

                    if let Ok(contents) = contents {
                        addon.submodule = contents.submodule_git_url;
                    }

                    addons.insert(name.into(), addon);
                },
                (Err(err), _contents) => {
                    return Err(err) 
                }
            } 
        }

        std::fs::write(path.join(CACHE_FILE_NAME), serde_json::to_string_pretty(&addons)?)?;
        Ok(addons)
    }
}
