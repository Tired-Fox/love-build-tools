use std::{borrow::Cow, collections::BTreeMap, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};

use crate::{Addon, Error, ADDONS_DIR, LOCK_FILE_NAME};

#[derive(Debug, Serialize, Deserialize)]
pub struct LockFile {
    #[serde(skip)]
    pub(crate) path: PathBuf,

    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) addons: BTreeMap<Cow<'static, str>, Addon>
}

impl LockFile {
    pub fn detect(dir: impl AsRef<Path>) -> Result<Self, Error> {
        let dir = dir.as_ref();

        if dir.join(LOCK_FILE_NAME).exists() {
            Self::read(&dir.join(LOCK_FILE_NAME))
        } else {
            Self::new(dir)
        }
    }

    pub fn update(&mut self, addon: &Addon) -> Result<bool, Error> {
        let name = addon.name();
        Ok(if let std::collections::btree_map::Entry::Vacant(e) = self.addons.entry(name.clone()) {
            e.insert(addon.clone());
            true
        } else {
            self.addons.get_mut(&name).unwrap().merge(&addon)
        })
    }

    pub fn set(&mut self, name: Cow<'static, str>, addon: Addon) {
        self.addons.insert(name, addon);
    }

    pub fn contains(&mut self, name: &str) -> bool {
        self.addons.contains_key(name)
    }

    fn write(&self) -> Result<(), Error> {
        Ok(std::fs::write(&self.path, serde_json::to_string_pretty(self)?)?)
    }

    fn read(file: &Path) -> Result<Self, Error> {
        let bytes = std::fs::read(file)?;
        let mut lock: Self = serde_json::from_slice(&bytes)?;

        lock.path = file.to_path_buf();

        Ok(lock)
    }

    fn new(dir: &Path) -> Result<Self, Error> {
        // Attempt to read sha1 from cloned addon repositories
        let mut addons = BTreeMap::default();

        let _addons = dir.join(ADDONS_DIR);
        if _addons.exists() {
            for entry in (std::fs::read_dir(_addons)?).flatten() {
                if entry.path().join(".git").exists()
                    && entry.path().join("config.json").exists()
                {
                    let output = std::process::Command::new("git")
                        .args([ "rev-parse", "--verify", "HEAD" ])
                        .output()?;

                    if output.status.success() {
                        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !sha.is_empty() {
                            let name = entry.path().file_stem().unwrap().to_string_lossy().to_string();
                            addons.insert(
                                name.clone().into(),
                                Addon::cats(name, Some(sha), None)
                            );
                            continue;
                        }
                    }

                    log::error!("checksum couldn't be retrieve for path: {}", entry.path().display());
                    if !output.stderr.is_empty() {
                        log::error!("{}", String::from_utf8_lossy(&output.stderr));
                    }
                } else if entry.path().is_dir() {
                    log::warn!("removing invalid addon: {}", entry.path().display());
                    std::fs::remove_dir_all(entry.path())?;
                } else if entry.path().is_file() {
                    log::warn!("removing invalid addon: {}", entry.path().display());
                    std::fs::remove_file(entry.path())?;
                }
            }
        }

        let lock = Self {
            path: dir.join(LOCK_FILE_NAME),
            addons
        };

        // TODO: Create error instead
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }

        log::debug!("creating lockfile {}", dir.join(LOCK_FILE_NAME).display());
        std::fs::write(dir.join(LOCK_FILE_NAME), serde_json::to_string_pretty(&lock)?)?;

        Ok(lock)
    }
}
