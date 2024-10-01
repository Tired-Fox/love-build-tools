use std::collections::BTreeMap;

use reqwest::{IntoUrl, RequestBuilder};

mod release;

pub use release::{ Release, Author, Asset, AssetName, AssetType };

pub struct Client {
    user_agent: String
}

impl Client {
    const GITHUB_API: &'static str = "https://api.github.com";

    pub fn new(user_agent: impl AsRef<str>) -> Self {
        Self {
            user_agent: user_agent.as_ref().to_string()
        }
    }

    fn get(&self, url: impl IntoUrl, params: Option<BTreeMap<String, String>>) -> RequestBuilder {
        let url = if let Some(params) = params {
            format!(
                "{}?{}",
                url.into_url().unwrap(),
                params.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(","),
            )
        } else {
            url.into_url().unwrap().to_string()
        };

        reqwest::Client::new()
            .get(url)
            .header("User-Agent", &self.user_agent)
            .header("X-GITHUB-API-VERSION", "2022-11-28")
            .header("Accept", "application/vnd.github+json")
    }

    pub async fn releases(&self, owner: impl std::fmt::Display, repo: impl std::fmt::Display) -> anyhow::Result<Vec<Release>> {
        Ok(self.get(format!("{}/repos/{owner}/{repo}/releases", Self::GITHUB_API), None)
            .send()
            .await?
            .json()
            .await?)
    }
}
