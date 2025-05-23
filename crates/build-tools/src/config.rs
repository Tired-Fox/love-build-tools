use std::path::{Path, PathBuf};

use love_version_manager::git::release::AssetType;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub name: String,
    pub version: String,

    #[serde(
        default = "Config::default_targets",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub target: Vec<AssetType>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub love_version: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<PathBuf>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: Default::default(),
            version: Default::default(),
            target: Self::default_targets(),
            love_version: Default::default(),
            icon: Default::default(),
            exclude: Default::default(),
        }
    }
}

impl Config {
    pub fn new(name: impl std::fmt::Display, version: impl std::fmt::Display) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            ..Default::default()
        }
    }

    pub fn read(path: impl AsRef<Path>) -> anyhow::Result<Option<Self>> {
        if !path.as_ref().exists() {
            return Ok(None);
        }

        let cfg = std::fs::read_to_string(path)?;
        Ok(Some(toml::from_str(&cfg)?))
    }
    pub fn write(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        std::fs::write(path, toml::to_string(self)?)?;
        Ok(())
    }

    #[inline(always)]
    pub fn default_targets() -> Vec<AssetType> {
        #[cfg(all(target_os = "windows", target_arch = "x86"))]
        {
            Vec::from([AssetType::Win32])
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            Vec::from([AssetType::Win64])
        }
        #[cfg(target_os = "linux")]
        {
            Vec::from([AssetType::Linux])
        }
        #[cfg(target_os = "macos")]
        {
            Vec::from([AssetType::Macos])
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Vec::new()
        }
    }

    #[inline(always)]
    pub fn default_target() -> Option<AssetType> {
        #[cfg(all(target_os = "windows", target_arch = "x86"))]
        {
            Some(AssetType::Win32)
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            Some(AssetType::Win64)
        }
        #[cfg(target_os = "linux")]
        {
            Some(AssetType::Linux)
        }
        #[cfg(target_os = "macos")]
        {
            Some(AssetType::Macos)
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            None
        }
    }
}
