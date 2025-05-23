use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use std::process::Command as Process;
use clap::Parser;
use love_build_tools::Config;
use love_version_manager::{git::release::AssetType, Manager as VersionManager};
use lua_language_addon_manager::Manager as AddonManager;
use zip::write::SimpleFileOptions;

#[derive(clap::Parser)]
#[clap(
    about = "Love Build Tools",
    long_about = "An automated tool for LOVE with featurs like running, building, creating CI/CD, installing versions, etc."
)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    #[clap(about = "Initialize a directory as a love project")]
    Init,
    #[clap(
        about = "Create a new love project",
        long_about = "Create a new directory and scaffold default files for a love project"
    )]
    New { name: String },
    #[clap(about = "Run the love project in the current directory")]
    Run {
        #[arg(long)]
        target: Option<AssetType>
    },
}

async fn process() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;

    match cli.command {
        Command::Init => {
            let current_dir = std::env::current_dir()?;
            let manager = AddonManager::new(&current_dir);
            manager.init()?;

            if Config::read("love.toml")?.is_none() {
                Config::new(current_dir.file_name().unwrap().to_string_lossy(), "0.1.0")
                    .write("love.toml")?;
            }

            if !current_dir.join(".gitignore").exists() {
                std::fs::write(current_dir.join(".gitignore"), "target")?;
            }

            if !PathBuf::from("main.lua").exists() {
                std::fs::write(
                    "main.lua",
                    indoc::indoc!(
                        "
                        function love.draw()
                            love.graphics.print('Hello World!', 400, 300)
                        end
                    "
                    ),
                )?;
            }

            manager.add("love2d")?;
        }
        Command::New { name } => {
            let new_dir = std::env::current_dir()?.join(&name);
            if new_dir.exists() {
                return Err(anyhow::anyhow!("directory '{name}' already exists"));
            }
            std::fs::create_dir_all(&new_dir)?;

            std::fs::write(new_dir.join(".gitignore"), "target")?;

            let manager = AddonManager::new(&new_dir);
            manager.init()?;

            Config::new(new_dir.file_name().unwrap().to_string_lossy(), "0.1.0")
                .write(new_dir.join("love.toml"))?;

            std::fs::write(
                new_dir.join("main.lua"),
                indoc::indoc!(
                    "
                    function love.draw()
                        love.graphics.print('Hello World!', 400, 300)
                    end
                "
                ),
            )?;

            manager.add("love2d")?;
        },
        Command::Run { target } => {
            let target = match target {
                None => match Config::default_target() {
                    None => anyhow::bail!("cannot resolve target; the current platform does not have a default target and a specific target was not provided"),
                    Some(t) => t
                },
                Some(t) => t,
            };

            // TODO: Check love_version in config and on system.
            //  If it doesnt exist install the needed bundle for the current system
            let mut installed = VersionManager::installed()?;
            installed.sort();

            let cfg = Config::read("love.toml")?.unwrap_or_default();

            let version = if let Some(version) = cfg.love_version.as_ref() {
                if !installed.contains(version) || !VersionManager::bundles(version)?.contains(&target) {
                    // TODO: Auto install bundle for specific version
                    anyhow::bail!("{target} bundle for love '{version}' is not installed")
                }

                version
            } else {
                if installed.is_empty() {
                    // TODO: Auto install bundle for latest version
                    anyhow::bail!("{target} bundle for love 'latest' is not installed")
                }

                installed.last().unwrap()
            };

            let current_dir = std::env::current_dir()?;
            match target {
                AssetType::Win64 | AssetType::Win32 => {
                    if cfg!(not(target_os = "windows")) {
                        anyhow::bail!("cannot run the windows executable on a non windows machine")
                    }

                    if target == AssetType::Win64 && cfg!(not(all(target_os = "windows", target_arch = "x86_64"))) {
                        anyhow::bail!("cannot run the x86_64 windows executable on a x86 windows machine")
                    }

                    let path = VersionManager::bundle_path(version, target);
                    let executable = path.join("love.exe");

                    let output = Process::new(&executable)
                        .arg(&current_dir)
                        .arg("--console")
                        .output()?;

                    if !output.status.success() {
                        std::process::exit(1);
                    }
                },
                AssetType::Macos => {
                    if cfg!(not(target_os = "macos")) {
                        anyhow::bail!("cannot run the macos executable on a non macos machine")
                    }

                    let path = VersionManager::bundle_path(version, target);
                    let executable = path.join("Contents").join("Macos").join("love");

                    let output = Process::new(&executable)
                        .arg(&current_dir)
                        .arg("--console")
                        .output()?;

                    if !output.status.success() {
                        std::process::exit(1);
                    }
                },
                AssetType::Linux => {
                    if cfg!(not(target_os = "linux")) {
                        anyhow::bail!("cannot run the linux AppImage on a non unix machine")
                    }

                    let path = VersionManager::bundle_path(version, target);
                    let executable = path.join("Contents").join("Macos").join("love");

                    let output = Process::new(&executable)
                        .arg(&current_dir)
                        .arg("--console")
                        .output()?;

                    if !output.status.success() {
                        std::process::exit(1);
                    }
                },
                AssetType::Web => {
                    let path = VersionManager::bundle_path(version, target);
                    let executable = path.join("index.js");

                    if !path.join("node_modules").exists() {
                        let output = Process::new("npm")
                            .arg("install")
                            .current_dir(&path)
                            .output()?;
                        if !output.status.success() {
                            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                            std::process::exit(1);
                        }
                    }

                    let debug_dir = current_dir.join("target").join("debug");
                    if !debug_dir.exists() {
                        std::fs::create_dir_all(&debug_dir)?;
                    }

                    let love_archive = debug_dir.join(format!("{}.love", cfg.name));
                    zip_all(&current_dir, &love_archive)?;

                    let output = Process::new("node")
                            .arg(&executable)
                            .arg(&love_archive)
                            .arg(debug_dir.join("output"))
                            .arg("-c")
                            .arg("-t")
                            .arg(&cfg.name)
                            .output()?;

                    if !output.status.success() {
                        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                        std::process::exit(1);
                    }

                    // TODO: Run web server that hosts the generated app and serves the correct
                    // headers
                    println!("Web assets generated at '{}'", debug_dir.join("output").display());
                },
                other => unimplemented!("{other} is not yet implemented for running love projects")
            }


            // TODO: Get path to bundles executable

            // TODO: Run executable given root path to files
        }
    }

    Ok(())
}

fn zip_all(dir: impl AsRef<Path>, dest: impl AsRef<Path>) -> anyhow::Result<()> {
    let base = dir.as_ref().to_path_buf();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(dest.as_ref())?;

    let walk = ignore::WalkBuilder::new(dir.as_ref())
        .filter_entry(move |p| {
            let path = p.path().strip_prefix(&base).unwrap().display().to_string();
            path != "love.toml"
            && path != "target"
        })
        .hidden(true)
        .ignore(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    let mut writer = zip::ZipWriter::new(&mut file);
    let options = SimpleFileOptions::default();

    let mut buffer = Vec::new();
    for result in walk.flatten() {
        let path = result.path().strip_prefix(dir.as_ref()).unwrap().to_string_lossy().to_string();
        if result.path().is_file() {
            writer.start_file(path, options)?;
            let mut f = File::open(result.path())?;
            f.read_to_end(&mut buffer)?;
            writer.write_all(&buffer)?;
            buffer.clear();
        } else if !path.is_empty() {
            writer.add_directory(path, options)?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = process().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
