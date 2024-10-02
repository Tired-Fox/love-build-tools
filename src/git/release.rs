use std::{fs::File, str::FromStr};

use regex::Regex;
use serde::Deserialize;

use crate::{Progress, SpinnerError, Version, DATA};

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub browser_download_url: String,
    pub content_type: String,
    pub created_at: String,
    pub download_count: usize,
    pub id: usize,
    pub name: AssetName,
    pub node_id: String,
    pub size: usize,
    pub state: String,
    pub updated_at: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Author {
    pub avatar_url: String,
    pub events_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub gravatar_id: String,
    pub html_url: String,
    pub id: usize,
    pub login: String,
    pub node_id: String,
    pub organizations_url: String,
    pub received_events_url: String,
    pub repos_url: String,
    pub site_admin: bool,
    pub starred_url: String,
    pub subscriptions_url: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub id: usize,
    pub draft: bool,
    pub prerelease: bool,
    pub name: String,
    pub assets: Vec<Asset>,
    pub assets_url: String,
    pub author: Author,
    pub created_at: String,
    pub published_at: String,
    pub html_url: String,
    #[serde(rename = "tag_name")]
    pub tag: Version,
    pub tarball_url: String,
    pub target_commitish: String,
    pub upload_url: String,
    pub url: String,
    pub zipball_url: String,
}

impl Release {
    pub fn get_platform_asset(&self) -> Option<&Asset> {
        self.assets.iter().find(|v| {
            #[cfg(target_os = "windows")]
            {
                v.name.ty.is_win_64()
            }
            #[cfg(target_os = "macos")]
            {
                v.name.ty.is_macos()
            }
            #[cfg(target_os = "linux")]
            {
                v.name.ty.is_linux()
            }
            #[cfg(target_os = "android")]
            {
                v.name.ty.is_android()
            }
            #[cfg(target_os = "ios")]
            {
                v.name.ty.is_ios()
            }
        })
    }

