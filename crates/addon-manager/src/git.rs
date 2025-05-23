use std::{path::Path, process::{Command, Stdio}};

use crate::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Git {
    Clone(String),
    Pull,
    Hash,
    Reset,
    Fetch,
}

impl Git {
    pub fn run(&self, destination: impl AsRef<Path>) -> Result<String, Error> {
        let mut command = Command::new("git");

        match self {
            Self::Clone(url) => command
                .arg("clone")
                .arg(url)
                .arg(destination.as_ref()),
            Self::Fetch => command
                .args(["fetch", "--prune"])
                .current_dir(&destination),
            Self::Pull => command
                .arg("pull")
                .current_dir(&destination),
            Self::Reset => command
                .args(["reset", "--hard"])
                .current_dir(&destination),
            Self::Hash => command
                .args(["rev-parse", "HEAD"])
                .current_dir(&destination),
        };

        let output = command
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()?;

        if !output.status.success() {
            return Err(Error::Git(self.clone(), String::from_utf8_lossy(&output.stderr).to_string()))
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
