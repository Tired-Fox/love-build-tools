use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use zip::write::SimpleFileOptions;

use crate::config::{Config, Settings};
use crate::{
    config::{Build, Framework, Arch},
    git::Client,
};
use crate::{Progress, SpinnerError, Version};

//      Ensure framework is installed for the specific version and target
//      Copy needed files to build directory
//      (maybe) Decompress
//      Splice executables
//      Apply packaging changes
//          - Set icon
//          - Rename files
//          - etc...
//      Compress

/// Builds the project based on the framework and build settings
pub struct Builder<'conf> {
    root: PathBuf,
    config: &'conf Build,
    framework: &'conf Framework,
    settings: &'conf Settings,
}

impl<'conf> Builder<'conf> {
    pub fn new(framework: &'conf Framework, settings: &'conf Settings, config: &'conf Build) -> Self {
        Self {
            root: std::env::current_dir().unwrap(),
            framework,
            settings,
            config,
        }
    }

    pub async fn bundle(&self, client: &Client) -> anyhow::Result<()> {
        let architectures = if self.settings.targets.is_empty() {
            &[Arch::default()]
        } else {
            self.settings.targets.as_slice()
        };

        for arch in architectures {
            let mut spinner = Progress::new(format!("[{arch}]"));
            let tag = format!("[{}:{arch}]", self.framework);
            let mut fail = false;

            spinner.update(format!("{tag} installing {}", self.framework));
            if self
                .ensure_framework_installed(client, &mut spinner)
                .await
                .ok_or_spin(
                    &mut spinner,
                    format!("[{arch}] failed to install {}", self.framework),
                )
                .is_none()
            {
                fail = true;
            }

            spinner.update(format!("{tag} creating output directory"));
            let target_dir = match self.output_dir(*arch).ok_or_spin(
                &mut spinner,
                format!("[{arch}] failed to create output directory"),
            ) {
                Some(td) => td,
                None => continue,
            };

            spinner.update(format!("{tag} copying dynamic libraries"));
            if self
                .copy_files(*arch, &target_dir)
                .ok_or_spin(
                    &mut spinner,
                    format!("{tag} failed to copy dynamic libraries"),
                )
                .is_none()
            {
                fail = true;
            }

            let dist_name = setup_dist(self.framework, self.settings, self.config)?;

            spinner.update(format!("{tag} compressing source and building executable"));
            if self
                .build_executable(*arch, &target_dir, &dist_name)
                .ok_or_spin(&mut spinner, format!("{tag} failed to build executable"))
                .is_none()
            {
                fail = true;
            }

            if PathBuf::from(&dist_name).exists() {
                std::fs::remove_dir_all(&dist_name)?;
            }

            spinner.update(format!("{tag} packaging the executable and it's libraries"));
            if self
                .package(*arch, &target_dir)
                .ok_or_spin(&mut spinner, format!("{tag} failed to package final build"))
                .is_none()
            {
                fail = true
            }

            if fail {
                spinner.finish_fail(format!("{tag} Build failed").as_str());
            } else {
                spinner.finish_success(format!("{tag} Build finished").as_str());
            }
        }

        Ok(())
    }

    pub async fn ensure_framework_installed(
        &self,
        client: &Client,
        spinner: &mut Progress,
    ) -> anyhow::Result<()> {
        // PERF: Caching / Auth / Parse from html
        let tags = client
            .tags(self.framework.owner(), self.framework.repo())
            .await?;

        if self.settings.version < self.framework.min_version() {
            anyhow::bail!(
                "minimum supported love version is {}",
                self.framework.min_version()
            );
        }

        let tag = match tags.iter().find(|t| {
            if let Ok(version) = Version::from_str(&t.name) {
                version == self.settings.version
            } else {
                false
            }
        }) {
            Some(tag) => tag,
            None => {
                anyhow::bail!(
                    "release version {} for {} was not found",
                    self.settings.version,
                    self.framework
                )
            }
        };

        client.install_by_tag(
            self.framework.owner(),
            self.framework.repo(),
            tag,
            self.framework.to_string(),
            spinner
        ).await
    }