    pub async fn install(
        &self,
        base_name: impl AsRef<str>,
        spinner: &mut Progress,
    ) -> anyhow::Result<()> {
        match self.get_platform_asset() {
            Some(asset) => {
                let base = DATA.join(std::env::consts::OS);
                let mut work_done = false;

                let name = base_name.as_ref();
                let zip_name = asset.name.name.clone();

                let archive_path = base.join(".archive");
                let version_file = format!(".{name}-version");
                let zip_file = archive_path.join(&zip_name);

                if !archive_path.exists() {
                    std::fs::create_dir_all(&archive_path).log_err_in_spin(
                        spinner,
                        format!("failed to create directory {}", archive_path.display()),
                    )?;
                }

                if !zip_file.exists() {
                    work_done = true;
                    spinner.update(format!(
                        "installing `{}` for {}",
                        base_name.as_ref(),
                        std::env::consts::OS
                    ));
                    let response = reqwest::get(asset.browser_download_url.as_str())
                        .await
                        .log_err_in_spin(spinner, "failed to download release")?;

                    let content = response
                        .bytes()
                        .await
                        .log_err_in_spin(spinner, "failed to read download as bytes")?;

                    std::fs::write(&zip_file, &content)
                        .log_err_in_spin(spinner, "failed to write download to disk")?;
                }

                if base.join(&version_file).exists() {
                    let version = Version::from_str(
                        std::fs::read_to_string(base.join(&version_file))
                            .log_err_in_spin(
                                spinner,
                                format!(
                                    "failed to read file {}",
                                    base.join(&version_file).display()
                                ),
                            )?
                            .trim(),
                    )
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                    if version == self.tag {
                        if work_done {
                            spinner.success(format!("Installed {} {}", name, self.tag).as_str());
                        }
                        return Ok(());
                    } else {
                        std::fs::write(base.join(&version_file), self.tag.to_string())
                            .log_err_in_spin(spinner, "failed to save installed version")?;
                    }
                } else {
                    std::fs::write(base.join(&version_file), self.tag.to_string())
                        .log_err_in_spin(spinner, "failed to save installed version")?;
                }

                if zip_name.ends_with(".zip") {
                    work_done = true;
                    spinner.update(format!(
                        "unzipping `{}` for {}",
                        base_name.as_ref(),
                        std::env::consts::OS
                    ));

                    let zf = File::open(&zip_file)?;
                    let mut archive = zip::ZipArchive::new(&zf)?;
                    let base = base.join(name);
                    if !base.exists() {
                        std::fs::create_dir_all(&base)?;
                    } else {
                        std::fs::remove_dir_all(&base)?;
                        std::fs::create_dir_all(&base)?;
                    }

                    for i in 0..archive.len() {
                        // Get the file at the current index.
                        let mut file = archive.by_index(i)?;
                        // Get the path to extract the file to.
                        let outpath = match file.enclosed_name() {
                            Some(path) => path.to_owned(),
                            None => continue, // Skip to the next file if the path is None.
                        };

                        if file.name().ends_with('/') {
                            std::fs::create_dir_all(base.join(outpath.file_name().unwrap()))
                                .log_err_in_spin(
                                    spinner,
                                    format!(
                                        "failed to create directory {}",
                                        base.join(outpath.file_name().unwrap()).display()
                                    ),
                                )?; // Create the directory.
                        } else {
                            spinner.log(format!(" └ unzipped file {}", outpath.display()));

                            // Create and copy the file contents to the output path.
                            let mut outfile = File::create(base.join(outpath.file_name().unwrap()))
                                .log_err_in_spin(
                                    spinner,
                                    format!(
                                        "failed to create file {}",
                                        base.join(outpath.file_name().unwrap()).display()
                                    ),
                                )?;

                            std::io::copy(&mut file, &mut outfile).log_err_in_spin(
                                spinner,
                                format!("failed to unzip file {}", outpath.display()),
                            )?;
                        }

                        // Set file permissions if running on a Unix-like system.
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;

                            if let Some(mode) = file.unix_mode() {
                                std::fs::set_permissions(
                                    &outpath,
                                    std::fs::Permissions::from_mode(mode),
                                )
                                .ok_or_spin(spinner, "failed to copy file permissions");
                            }
                        }
                    }
                } else if zip_name.ends_with(".AppImage") {
                    work_done = true;
                    std::fs::rename(
                        &zip_file,
                        zip_file.with_file_name(format!("{}.AppImage", name)),
                    )
                    .log_err_in_spin(spinner, "failed to rename AppImage")?;
                }

                if work_done {
                    spinner.success(format!("Installed {} {}", name, self.tag).as_str());
                }
            }
            None => {
                return Err(anyhow::anyhow!(
                    "no download for current target os: {}",
                    std::env::consts::OS
                ))
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, strum::EnumIs)]
pub enum AssetType {
    Android,
    Ios,
    Macos,
    Linux,
    Win64,
    Other,
}

#[derive(Debug, Clone)]
pub struct AssetName {
    name: String,
    ty: AssetType,
}

impl<'de> Deserialize<'de> for AssetName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        AssetName::from_str(value.as_str()).map_err(serde::de::Error::custom)
    }
}

impl FromStr for AssetName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let targeted = Regex::new(r"(love|lovr)-(v?\d+(?:\.\d+)*)[-.](?<os>android|ios|macos|win64|x86_64|apk|app)(?:.apk|.zip|.AppImage)").unwrap();

        let asset_type = match targeted.captures(s) {
            Some(captures) => match captures.name("os").as_ref().map(|v| v.as_str()) {
                Some("android" | "apk") => AssetType::Android,
                Some("ios") => AssetType::Ios,
                Some("macos" | "app") => AssetType::Macos,
                Some("x86_64") => AssetType::Linux,
                Some("win64") => AssetType::Win64,
                Some(other) => return Err(format!("unknown asset os: {other}")),
                _ => return Err("unknown asset os".to_string()),
            },
            None => AssetType::Other,
        };

        Ok(Self {
            name: s.to_string(),
            ty: asset_type,
        })
    }
}
