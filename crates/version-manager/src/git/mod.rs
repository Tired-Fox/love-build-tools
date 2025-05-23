use std::collections::BTreeMap;

use refs::{Refs, Reference};
use release::Release;
use reqwest::{IntoUrl, RequestBuilder};

pub mod refs;
pub mod release;

pub struct Client {
    token: Option<String>,
    user_agent: String,
}

impl Client {
    const GITHUB_API: &'static str = "https://api.github.com";

    pub fn new(user_agent: impl AsRef<str>) -> Self {
        Self {
            token: None,
            user_agent: user_agent.as_ref().to_string(),
        }
    }

    pub fn with_token(self, token: impl AsRef<str>) -> Self {
        Self {
            token: Some(token.as_ref().to_string()),
            ..self
        }
    }

    fn get(&self, url: impl IntoUrl, params: Option<BTreeMap<String, String>>) -> RequestBuilder {
        let url = if let Some(params) = params {
            format!(
                "{}?{}",
                url.into_url().unwrap(),
                params
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join(","),
            )
        } else {
            url.into_url().unwrap().to_string()
        };

        let mut client = reqwest::Client::new()
            .get(url)
            .header("User-Agent", &self.user_agent)
            .header("X-GITHUB-API-VERSION", "2022-11-28")
            .header("Accept", "application/vnd.github+json");

        if let Some(token) = self.token.as_deref() {
            client = client.header("Authorization", format!("Bearer {token}"));
        }

        client
    }

    pub async fn release_by(
        &self,
        owner: impl std::fmt::Display,
        repo: impl std::fmt::Display,
        by: ReleaseRef,
    ) -> anyhow::Result<Release> {
        let response = self
            .get(
                match &by {
                    ReleaseRef::Latest => {
                        format!("{}/repos/{owner}/{repo}/releases/latest", Self::GITHUB_API)
                    }
                    ReleaseRef::Tag(tag) => format!(
                        "{}/repos/{owner}/{repo}/releases/tags/{tag}",
                        Self::GITHUB_API
                    ),
                    ReleaseRef::Id(id) => {
                        format!("{}/repos/{owner}/{repo}/releases/{id}", Self::GITHUB_API)
                    }
                },
                None,
            )
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("{owner}/{repo} release '{by}' not found"))
        }

        Ok(response.json().await?)
    }

    pub async fn refs(
        &self,
        owner: impl std::fmt::Display,
        repo: impl std::fmt::Display,
        r#ref: Refs,
    ) -> anyhow::Result<Vec<Reference>> {
        Ok(self
            .get(match r#ref {
                Refs::Tags => format!(
                    "{}/repos/{owner}/{repo}/git/matching-refs/tags",
                    Self::GITHUB_API
                ),
                Refs::Heads => format!(
                    "{}/repos/{owner}/{repo}/git/matching-refs/heads",
                    Self::GITHUB_API
                ),
            }, None)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn r#ref(
        &self,
        owner: impl std::fmt::Display,
        repo: impl std::fmt::Display,
        ref_type: Refs,
        r#ref: impl std::fmt::Display,
    ) -> anyhow::Result<Vec<Reference>> {
        Ok(self
            .get(match ref_type {
                Refs::Tags => format!(
                    "{}/repos/{owner}/{repo}/git/matching-refs/tags/{ref}",
                    Self::GITHUB_API
                ),
                Refs::Heads => format!(
                    "{}/repos/{owner}/{repo}/git/matching-refs/heads/{ref}",
                    Self::GITHUB_API
                ),
            }, None)
            .send()
            .await?
            .json()
            .await?)
    }
}

pub enum ReleaseRef {
    Latest,
    Tag(String),
    Id(String),
}

impl std::fmt::Display for ReleaseRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Latest => write!(f, "latest"),
            Self::Tag(tag) => write!(f, "{tag}"),
            Self::Id(id) => write!(f, "{id}"),
        }
    }
}
