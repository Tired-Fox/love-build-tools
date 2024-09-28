use std::{borrow::Cow, path::{Path, PathBuf}};

use serde_json::json;

use crate::{git::{Cli, Client}, Addon, Error, LockFile, ADDONS_DIR, LUARC};

#[derive(Debug)]
pub struct Manager {
    base: PathBuf,
    lock_file: LockFile,
    client: Client,
}

impl Manager {
    pub fn new(dir: impl AsRef<Path>, user_agent: impl AsRef<str>) -> Result<Self, Error> {
        let path = dir.as_ref();
        Ok(Self {
            lock_file: LockFile::detect(path)?,
            base: path.to_path_buf(),
            client: Client::new(user_agent)
        })
    }

    pub fn clone_addon(&mut self, name: Cow<'static, str>) -> Result<(), Error> {
        // PERF: Return error or log when addon is not in lock file
        if let Some(addon) = self.lock_file.addons.get(&name) {
            let temp_name = addon.checksum.clone().unwrap_or(uuid::Uuid::now_v7().to_string());
            let from = std::env::temp_dir().join(&temp_name);
            let to = self.base.join(ADDONS_DIR).join(addon.name().as_ref());

            if let Err(err) = Cli::clone(std::env::temp_dir(), addon.clone_url(), &temp_name) {
                if from.exists() {
                    std::fs::remove_dir_all(&from)?;
                }
                return Err(err);
            }

            if to.exists() {
                std::fs::remove_dir_all(&to)?;
            }

            if let Some(parent) = to.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::rename(from, to)?;
        }

        Ok(())
    }

    pub fn add(&mut self, addons: Vec<Addon>) -> Result<(), Error> {
        let addon_path = self.base.join(ADDONS_DIR);
        for addon in addons.iter() {
            let name = addon.name();
            let path = addon_path.join(name.as_ref());

            if !path.exists() || !self.lock_file.contains(name.as_ref()) {
                self.lock_file.update(addon)?;
                self.clone_addon(name.clone())?;
            } else {
                let branch_diff = addon.branch.as_ref().map(|v| Cli::branch_name(&path).map(|n| &n != v).unwrap_or_default()).unwrap_or_default();
                let checksum_diff = addon.checksum.as_ref().map(|v| Cli::checksum(&path).map(|n| &n != v).unwrap_or_default()).unwrap_or_default();
                self.lock_file.update(addon)?;

                if branch_diff || checksum_diff {
                    self.clone_addon(name.clone())?;
                }
            };
        }

        if !self.base.join(LUARC).exists() {
            std::fs::write(self.base.join(LUARC), serde_json::to_string_pretty(&json!({
                "Lua.workspace.userThirdParty": [ self.base.join(ADDONS_DIR) ]
            }))?)?;
        }
        
        Ok(())
    }
}
