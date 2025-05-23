use std::{path::{Path, PathBuf}, str::FromStr};

use crate::{Addon, Error, LuaRc};

pub struct Manager {
    path: PathBuf,
    addons: PathBuf,
    luarc: PathBuf,
}

impl Manager {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            addons: PathBuf::from(".addons"),
            luarc: PathBuf::from(".luarc.json"),
        }
    }

    pub fn with_addons(self, addons: impl AsRef<Path>) -> Self {
        Self {
            addons: addons.as_ref().to_path_buf(),
            ..self
        }
    }

    pub fn with_luarc(self, luarc: impl AsRef<Path>) -> Self {
        Self {
            luarc: luarc.as_ref().to_path_buf(),
            ..self
        }
    }
}

impl Manager {
    pub fn init(&self) -> Result<(), Error> {
        let luarc = self.path.join(&self.luarc);

        if !luarc.exists() {
            log::info!("Default .luarc.json file generated");
            LuaRc::default().write(&luarc)?;
        }

        Ok(())
    }

    pub fn add(&self, name: impl AsRef<str>) -> Result<(), Error> {
        let addons = self.path.join(&self.addons);
        let luarc = self.path.join(&self.luarc);

        let mut rc = LuaRc::read_as(&luarc)?
            .ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, ".luarc.json file not found"))?;

        let mut addon = Addon::from_str(name.as_ref())?;

        if addon.exists(&addons) {
            log::info!("Pulling addon's latest version at .addons/{}", &addon.repo);
            addon.pull(&addons)?;
        } else {
            log::info!("Downloading addon to .addons/{}", &addon.repo);
            addon.download(&addons)?;
        }

        if rc.workspace.is_none() {
            rc.workspace = Some(Default::default());
        }

        let workspace = rc.workspace.as_mut().unwrap();

        log::info!("Updating .luarc.json");
        _ = workspace.addons.insert(addon.repo.clone(), addon);
        let path = dunce::canonicalize(&addons)?.display().to_string();
        if !workspace.user_third_party.iter().any(|v| v.as_ref() == ".addons" || v.as_ref() == path) {
            workspace.user_third_party.push(".addons".into());
        }
        rc.write(&luarc)?;

        Ok(())
    }

    pub fn remove(&self, name: impl AsRef<str>) -> Result<(), Error> {
        let addons = self.path.join(&self.addons);
        let luarc = self.path.join(&self.luarc);

        let mut rc = LuaRc::read_as(&luarc)?
            .ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, ".luarc.json file not found"))?;

        let path = addons.join(name.as_ref());

        if rc.workspace.as_ref().map(|v| v.addons.contains_key(name.as_ref())).unwrap_or_default() || path.exists() {
            log::info!("Removing addon '{}'", name.as_ref());
        } else {
            log::info!("No addon named '{}' found", name.as_ref());
            return Ok(());
        }

        if let Some(workspace) = rc.workspace.as_mut() {
            if workspace.addons.remove(name.as_ref()).is_some() {
                rc.write(&luarc)?;
            }
        }

        if addons.join(name.as_ref()).exists() {
            std::fs::remove_dir_all(&path)?;
        }

        Ok(())
    }
}
