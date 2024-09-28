mod error;
mod lock;
mod manager;

pub mod git;

use std::{borrow::Cow, str::FromStr};

pub use error::Error;
pub use lock::LockFile;
pub use manager::Manager;

use reqwest::Url;
use serde::{Deserialize, Serialize};

static ADDONS_DIR: &str = ".addons";
static LUARC: &str = ".luarc.json";
static LOCK_FILE_NAME: &str = ".llam.lock";
static LUA_LS: &str = "LuaLS";
static LLS_ADDONS: &str = "LLS-Addons";

#[derive(Default, Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    #[default]
    LuaCats,
    Github,
}

impl FromStr for Target {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("https://") {
            // TODO: Convert the error
            let url = Url::parse(s).unwrap();
            match url.host_str() {
                Some("github.com") => Ok(Target::Github),
                Some(other) => Err(Error::custom(format!("unsupported addon source: {other}"))),
                _ => Err(Error::custom(format!("unsupported addon source: {s}"))),
            }
        } else {
            Ok(Target::LuaCats)
        }
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Addon {
    pub src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub target: Target,
}

impl Addon {
    pub fn cats(name: String, checksum: Option<String>, branch: Option<String>) -> Self {
        Self {
            src: name,
            checksum,
            branch,
            target: Target::LuaCats,
        }
    }

    pub fn name(&self) -> Cow<'static, str> {
        match self.target {
            Target::LuaCats => self.src.clone().into(),
            Target::Github => {
                let url = Url::parse(self.src.as_str()).unwrap();
                url.path_segments()
                    .unwrap()
                    .nth(1)
                    .unwrap()
                    .to_string()
                    .into()
            }
        }
    }

    pub fn clone_url(&self) -> String {
        match self.target {
            Target::LuaCats => format!("https://github.com/LuaCATS/{}.git", self.src),
            Target::Github => self.src.to_string()
        } 
    }

    pub fn merge(&mut self, other: &Self) -> bool {
        let mut diff = self.src != other.src || self.target != other.target;

        self.src = other.src.clone();
        self.target = other.target;

        if let Some(branch) = other.branch.as_ref() {
            self.branch = Some(branch.to_string());
            diff = true;
        }

        if let Some(checksum) = other.checksum.as_ref() {
            self.checksum = Some(checksum.to_string());
            diff = true;
        }

        diff
    }
}

impl From<&str> for Addon {
    fn from(s: &str) -> Self {
        let mut target = s;
        let mut checksum = None;

        if target.contains('@') {
            let (f, s) = target.split_once('@').unwrap();
            target = f;
            checksum = Some(s.to_string());
        }

        Self {
            target: Target::from_str(target).unwrap(),
            src: target.to_string(),
            checksum,
            branch: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_basic_source() {
        let source = Addon::from("love2d");
        assert_eq!(
            source,
            Addon {
                src: "love2d".to_string(),
                ..Default::default()
            }
        );

        let source = Addon::from("https://github.com/LuaCATS/love2d");
        assert_eq!(
            source,
            Addon {
                src: "https://github.com/LuaCATS/love2d".to_string(),
                target: Target::Github,
                ..Default::default()
            }
        );
    }

    #[test]
    #[should_panic]
    fn parse_fail() {
        let _ = Addon::from("https://example.com/LuaCATS/love2d@1234");
    }

    #[test]
    fn parse_checksum() {
        let source = Addon::from("love2d@1234");
        assert_eq!(
            source,
            Addon {
                src: "love2d".to_string(),
                checksum: Some("1234".to_string()),
                ..Default::default()
            }
        );

        let source = Addon::from("https://github.com/LuaCATS/love2d@1234567678");
        assert_eq!(
            source,
            Addon {
                src: "https://github.com/LuaCATS/love2d".to_string(),
                checksum: Some("1234567678".to_string()),
                target: Target::Github,
                ..Default::default()
            }
        );
    }
}
