use std::{path::PathBuf, slice::Iter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Blob {
    File,
    Executable,
    Symlink
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Mode {
    Blob(Blob),
    SubDirectory,
    SubModule,
}

fn deserialize_entry_type<'de, D>(deserializer: D) -> Result<Mode, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    // use our visitor to deserialize an `ActualValue`
    let value = String::deserialize(deserializer)?;
    match value.as_str() {
        "100644" => Ok(Mode::Blob(Blob::File)),
        "100755" => Ok(Mode::Blob(Blob::Executable)),
        "120000" => Ok(Mode::Blob(Blob::Symlink)),
        "040000" => Ok(Mode::SubDirectory),
        "160000" => Ok(Mode::SubModule),
        other => Err(serde::de::Error::custom(format!("unknown entry type: {other}")))
    }
}

#[derive(Debug, Deserialize)]
pub struct TreeEntry {
    #[serde(deserialize_with="deserialize_entry_type")]
    pub mode: Mode,
    pub path: PathBuf,
    pub sha: String,
    #[serde(rename="type")]
    pub ty: String,
    pub url: Option<String>
}

#[derive(Debug, Deserialize)]
pub struct Tree {
    pub sha: String,
    pub truncated: bool,
    pub url: String,
    pub tree: Vec<TreeEntry>
}

impl Tree {
    pub fn iter(&self) -> Iter<'_, TreeEntry> {
        self.tree.iter()
    }
}

