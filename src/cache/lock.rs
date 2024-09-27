use std::{borrow::Cow, collections::BTreeMap, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};

use crate::{git::client::branch::Branch, Error};

static LOCK_FILE_NAME: &str = ".llam.lock";

#[derive(Debug, Serialize, Deserialize)]
pub struct Lock {
    #[serde(skip)]
    pub(crate) path: PathBuf,

    pub(crate) lls_addon: Cow<'static, str>,
    pub(crate) addons: BTreeMap<Cow<'static, str>, Cow<'static, str>>
}

impl Lock {
    pub async fn detect(dir: impl AsRef<Path>, branch: &Branch) -> Result<Self, Error> {
        let dir = dir.as_ref();

        if dir.join(LOCK_FILE_NAME).exists() {
            Self::read(&dir.join(LOCK_FILE_NAME)).await
        } else {
            Self::new(dir, branch).await
        }
    }

    async fn read(file: &Path) -> Result<Self, Error> {
        let bytes = std::fs::read(file)?;
        let mut lock: Self = serde_json::from_slice(&bytes)?;

        lock.path = file.to_path_buf();

        Ok(lock)
    }

    async fn new(dir: &Path, branch: &Branch) -> Result<Self, Error> {
        // Attempt to read sha1 from cloned addon repositories
        let mut addons = BTreeMap::default();
        if dir.exists() {
            for entry in (std::fs::read_dir(dir)?).flatten() {
                if entry.path().join(".git").exists()
                    && entry.path().join("config.json").exists()
                {
                    let output = std::process::Command::new("git")
                        .args([ "rev-parse", "--verify", "HEAD" ])
                        .output()?;

                    if output.status.success() {
                        let sha = String::from_utf8(output.stdout).unwrap().trim().to_string();
                        if !sha.is_empty() {
                            addons.insert(entry.path().file_stem().unwrap().to_str().unwrap().to_string().into(), sha.into());
                            continue;
                        }
                    }

                    //std::fs::remove_dir(entry.path())?;
                    println!("Remove invalid addon: {}", entry.path().display());
                } else if entry.path().is_dir() {
                    std::fs::remove_dir_all(entry.path())?;
                } else if entry.path().is_file() {
                    std::fs::remove_file(entry.path())?;
                }
            }
        }

        let lock = Self {
            path: dir.join(LOCK_FILE_NAME),

            lls_addon: branch.commit.sha.clone().into(),
            addons
        };

        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }

        std::fs::write(dir.join(LOCK_FILE_NAME), serde_json::to_string_pretty(&lock)?)?;

        Ok(lock)
    }
}
