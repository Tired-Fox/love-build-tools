use std::{ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub id: usize,
    pub name: AssetName,
    pub browser_download_url: String,
    pub content_type: String,
    pub created_at: String,
    pub download_count: usize,
    pub node_id: String,
    pub size: usize,
    pub state: String,
    pub updated_at: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Author {
    pub avatar_url: String,
    pub events_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub gravatar_id: String,
    pub html_url: String,
    pub id: usize,
    pub login: String,
    pub node_id: String,
    pub organizations_url: String,
    pub received_events_url: String,
    pub repos_url: String,
    pub site_admin: bool,
    pub starred_url: String,
    pub subscriptions_url: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub id: usize,
    pub draft: bool,
    pub prerelease: bool,
    pub name: String,
    pub assets: Vec<Asset>,
    pub assets_url: String,
    pub author: Author,
    pub created_at: String,
    pub published_at: String,
    pub html_url: String,
    #[serde(rename = "tag_name")]
    pub tag: String,
    pub tarball_url: String,
    pub target_commitish: String,
    pub upload_url: String,
    pub url: String,
    pub zipball_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tag {
    pub name: String,
    pub tarball_url: String,
    pub zipball_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, strum::EnumIs, strum::Display, strum::VariantNames, strum::AsRefStr)]
#[strum(serialize_all="camelCase")]
#[serde(rename_all="camelCase")]
pub enum AssetType {
    Win32,
    Win64,
    Linux,
    Web,
    Android,
    Macos,
    Ios,
    #[serde(skip)]
    Other,
}

impl AssetType {
    pub fn iter<'a>() -> std::slice::Iter<'a, Self> {
        [
            Self::Win32,
            Self::Win64,
            Self::Linux,
            Self::Web,
            Self::Android,
            Self::Macos,
            Self::Ios,
        ].iter()
    }
}

impl FromStr for AssetType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "win32" => Ok(Self::Win32),
            "win64" => Ok(Self::Win64),
            "linux" => Ok(Self::Linux),
            "web" => Ok(Self::Web),
            "macos" => Ok(Self::Macos),
            "android" => Ok(Self::Android),
            "ios" => Ok(Self::Ios),
            _ => Err(format!("unsupported platform '{s}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssetName {
    pub ty: AssetType,
    pub name: String,
}

impl Deref for AssetName {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl<'de> Deserialize<'de> for AssetName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        AssetName::from_str(value.as_str()).map_err(serde::de::Error::custom)
    }
}

impl FromStr for AssetName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let asset_type = if s.contains("ios") {
            AssetType::Ios
        } else if s.ends_with("apk") {
            AssetType::Android
        } else if s.contains("win32") && s.ends_with("zip") {
            AssetType::Win32
        } else if s.contains("win64") && s.ends_with("zip") {
            AssetType::Win64
        } else if s.contains("AppImage") {
            AssetType::Linux
        } else if s.contains("macos") || s.ends_with("app.zip") {
            AssetType::Macos
        } else if s.contains("js-emscript") {
            AssetType::Web
        } else {
            AssetType::Other
        };

        Ok(Self {
            name: s.to_string(),
            ty: asset_type,
        })
    }
}
