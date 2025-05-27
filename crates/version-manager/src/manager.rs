use std::{io::Write, path::{Path, PathBuf}, str::FromStr};

use strum::VariantNames;

use crate::{
    git::{self, release::{Asset, AssetType, Release}, ReleaseRef}, Version
};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Target {
    #[default]
    Latest,
    Tag(Version),
}

impl FromStr for Target {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("latest") {
            Ok(Self::Latest)
        } else {
            Ok(Self::Tag(Version::from_str(s)?))
        }
    }
}

pub struct Manager;
impl Manager {
    pub fn installed() -> anyhow::Result<Vec<String>> {
        let base_path = dirs::data_dir().unwrap().join("love-version-manager");
        Ok(base_path
            .read_dir()?
            .flatten()
            .map(|d| d.path().file_name().unwrap().to_string_lossy().to_string())
            .collect())
    }


    pub fn bundles(target: impl AsRef<str>) -> anyhow::Result<Vec<AssetType>> {
        let base_path = dirs::data_dir().unwrap().join("love-version-manager").join(target.as_ref());
        Ok(base_path
            .read_dir()?
            .flatten()
            .filter_map(|d| {
                let name = d.path().file_name().unwrap().to_string_lossy().to_string();
                if name == "linux.AppImage" {
                    Some(AssetType::Linux)
                } else if name == "android.apk" {
                    Some(AssetType::Android)
                } else {
                    AssetType::from_str(&name).ok()
                }
            })
            .collect::<Vec<_>>())
    }

    pub fn bundle_path(version: impl AsRef<str>, target: AssetType) -> PathBuf {
        dirs::data_dir().unwrap().join("love-version-manager").join(version.as_ref()).join(target.as_ref())
    }

    pub async fn get_release(target: Option<Target>) -> anyhow::Result<Release> {
        let client = git::Client::new("love-build-tools");
        client
            .release_by(
                "Tired-Fox",
                "love-bundles",
                match target {
                    Some(Target::Tag(mut tag)) => {
                        if tag.patch == Some(0) {
                            _ = tag.patch.take();
                        }
                        ReleaseRef::Tag(tag.to_string())
                    }
                    None | Some(Target::Latest) => ReleaseRef::Latest,
                },
            )
            .await
    }

    pub async fn install_release(target: Option<Target>, platforms: Vec<AssetType>) -> anyhow::Result<()> {
        let base_path = dirs::data_dir().unwrap().join("love-version-manager");
        if !base_path.exists() {
            std::fs::create_dir_all(&base_path)?;
        }

        let release = Self::get_release(target).await?;
        let tag_path = base_path.join(&release.tag);
        if !tag_path.exists() {
            std::fs::create_dir_all(&tag_path)?;
        }

        for asset in release.assets.iter().filter(|a| platforms.contains(&a.name.ty)) {
            let bundle = Bundle::from(asset);
            if bundle.exists(&tag_path) { continue; }

            log::info!("[{} {}]", release.tag, bundle.ty);
            match asset.name.ty {
                AssetType::Linux | AssetType::Android => {
                    log::info!("  Downloading {}/{}", release.tag, bundle.name);
                    bundle.download(&tag_path).await?
                },
                _ => {
                    log::info!("Downloading {}/{}", release.tag, bundle.name);
                    bundle.download(&tag_path).await?;
                    log::info!("Unziping {tag}/{} into {tag}/{}", bundle.name, bundle.ty, tag=release.tag);
                    bundle.unzip(&tag_path)?;
                    log::info!("Removing {}/{}", release.tag, bundle.name);
                    bundle.clean(&tag_path)?;
                }
            }
        }

        Ok(())
    }

    pub async fn remove_release(target: Version, platforms: Option<Vec<AssetType>>) -> anyhow::Result<()> {
        let base_path = dirs::data_dir().unwrap().join("love-version-manager");
        let tag_path = base_path.join(target.to_string());

        if !base_path.exists() || !tag_path.exists() { return Ok(()) }

        match platforms {
            Some(platforms) if platforms.len() == (AssetType::VARIANTS.len() - 1) => {
                log::info!("Removing {target}");
                std::fs::remove_dir_all(&tag_path)?;
            },
            None => {
                log::info!("Removing {target}");
                std::fs::remove_dir_all(&tag_path)?;
            },
            Some(platforms) => {
                for platform in platforms.iter() {
                    let ty = platform.as_ref();
                    match platform {
                        AssetType::Linux => if tag_path.join("linux.AppImage").exists() {
                            log::info!("Removing {target}/linux.AppImage");
                            std::fs::remove_file(tag_path.join("linux.AppImage"))?;
                        }
                        AssetType::Android => if tag_path.join("android.apk").exists() {
                            log::info!("Removing {target}/android.apk");
                            std::fs::remove_file(tag_path.join("android.apk"))?;
                        },
                        _ => if tag_path.join(platform.to_string()).exists() {
                            log::info!("Removing {target}/{ty}");
                            std::fs::remove_dir_all(tag_path.join(ty))?;
                        }
                    }
                }

                if tag_path.read_dir()?.count() == 0 {
                    println!("Removing {target}");
                    std::fs::remove_dir_all(&tag_path)?;
                }
            }
        }

        Ok(())
    }
}

pub struct Bundle {
    pub name: String,
    pub ty: AssetType,
    pub download_url: String,
}

impl Bundle {
    pub fn exists(&self, base: impl AsRef<Path>) -> bool {
        match &self.ty {
            AssetType::Linux | AssetType::Android => base.as_ref().join(&self.name).exists(),
            _ => base.as_ref().join(self.ty.as_ref()).exists(),
        }
    }

    pub async fn download(&self, base: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = base.as_ref().join(&self.name);
        if path.exists() { return Ok(()); }

        let mut response = reqwest::Client::new()
            .get(&self.download_url)
            .send()
            .await?;

        let mut options = std::fs::OpenOptions::new();

        options.write(true)
            .truncate(true)
            .create(true);

        #[cfg(target_os="linux")]
        {
            use std::os::unix::fs::OpenOptionsExt;
            if self.ty == AssetType::Linux {
                options.mode(755);
            }
        }

        let mut file = options.open(&path)?;

        while let Some(chunk) = response.chunk().await.transpose() {
            file.write_all(chunk?.to_vec().as_slice())?;
        }

        Ok(())
    }

    pub fn unzip(&self, base: impl AsRef<Path>) -> anyhow::Result<()> {
        let asset_path = base.as_ref().join(&self.name);
        let bundle_path = base.as_ref().join(self.ty.as_ref());

        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(&asset_path)?;

        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => {
                    let p: Vec<_> = path.components().collect();
                    let mut b: Vec<_> = bundle_path.components().collect();
                    b.extend(&p[1..]);
                    b.iter().collect::<PathBuf>()
                },
                None => continue,
            };

            if file.is_dir() {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent)?;
                    }
                }

                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }

        Ok(())
    }

    pub fn clean(&self, base: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = base.as_ref().join(&self.name);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

impl From<&Asset> for Bundle {
    fn from(value: &Asset) -> Self {
        Self {
            name: match &value.name.ty {
                AssetType::Linux => "linux.AppImage".into(),
                AssetType::Android => "android.apk".into(),
                _ => value.name.name.clone()
            },
            ty: value.name.ty.clone(),
            download_url: value.browser_download_url.clone(),
        }
    }
}
