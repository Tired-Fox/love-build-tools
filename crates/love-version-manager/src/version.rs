use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: Option<usize>,
    pub prerelease: Option<String>,
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch.unwrap_or_default() == other.patch.unwrap_or_default()
    }
}

impl Version {
    pub const fn min_love_version() -> Version {
        Version {
            major: 11,
            minor: 0,
            patch: None,
            prerelease: None,
        }
    }

    pub const fn min_lovr_version() -> Version {
        Version {
            major: 0,
            minor: 15,
            patch: Some(0),
            prerelease: None,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}{}{}",
            self.major,
            self.minor,
            match self.patch {
                Some(v) => format!(".{v}"),
                None => String::new(),
            },
            match self.prerelease.as_deref() {
                Some(v) => v,
                None => "",
            }
        )
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Version::from_str(&value)
            .map_err(serde::de::Error::custom)
    }
}

impl FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = if let Some(s) = s.strip_prefix("v") {
            s.splitn(3, '.')
        } else {
            s.splitn(3, '.')
        };

        let major = parts
                .next()
                .unwrap_or("0")
                .parse::<usize>()
                .map_err(|v| v.to_string())?;

        let minor = parts
                .next()
                .unwrap_or("0")
                .parse::<usize>()
                .map_err(|v| v.to_string())?;

        let (patch, prerelease) = match parts.next() {
            Some(part) => {
                let patch = part.chars().take_while(char::is_ascii_digit).collect::<String>();
                let prerelease = if patch.len() == part.len() {
                    None
                } else {
                    Some(part[patch.len()..].to_string())
                };
                (
                    Some(patch.parse::<usize>().map_err(|e| e.to_string())?),
                    prerelease
                )
            },
            None => (None, None),
        };

        Ok(Self {
            major,
            minor,
            patch,
            prerelease
        })
    }
}
