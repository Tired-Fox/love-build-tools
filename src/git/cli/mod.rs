use std::path::Path;

use crate::Error;

pub struct Cli;
impl Cli {
    pub fn checksum<P: AsRef<Path>>(dir: P) -> Result<String, Error> {
        let result = std::process::Command::new("git") 
            .args(["rev-parse", "--verify", "HEAD"])
            .current_dir(dir)
            .output()?;

        Ok(String::from_utf8_lossy(&result.stdout).to_string())
    }

    pub fn branch_name<P: AsRef<Path>>(dir: P) -> Result<String, Error> {
        let result = std::process::Command::new("git") 
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(dir)
            .output()?;

        Ok(String::from_utf8_lossy(&result.stdout).to_string())
    }

    pub fn clone(dir: impl AsRef<Path>, url: impl AsRef<str>, name: impl AsRef<str>) -> Result<(), Error> {
        let result = std::process::Command::new("git") 
            .args(["clone", url.as_ref(), name.as_ref()])
            .current_dir(dir)
            .output()?;

        if result.status.success() {
            Ok(()) 
        } else {
            Err(Error::custom(String::from_utf8_lossy(&result.stderr)))
        }
    }
}
