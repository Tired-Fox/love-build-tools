use std::path::{Path, PathBuf};

use clap::Parser;
use lbt::{
    build::{setup_dist, Builder},
    config::{Arch, Build, Config, Framework, Settings},
    git, Version,
};
use notify::Watcher;

#[derive(Parser)]
pub struct LBT {
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Build the project. `--target` is required if more than one is defined in the config.
    Build {
        /// Specify the targeted framework
        #[arg(long)]
        target: Option<Framework>,
    },
    /// Run the project. `--target` is required if more than one is defined in the config.
    Run {
        /// Specify the targeted framework
        #[arg(long)]
        target: Option<Framework>,
        #[arg(long, short)]
        watch: bool,
    },
    /// Initialize the current directory to be a love-build-tools project.
    Init {
        /// Framework the project should target by default
        framework: Framework,
        /// Version of the framework to use. Defaults to the latest version
        version: Option<Version>,
    },
    /// Create a new love-build-tools project in a new directory.
    New {
        /// Name of the new project
        name: String,
        /// Framework the project should target by default
        framework: Framework,
        /// Version of the framework to use. Defaults to the latest version
        version: Option<Version>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = git::Client::new("love-build-tools");

    let mut config = Build::parse_or_default()?;

    //// TODO: Convert from install command to pull from a config

    //#[allow(clippy::single_match)]
    match LBT::parse().command {
        Subcommand::Build { target } => {
            let target = match target.as_ref() {
                None => if config.targets.len() > 1 { 
                    anyhow::bail!("more than one target possible. use `--target` to specific which one to build")
                } else if !config.targets.is_empty() {
                    config.targets.first_key_value()
                } else {
                    anyhow::bail!("no targets defined in configuration")
                },
                Some(target) => config.targets.get(target).map(|s| (target, s))
            };

            if let Some((framework, settings)) = target {
                Builder::new(framework, settings, &config)
                    .bundle(&client)
                    .await?;
            }
        }
        Subcommand::Run { target, watch } => {
            let arch = Arch::default();

            let target = match target.as_ref() {
                None => if config.targets.len() > 1 { 
                    anyhow::bail!("more than one target possible. use `--target` to specific which one to run")
                } else if !config.targets.is_empty() {
                    config.targets.first_key_value()
                } else {
                    anyhow::bail!("no targets defined in configuration")
                },
                Some(target) => config.targets.get(target).map(|s| (target, s))
            };

            if let Some((framework, settings)) = target {
                // TODO: File watcher and copy files to dist
                // restart exe if file updates in `.dist`
                let exe = framework.exe(arch);

                if let Some(cfg) = config.config.as_mut() {
                    cfg.console = true;
                }

                let dist_name = setup_dist(framework, settings, &config)?;
                let full_dist_path = std::env::current_dir()
                    .unwrap()
                    .join(&dist_name)
                    .display()
                    .to_string();

                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<notify::Result<notify::Event>>();
                let mut watcher = notify::recommended_watcher(move |event| tx.send(event).unwrap())?;
                watcher.watch(Path::new("src"), notify::RecursiveMode::Recursive)?;

                let mut child = tokio::process::Command::new(exe.display().to_string())
                    .arg(full_dist_path.as_str())
                    .spawn()?;

                let cd = std::env::current_dir()?;
                let dist = PathBuf::from(&dist_name);
                let entry = PathBuf::from(settings.entry.as_deref().unwrap_or("src/main.lua"));
                let conf = PathBuf::from("src/conf.lua");
 
                if watch {

                    loop {
                        tokio::select! {
                            event = rx.recv() => if let Some(event) = event {
                                let event = event?;
                                for path in event.paths {
                                    let path = path.strip_prefix(&cd).unwrap();
                                    if path == entry {
                                        match event.kind {
                                            notify::EventKind::Remove(_) => {
                                                std::fs::remove_file(dist.join("main.lua"))?;
                                            }
                                            notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                                                std::fs::copy(path, dist.join("main.lua"))?;
                                            }
                                            _ => {}
                                        }
                                    } else if path == conf {
                                        match event.kind {
                                            notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                                                std::fs::copy(&conf, dist.join("conf.lua"))?;
                                            }
                                            notify::EventKind::Remove(_) => if let Some(cfg) = config.config.as_ref() {
                                                let conf = format!("function love.conf({})\n{}end", Config::param(), cfg);
                                                std::fs::write(dist.join("conf.lua"), conf)?;
                                            } else {
                                                std::fs::remove_file(dist.join("conf.lua"))?;
                                            },
                                                _ => {}
                                        }
                                    } else {
                                        let dest = path.strip_prefix("src").map(|p| dist.join(p)).unwrap();
                                        match event.kind {
                                            notify::EventKind::Remove(_) => if dest.exists() {
                                                std::fs::remove_file(dest)?;
                                            }
                                            notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                                                std::fs::copy(path, dest)?;
                                            }
                                            _ => {}
                                        }
                                    }
                                }

                                child.kill().await.expect("failed to kill old game process");
                                child = tokio::process::Command::new(exe.display().to_string())
                                    .arg(full_dist_path.as_str())
                                    .spawn()?;
                                },
                                _ = child.wait() => {
                                    break;
                                },
                        }
                    }
                }

                child.wait().await?;

                if PathBuf::from(&dist_name).exists() {
                    std::fs::remove_dir_all(&dist_name)?;
                }

                println!("Exited");
            }
        }
        Subcommand::Init { framework, version } => {
            config.targets
                .insert(framework, Settings {
                    version: version.unwrap_or(framework.latest()),
                    ..Default::default()
                });

            config.save()?;
        }
        Subcommand::New {
            name,
            framework,
            version,
        } => {
            let dir = std::env::current_dir()?.join(name);

            if dir.exists() {
                return Err(anyhow::anyhow!("directory already exists: {dir:?}"));
            }

            std::fs::create_dir_all(dir.join("src"))?;
            std::fs::write(dir.join("src").join("main.lua"), framework.sample())?;

            config.targets
                .insert(
                    framework,
                    Settings {
                        version: version.unwrap_or(framework.latest()),
                        ..Default::default()
                    },
                );

            config.save()?;
        }
    }

    Ok(())
}