    pub fn output_dir(&self, target: Arch) -> anyhow::Result<PathBuf> {
        let target_dir = self
            .root
            .join("build")
            .join(self.framework.to_string())
            .join(target.to_string());

        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)?;
        }
        std::fs::create_dir_all(&target_dir)?;

        Ok(target_dir)
    }

    pub fn copy_files(&self, target: Arch, dest: &Path) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(self.framework.path(target))?.flatten() {
            std::fs::copy(entry.path(), dest.join(entry.path().file_name().unwrap()))?;
        }

        Ok(())
    }

    pub fn build_executable(&self, target: Arch, dest: &Path, source: impl AsRef<Path>) -> anyhow::Result<()> {
        let source = source.as_ref();
        let exe = dest.join(format!("{}.exe", self.config.name));
        let compressed = format!("{}.{}", self.config.name, self.framework);
        // Build based on target
        match target {
            Arch::Win64 => {
                let mut archive = Archive::new(self.root.join(source), dest.join(&compressed))?;
                archive.add_dir(&self.root.join(source), true)?;
                archive.finish()?;

                std::fs::copy(self.framework.exe(target), &exe)?;

                let mut out = std::fs::OpenOptions::new().append(true).open(&exe)?;
                out.write_all(&std::fs::read(dest.join(&compressed))?)?;
            }
            _ => unimplemented!(),
        }

        self.apply_customizations(target, dest)?;

        Ok(())
    }

    pub fn apply_customizations(&self, target: Arch, _dest: &Path) -> anyhow::Result<()> {
        // TODO: If custom icon then apply that to executable
        match target {
            // Can only manipulate icon when on windows
            Arch::Win64 if std::env::consts::OS == "windows" => {
                // TODO: Use win32 api to update exe ico
                // - https://stackoverflow.com/q/67691200
                // - Image png to ico: https://docs.rs/ico/latest/ico/
                //      - or https://docs.rs/image/latest/image/index.html to allow it to
                //      automatically convert the icon file from more formats
            }
            _ => unimplemented!(),
        }

        Ok(())
    }

    pub fn package(&self, target: Arch, dest: &Path) -> anyhow::Result<()> {
        match target {
            Arch::Win64 => {
                let mut archive =
                    Archive::new(dest, dest.join(format!("{}.zip", self.config.name)))?;
                archive.add_dir(dest, false)?;
                archive.finish()?;
            }
            _ => unimplemented!(),
        }

        Ok(())
    }
}

struct Archive {
    prefix: PathBuf,
    archive: PathBuf,
    writer: zip::ZipWriter<std::fs::File>,
    compression: SimpleFileOptions,
}

impl Archive {
    pub fn new(prefix: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let mut archive = Self {
            prefix: prefix.as_ref().to_path_buf(),
            archive: path.as_ref().to_path_buf(),
            writer: zip::ZipWriter::new(
                std::fs::OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(path)?,
            ),
            compression: SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o755),
        };
        archive.writer.set_flush_on_finish_file(true);
        Ok(archive)
    }

    fn add_file(&mut self, file: &Path) -> anyhow::Result<()> {
        let name = file.strip_prefix(&self.prefix).unwrap();
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .ok_or(anyhow::anyhow!("{name:?} Is a Non UTF-8 Path"))?;

        self.writer.start_file(path_as_string, self.compression)?;
        self.writer.write_all(&std::fs::read(file)?)?;
        Ok(())
    }

    pub fn add_dir(&mut self, dir: &Path, recursive: bool) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(dir)?.flatten() {
            if entry.path() == self.archive {
                continue;
            }

            let path = entry.path();
            let name = path.strip_prefix(&self.prefix).unwrap();
            let path_as_string = name
                .to_str()
                .map(str::to_owned)
                .ok_or(anyhow::anyhow!("{name:?} Is a Non UTF-8 Path"))?;

            // Write file or directory explicitly
            // Some unzip tools unzip files with directory paths correctly, some do not!
            if path.is_file() {
                self.add_file(&path)?;
            } else if !name.as_os_str().is_empty() {
                // Only if not root! Avoids path spec / warning
                // and mapname conversion failed error on unzip
                self.writer
                    .add_directory(path_as_string, self.compression)?;
                if recursive {
                    self.add_dir(&path, recursive)?;
                }
            }
        }
        Ok(())
    }

    pub fn finish(self) -> anyhow::Result<std::fs::File> {
        Ok(self.writer.finish()?)
    }
}

pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>, recursive: bool, exclude: &[PathBuf]) -> std::io::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    if exclude.contains(&from.to_path_buf()) {
        return Ok(())
    }
    
    if from.is_dir() {
        if !to.exists() {
            std::fs::create_dir_all(to)?;
        }

        if recursive {
            // Collect all nested paths and check if they are excluded
            for path in std::fs::read_dir(from)?.flatten() {
                copy(path.path(), to.join(path.file_name()), recursive, exclude)?;
            }
        }
    } else {
        std::fs::copy(from, to)?;
    }

    Ok(())
}

pub fn setup_dist(framework: &Framework, settings: &Settings, config: &Build) -> anyhow::Result<String> {
    let dist_name = format!(".{}", framework);
    let dist = PathBuf::from(&dist_name);
    if !dist.exists() {
        std::fs::create_dir_all(&dist)?;
    }

    let entry = PathBuf::from(settings.entry.as_deref().unwrap_or("src/main.lua"));
    let conf = PathBuf::from("src/conf.lua");

    copy("src", &dist_name, true, &[entry.clone(), conf.clone()])?;

    if conf.exists() {
        std::fs::copy(&conf, format!("{dist_name}/conf.lua"))?;
    } else if let Some(cfg) = config.config.as_ref() {
        let conf = format!("function love.conf({})\n{}end", Config::param(), cfg);
        std::fs::write(dist.join("conf.lua"), conf)?;
    }

    if entry.exists() {
        std::fs::copy(&entry, format!("{dist_name}/main.lua"))?;
    }

    Ok(dist_name)
}
