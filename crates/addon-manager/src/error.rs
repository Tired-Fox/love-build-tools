use crate::Git;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    Url(url::ParseError),

    InvalidHost(String),
    InvalidAddon(String),
    InvalidConfigurationField(String),

    Git(Git, String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Json(err) => write!(f, "json: {err}"),
            Self::Url(err) => write!(f, "url: {err}"),

            Self::InvalidHost(name) => write!(f, "invalid host '{name}': expected 'github.com'"),
            Self::InvalidAddon(message) => write!(f, "invalid addon: {message}"),
            Self::InvalidConfigurationField(field) => write!(f, "invalid configuration field '{field}'"),

            Self::Git(git, message) => match git {
                Git::Clone(url) => write!(f, "failed to clone repository({url}): {message}"),
                Git::Pull => write!(f, "failed to pull repository: {message}"),
                Git::Fetch => write!(f, "failed to fetch repository changes: {message}"),
                Git::Reset => write!(f, "failed to reset repository: {message}"),
                Git::Hash => write!(f, "failed to parse repository hash: {message}"),
            },
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Self::Url(value)
    }
}

impl From<luarc::Error> for Error {
    fn from(value: luarc::Error) -> Self {
        use luarc::Error::*;

        match value {
            Io(io) => Self::Io(io),
            Json(json) => Self::Json(json),
        }
    }
}
