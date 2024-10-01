use std::{collections::BTreeMap, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{Version, DATA};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub project: Project,
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    pub build: BTreeMap<Framework, Build>,
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    pub target: BTreeMap<Target, Settings>,
}

impl Config {
    pub fn parse_or_default() -> anyhow::Result<Self> {
        let cd = std::env::current_dir()?;

        if cd.join("lbt.toml").exists() {
            let content = std::fs::read_to_string(cd.join("lbt.toml"))?;
            Ok(toml::from_str(content.as_str())?)
        } else {
            Ok(Self {
                project: Project {
                    name: cd.file_name().unwrap().to_str().unwrap().to_string(),
                    icon: None,
                },
                build: BTreeMap::default(),
                target: BTreeMap::default(),
            })
        }
    }

    pub fn new(name: impl std::fmt::Display) -> Self {
        Self {
            project: Project {
                name: name.to_string(),
                icon: None,
            },
            build: BTreeMap::default(),
            target: BTreeMap::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Project {
    /// Name of the project
    ///
    /// This is used when naming final executables and directories
    pub name: String,
    /// Icon to use when a more specific icon is not specified
    #[serde(skip_serializing_if="Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Framework {
    Love,
    Lovr,
}

impl FromStr for Framework {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "love" => Ok(Self::Love),
            "lovr" => Ok(Self::Lovr),
            other => Err(format!("invalid framework: {other}"))
        } 
    }
}

impl std::fmt::Display for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Love => "love",
            Self::Lovr => "lovr",
        })
    }
}

impl Framework {
    #[inline]
    pub const fn owner(&self) -> &str {
        match self {
            Self::Love => "love2d",
            Self::Lovr => "bjornbytes",
        }
    }

    #[inline]
    pub const fn repo(&self) -> &str {
        match self {
            Self::Love => "love",
            Self::Lovr => "lovr",
        }
    }

    #[inline]
    pub const fn min_version(&self) -> Version {
        match self {
            Self::Love => Version::min_love_version(),
            Self::Lovr => Version::min_lovr_version(),
        }
    }

    #[inline]
    pub const fn latest(&self) -> Version {
        match self {
            Self::Love => Version::latest_love_version(),
            Self::Lovr => Version::latest_lovr_version(),
        }
    }

    #[inline]
    pub fn path(&self, target: Target) -> PathBuf {
        DATA.join(target.to_string()).join(self.to_string())
    }

    #[inline]
    pub fn exe(&self, target: Target) -> PathBuf {
        self.path(target).join(format!("{self}.exe"))
    }

    #[inline]
    pub fn sample(&self) -> &'static str {
        match self {
            Self::Love => indoc::indoc! {r#"
                function love.draw()
                    love.graphics.print("Hello World!", 400, 300)
                end
            "#},
            Self::Lovr => indoc::indoc! {r#"
                function lovr.draw(pass)
                    pass:text("hello world", 0, 1.7, -3, 0.5)
                end
            "#},
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Build {
    /// Version to use of the framework when building
    pub version: Version,
    /// Optional list of targets to build for.
    ///
    /// Defaults to only building for the current OS
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub targets: Vec<Target>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    Win64,
    Macos,
    Linux,
    Ios,
    Android
}

impl Default for Target {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        { Self::Win64 }
        #[cfg(target_os = "macos")]
        { Self::Macos }
        #[cfg(target_os = "linux")]
        { Self::Linux }
        #[cfg(target_os = "ios")]
        { Self::Ios }
        #[cfg(target_os = "android")]
        { Self::Android }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Win64 => "windows",
            Self::Macos => "macos",
            Self::Linux => "linux",
            Self::Ios => "ios",
            Self::Android => "android",
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Settings {
    /// Specific icon to use when building for the specific target (OS)
    #[serde(skip_serializing_if="Option::is_none")]
    pub icon: Option<String>,
}
