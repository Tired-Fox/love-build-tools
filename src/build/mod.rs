use std::path::{Path, PathBuf};
use std::io::Write;

use zip::write::SimpleFileOptions;

use crate::{config::{Build, Config, Framework, Target}, git::Client};

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
    config: &'conf Config,
    framework: &'conf Framework,
    build: &'conf Build
}

impl<'conf> Builder<'conf> {
    pub fn new(framework: &'conf Framework, build: &'conf Build, config: &'conf Config) -> Self {
        Self {
            root: std::env::current_dir().unwrap(),
            framework,
            build,
            config
        }
    }
    
    pub async fn bundle(&self, client: &Client) -> anyhow::Result<()> {
        let targets = if self.build.targets.is_empty() {
            &[Target::default()]
        } else {
            self.build.targets.as_slice()
        };

        for target in targets {
            println!("[{target}]");
            self.ensure_framework_installed(client).await?;
            let target_dir = self.output_dir(*target)?;
            self.copy_files(*target, &target_dir)?;
            self.build_executable(*target, &target_dir)?;
            self.package(*target, &target_dir)?;
        }

        Ok(())
    }

    pub async fn ensure_framework_installed(&self, client: &Client) -> anyhow::Result<()> {
        // PERF: Caching / Auth / Parse from html
        let releases = client.releases(self.framework.owner(), self.framework.repo()).await?;

        if self.build.version < self.framework.min_version() {
            return Err(anyhow::anyhow!("minimum supported love version is {}", self.framework.min_version()))
        }

        let release = match releases.iter().find(|r| r.tag == self.build.version) {
            Some(release) => release,
            None => return Err(anyhow::anyhow!("release version {} for {} was not found", self.build.version, self.framework))
        };

        release.install(self.framework.to_string()).await
    }

    pub fn output_dir(&self, target: Target) -> anyhow::Result<PathBuf> {
        let target_dir = self.root.join("build").join(self.framework.to_string()).join(target.to_string());

        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)?;
        }
        std::fs::create_dir_all(&target_dir)?;

        Ok(target_dir)
    }

    pub fn copy_files(&self, target: Target, dest: &Path) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(self.framework.path(target))?.flatten() {
            if let Some("dll") = entry.path().extension().and_then(|v| v.to_str()) {
                std::fs::copy(entry.path(), dest.join(entry.path().file_name().unwrap()))?;
            }
        }
        Ok(())
    }

    pub fn build_executable(&self, target: Target, dest: &Path) -> anyhow::Result<()> {
        let exe = dest.join(format!("{}.exe", self.config.project.name));
        let compressed = format!("{}.{}", self.config.project.name, self.framework);
        // Build based on target
        match target {
            Target::Win64 => {
                // TODO:
                //   1. Compress lua-files and assets

                let mut archive = Archive::new(self.root.join("src"), dest.join(&compressed))?;
                archive.add_dir(&self.root.join("src"), true)?;
                archive.finish()?;

                //   2. Append compress file bytes to end of love/lovr.exe
                std::fs::copy(self.framework.exe(target), &exe)?;

                let mut out = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&exe)?;
                out.write_all(&std::fs::read(dest.join(&compressed))?)?;

                // PERF: Should the compressed file be removed?
                std::fs::remove_file(dest.join(&compressed))?;
            },
            _ => unimplemented!()
        }

        self.apply_customizations(target, dest)?;

        Ok(())
    }

    pub fn apply_customizations(&self, target: Target, dest: &Path) -> anyhow::Result<()> {
        // TODO: If custom icon then apply that to executable
        match target {
            Target::Win64 => {
                // TODO: Use win32 api to update exe ico
                // - https://stackoverflow.com/q/67691200
                // - Image png to ico: https://docs.rs/ico/latest/ico/
                //      - or https://docs.rs/image/latest/image/index.html to allow it to
                //      automatically convert the icon file from more formats
            }
            _ => unimplemented!()
        }

        Ok(())
    }

    pub fn package(&self, target: Target, dest: &Path) -> anyhow::Result<()> {
        match target {
            Target::Win64 => {
                let mut archive = Archive::new(dest, dest.join(format!("{}.zip", self.config.project.name)))?;
                archive.add_dir(dest, false)?;
                archive.finish()?;
            },
            _ => unimplemented!()
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
            writer: zip::ZipWriter::new(std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(path)?),
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

        println!("adding file {file:?} as {name:?} ...");
        self.writer.start_file(path_as_string, self.compression)?;
        self.writer.write_all(&std::fs::read(file)?)?;
        Ok(())
    }

    pub fn add_dir(&mut self, dir: &Path, recursive: bool) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(dir)?.flatten() {
            if entry.path() == self.archive {
                continue
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
                println!("adding dir {path_as_string:?} as {name:?} ...");
                self.writer.add_directory(path_as_string, self.compression)?;
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
