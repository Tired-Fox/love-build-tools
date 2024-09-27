use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    pub git: Option<String>,
    pub html: Option<String>,
    #[serde(rename="self")]
    pub this: String,
}

#[derive(Debug, Deserialize)]
pub struct Contents {
    #[serde(rename="_links")]
    pub links: Links,
    pub download_url: Option<String>,
    pub git_url: Option<String>,
    pub html_url: Option<String>,
    pub name: Option<String>,
    pub path: String,
    pub sha: String,
    pub size: usize,
    pub submodule_git_url: Option<String>,
    #[serde(rename="type")]
    pub ty: String,
    pub url: String,
}
