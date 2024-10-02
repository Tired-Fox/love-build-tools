use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: Option<usize>,
}

impl Version {
    pub const fn min_love_version() -> Version {
        Version {
            major: 11,
            minor: 0,
            patch: None,
        }
    }

    pub const fn min_lovr_version() -> Version {
        Version {
            major: 0,
            minor: 15,
            patch: Some(0),
        }
    }

    pub const fn latest_love_version() -> Version {
        Version {
            major: 11,
            minor: 5,
            patch: None,
        }
    }

    pub const fn latest_lovr_version() -> Version {
        Version {
            major: 0,
            minor: 17,
            patch: Some(0),
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}{}",
            self.major,
            self.minor,
            match self.patch {
                Some(v) => format!(".{v}"),
                None => String::new(),
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

        Version::from_str(if value.starts_with("v") {
            &value[1..]
        } else {
            value.as_str()
        })
        .map_err(serde::de::Error::custom)
    }
}

impl FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(3, '.');

        Ok(Self {
            major: parts
                .next()
                .unwrap_or("0")
                .parse::<usize>()
                .map_err(|v| v.to_string())?,
            minor: parts
                .next()
                .unwrap_or("0")
                .parse::<usize>()
                .map_err(|v| v.to_string())?,
            patch: match parts.next() {
                Some(patch) => Some(patch.parse::<usize>().map_err(|e| e.to_string())?),
                None => None,
            },
        })
    }
}
