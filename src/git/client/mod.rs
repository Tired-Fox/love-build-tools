use std::{fmt::Display, path::Path};

use bytes::Bytes;
use reqwest::IntoUrl;

pub mod tree;
pub mod branch;
pub mod content;

use serde_json::json;
use tree::Tree;
use branch::Branch;
use content::Contents;

use crate::Error;

static GITHUB_API: &str = "https://api.github.com";
static GITHUB_CONTENT: &str = "https://raw.githubusercontent.com";
static GITHUB_API_VERSION: &str = "2022-11-28";

static ACCEPT_JSON: &str = "application/vnd.github+json";
static ACCEPT_OBJECT: &str = "application/vnd.github.object+json";
static ACCEPT_RAW: &str = "application/vnd.github.raw+json";
static ACCEPT_HTML: &str = "application/vnd.github.html+json";

#[derive(Debug, Default)]
pub struct Client(String);
impl Client {
    pub fn new(user_agent: impl AsRef<str>) -> Self {
        Self(user_agent.as_ref().to_string())
    }

    fn get(&self, url: impl IntoUrl) -> reqwest::RequestBuilder {
        reqwest::Client::new()
            .get(url)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .header("User-Agent", &self.0)
    }

    pub async fn tree<S: Display>(&self, owner: impl Display, repo: impl Display, branch: impl Display, sha: Option<S>) -> Result<Tree, Error> {
        let mut request = self.get(format!("{}/repos/{owner}/{repo}/git/trees/{branch}?recursive=true", GITHUB_API))
            .header("Accept", ACCEPT_JSON);

        if let Some(sha) = sha {
            request = request.body(serde_json::to_string(&json!({
                "base_tree": sha.to_string(),
                "tree": []
            }))?);
        }

        let response = request
            .send()
            .await?;

        let status = response.status();
        let content = response.bytes().await?;

        if status.is_success() {
            let jd = &mut serde_json::Deserializer::from_slice(&content);
            serde_path_to_error::deserialize(jd).map_err(|e| Error::context("Tree", e))
        } else {
            eprintln!("{}", String::from_utf8_lossy(&content));
            Err(Error::custom("failed to get branch tree"))
        }
    }

    pub async fn branches(&self, owner: impl Display, repo: impl Display) -> Result<Vec<Branch>, Error> {
        let response = self.get(format!("{}/repos/{owner}/{repo}/branches", GITHUB_API))
            .header("Accept", ACCEPT_JSON)
            .send()
            .await?;

        let status = response.status();
        let content = response.bytes().await?;

        if status.is_success() {
            let jd = &mut serde_json::Deserializer::from_slice(&content);
            return serde_path_to_error::deserialize(jd).map_err(|e| Error::context("Branch", e))
        } else {
            eprintln!("{}", String::from_utf8_lossy(&content));
            Err(Error::custom("failed to get list of branches"))
        }
    }

    pub async fn branch(&self, owner: impl Display, repo: impl Display, branch: impl Display) -> Result<Branch, Error> {
        let response = self.get(format!("{}/repos/{owner}/{repo}/branches/{branch}", GITHUB_API))
            .header("Accept", ACCEPT_JSON)
            .send()
            .await?;

        let status = response.status();
        let content = response.bytes().await?;

        if status.is_success() {
            let jd = &mut serde_json::Deserializer::from_slice(&content);
            return serde_path_to_error::deserialize(jd).map_err(|e| Error::context("Branch", e))
        } else {
            eprintln!("{}", String::from_utf8_lossy(&content));
            Err(Error::custom("failed to get branch"))
        }
    }

    pub async fn contents(&self, owner: impl Display, repo: impl Display, path: impl AsRef<Path>) -> Result<Contents, Error> {
        let bytes = self.get(format!("{}/repos/{owner}/{repo}/contents/{}", GITHUB_API, path.as_ref().display()))
            .header("Accept", ACCEPT_OBJECT)
            .send()
            .await?
            .bytes()
            .await?;

        let jd = &mut serde_json::Deserializer::from_slice(&bytes);
        serde_path_to_error::deserialize(jd).map_err(|e| Error::context("Contents", e))
    }

    pub async fn raw_blob(&self, owner: impl Display, repo: impl Display, branch: impl Display, path: impl Display) -> Result<Bytes, Error> {
        Ok(self.get(format!("{}/{owner}/{repo}/refs/heads/{branch}/{path}", GITHUB_CONTENT))
            .send()
            .await?
            .bytes()
            .await
            ?)
    } 
}
