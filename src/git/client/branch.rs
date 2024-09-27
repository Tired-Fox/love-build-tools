#[derive(Debug, serde::Deserialize)]
pub struct Commit {
    pub sha: String,
    pub url: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit: Commit,
}
