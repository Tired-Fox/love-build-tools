use std::{
    path::Path,
    str::FromStr,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{Error, Git};

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Addon {
    pub domain: String,
    pub host: String,
    #[serde(rename = "repository")]
    pub repo: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl Addon {
    pub fn luacats(
        repo: impl std::fmt::Display,
        version: Option<String>,
        hash: Option<String>,
    ) -> Self {
        Self::new("github.com", "LuaCATS", repo, version, hash)
    }

    pub fn new(
        domain: impl std::fmt::Display,
        host: impl std::fmt::Display,
        repo: impl std::fmt::Display,
        version: Option<String>,
        hash: Option<String>,
    ) -> Self {
        Self {
            domain: domain.to_string(),
            host: host.to_string(),
            repo: repo.to_string(),
            version,
            hash,
        }
    }

    pub fn is_luacats(&self) -> bool {
        self.host.as_str() == "LuaCATS"
    }

    /// Check if the addon exists in the base bath
    ///
    /// The base path is where a collection of addons lives, not a single addon
    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        path.as_ref().to_path_buf().join(&self.repo).exists()
    }

    /// Reset the addons repository removing all changes
    pub fn reset(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref().to_path_buf().join(&self.repo);
        Git::Reset.run(&path)?;

        let hash = Git::Hash.run(&path).map(|v| v.trim().to_string())?;
        self.hash = Some(hash);

        Ok(())
    }

    /// Clone the addon and update the hash
    pub fn download(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref().to_path_buf().join(&self.repo);
        Git::Clone(self.to_string()).run(&path)?;

        let hash = Git::Hash.run(&path).map(|v| v.trim().to_string())?;
        self.hash = Some(hash);

        Ok(())
    }

    /// Pull the addon and update the hash
    pub fn pull(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref().to_path_buf().join(&self.repo);
        Git::Pull.run(&path)?;

        let hash = Git::Hash.run(&path).map(|v| v.trim().to_string())?;
        self.hash = Some(hash);

        Ok(())
    }

    /// Fetch latest changes from git without pulling
    pub fn fetch(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path = path.as_ref().to_path_buf().join(&self.repo);
        Git::Fetch.run(path)?;
        Ok(())
    }
}

impl std::fmt::Display for Addon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "https://github.com/{}/{}", self.host, self.repo)?;

        if let Some(version) = self.version.as_deref() {
            write!(f, "#{version}")?;
        }

        Ok(())
    }
}

impl FromStr for Addon {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix("git+") {
            let url = if s.starts_with("http") {
                Url::parse(s)?
            } else {
                Url::parse(&format!("https://{s}"))?
            };
            let host = url
                .host_str()
                .ok_or(Error::InvalidAddon("missing host to repository".into()))?;

            let mut segments = url.path_segments().ok_or(Error::InvalidAddon(
                "missing user/org and repostiory path segments".into(),
            ))?;
            let target = segments
                .next()
                .ok_or(Error::InvalidAddon("missing user/org path segment".into()))?;
            let repo = segments.next().ok_or(Error::InvalidAddon(
                "missing repository path segment".into(),
            ))?;
            Ok(Self::new(
                host,
                target,
                repo,
                url.fragment().map(ToString::to_string),
                None,
            ))
        } else {
            let (repo, digest) = s
                .split_once("#")
                .map(|(r, d)| (r, Some(d.to_string())))
                .unwrap_or((s, None));
            Ok(Self::luacats(repo, digest, None))
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::Addon;

    #[test]
    fn git_source() {
        let src = Addon::from_str("git+github.com/LuaCATS/love2d");
        assert!(src.is_ok());
        let src = src.unwrap();
        assert!(
            src == Addon {
                domain: "github.com".to_string(),
                host: "LuaCATS".to_string(),
                repo: "love2d".to_string(),
                version: None,
                hash: None,
            }
        );
    }

    #[test]
    fn git_source_with_version() {
        let src = Addon::from_str("git+github.com/LuaCATS/love2d#master");
        assert!(src.is_ok());
        let src = src.unwrap();
        assert!(
            src == Addon {
                domain: "github.com".to_string(),
                host: "LuaCATS".to_string(),
                repo: "love2d".to_string(),
                version: Some("master".to_string()),
                hash: None,
            }
        );
    }

    #[test]
    fn luacats_source() {
        let src = Addon::from_str("love2d");
        assert!(src.is_ok());
        let src = src.unwrap();
        assert!(
            src == Addon {
                domain: "github.com".to_string(),
                host: "LuaCATS".to_string(),
                repo: "love2d".to_string(),
                version: None,
                hash: None,
            }
        );
    }

    #[test]
    fn luacats_source_with_version() {
        let src = Addon::from_str("love2d#master");
        assert!(src.is_ok());
        let src = src.unwrap();
        assert!(
            src == Addon {
                domain: "github.com".to_string(),
                host: "LuaCATS".to_string(),
                repo: "love2d".to_string(),
                version: Some("master".to_string()),
                hash: None
            }
        );
    }
}
