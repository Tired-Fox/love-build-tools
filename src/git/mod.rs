use std::{collections::BTreeMap, fs::File, str::FromStr};

use release::Tag;
use reqwest::{IntoUrl, RequestBuilder};

mod release;

pub use release::{Asset, AssetName, AssetType, Author, Release};

use crate::{Progress, SpinnerError, Version, data_dir};

pub struct Client {
    user_agent: String,
}

impl Client {
    const GITHUB_API: &'static str = "https://api.github.com";

    pub fn new(user_agent: impl AsRef<str>) -> Self {
        Self {
            user_agent: user_agent.as_ref().to_string(),
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

        reqwest::Client::new()
            .get(url)
            .header("User-Agent", &self.user_agent)
            .header("X-GITHUB-API-VERSION", "2022-11-28")
            .header("Accept", "application/vnd.github+json")
    }

    pub async fn tags(
        &self,
        owner: impl std::fmt::Display,
        repo: impl std::fmt::Display,
    ) -> anyhow::Result<Vec<Tag>> {
        Ok(self
            .get(
                format!("{}/repos/{owner}/{repo}/tags", Self::GITHUB_API),
                None,
            )
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn release_by_tag(
        &self,
        owner: impl std::fmt::Display,
        repo: impl std::fmt::Display,
        tag: &Tag,
    ) -> anyhow::Result<Release> {
        Ok(self
            .get(
                format!("{}/repos/{owner}/{repo}/releases/tags/{}", Self::GITHUB_API, tag.name),
                None,
            )
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn releases(
        &self,
        owner: impl std::fmt::Display,
        repo: impl std::fmt::Display,
    ) -> anyhow::Result<Vec<Release>> {
        Ok(self
            .get(
                format!("{}/repos/{owner}/{repo}/releases", Self::GITHUB_API),
                None,
            )
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn install_by_tag(
        &self,
        owner: impl std::fmt::Display,
        repo: impl std::fmt::Display,
        tag: &Tag,
        base_name: impl AsRef<str>,
        spinner: &mut Progress,
    ) -> anyhow::Result<()> {
        let release = self.release_by_tag(owner, repo, tag).await?;

        match release.get_platform_asset() {
            Some(asset) => {
                let base = data_dir().join(std::env::consts::OS);
                let mut work_done = false;

                let name = base_name.as_ref();
                let zip_name = asset.name.name.clone();

                let archive_path = base.join(".archive");
                let version_file = format!(".{name}-version");
                let zip_file = archive_path.join(&zip_name);

                if !archive_path.exists() {
                    if let Err(err) = std::fs::create_dir_all(&archive_path) {
                        spinner.fail(format!("failed to create directory {}\n  {err}", archive_path.display()));
                    }
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

                    if let Err(err) = std::fs::write(&zip_file, &content) {
                        spinner.fail(format!("failed to write download to disk\n {err}"))
                    }
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
                    if version == release.tag {
                        if work_done {
                            spinner.success(format!("Installed {} {}", name, release.tag).as_str());
                        }
                        return Ok(());
                    } else {
                        std::fs::write(base.join(&version_file), release.tag.to_string())
                            .log_err_in_spin(spinner, "failed to save installed version")?;
                        }
                } else {
                    std::fs::write(base.join(&version_file), release.tag.to_string())
                        .log_err_in_spin(spinner, "failed to save installed version")?;
                }

                // TODO: Seperate logic for handling zip files
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
                    spinner.success(format!("Installed {} {}", name, release.tag).as_str());
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


