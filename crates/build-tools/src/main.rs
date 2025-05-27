use std::collections::BTreeSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use std::process::Command as Process;
use clap::Parser;
use love_build_tools::Config;
use love_version_manager::{git::release::AssetType, Manager as VersionManager};
use lua_language_addon_manager::Manager as AddonManager;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::name::QName;
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
    #[clap(about = "Build the love project in the current directory")]
    Build {
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
                let name = current_dir.file_name().unwrap().to_string_lossy();
                let id = format!("com.{name}");
                Config::new(name, id, "0.1.0").write("love.toml")?;
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

            let name = new_dir.file_name().unwrap().to_string_lossy();
            let id = format!("com.{name}");
            Config::new(name, id, "0.1.0") .write(new_dir.join("love.toml"))?;

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
                    zip_all(&current_dir, &love_archive, vec!["target", "love.toml"])?;

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
        },
        Command::Build { target } => {
            let target = match target {
                None => match Config::default_target() {
                    None => anyhow::bail!("cannot resolve target; the current platform does not have a default target and a specific target was not provided"),
                    Some(t) => t
                },
                Some(t) => t,
            };

            let mut installed = VersionManager::installed()?;
            installed.sort();

            let cfg = Config::read("love.toml")?.unwrap_or_default();

            // TODO: Check love_version in config and on system.
            //  If it doesnt exist install the needed bundle for the current system
            let version = if let Some(version) = cfg.love_version.as_ref() {
                if !installed.contains(version) || !VersionManager::bundles(version)?.contains(&target) {
                    // TODO: Auto install bundle for specific version
                    anyhow::bail!("{target} bundle for love '{version}' is not installed")
                }

                version
            } else {
                let version = installed.last();
                let bundles = version.map(VersionManager::bundles).transpose()?;

                if installed.is_empty() || !bundles.map(|v| v.contains(&target)).unwrap_or_default() {
                    // TODO: Auto install bundle for latest version
                    anyhow::bail!("{target} bundle for love '{}' is not installed", version.map(String::as_str).unwrap_or("latest"))
                }

                installed.last().unwrap()
            };

            let current_dir = std::env::current_dir()?;
            let dest = PathBuf::from("target").join("release");
            if !dest.exists() {
                std::fs::create_dir_all(&dest)?;
            }

            match target {
                AssetType::Win64 | AssetType::Win32 => {
                    let bundle_path = VersionManager::bundle_path(version, target);

                    let executable = bundle_path.join("love.exe");
                    let libs = bundle_path
                        .read_dir()?
                        .flatten()
                        .filter_map(|e| {
                            let path = e.path();
                            path
                                .extension()
                                .map(|v| v.to_string_lossy().as_ref() == "dll")
                                .unwrap_or_default()
                                .then_some((path.clone(), path.file_name().unwrap().to_string_lossy().to_string()))
                        })
                        .collect::<Vec<_>>();

                    let target_path = dest.join(target.to_string());
                    if !target_path.exists() {
                        std::fs::create_dir_all(&target_path)?;
                    }
                    let archive_path = target_path.join(format!("{}.love", cfg.name));
                    let current_dir = std::env::current_dir()?;
                    zip_all(&current_dir, &archive_path, vec!["target", "love.toml"])?;

                    let executable_path = (target_path.join(format!("{}.exe", cfg.name)), format!("{}.exe", cfg.name));
                    {
                        let mut outfile = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .create(true)
                            .open(&executable_path.0)?;

                        {
                            let mut executable = std::fs::File::open(&executable)?;
                            std::io::copy(&mut executable, &mut outfile)?;
                        }

                        {
                            let mut love_archive = std::fs::File::open(&archive_path)?;
                            std::io::copy(&mut love_archive, &mut outfile)?;
                        }
                    }

                    for (dll, name) in libs.iter() {
                        if !dest.join(name).exists() {
                            std::fs::copy(dll, target_path.join(name))?;
                        }
                    }

                    zip(
                        dest.join(format!("{}-{}.zip", cfg.name, target)), 
                        libs
                            .into_iter()
                            .chain([executable_path])
                    )?;
                },
                AssetType::Macos => {
                    let current_dir = std::env::current_dir()?;
                    let bundle_path = VersionManager::bundle_path(version, target);
                    let dest = PathBuf::from("target").join("release");
                    let target_path = dest.join(target.to_string());
                    let app_path = target_path.join(format!("{}.app", cfg.name));
                    let archive_path = app_path.join("Contents").join("Resources").join(format!("{}.love", cfg.name));
                    let plist_path = app_path.join("Contents").join("Info.plist");

                    copy_all(&bundle_path, &app_path)?;
                    zip_all(&current_dir, &archive_path, vec!["target", "love.toml"])?;

                    let info_plist = std::fs::read_to_string(&plist_path)?;
                    let mut reader = quick_xml::Reader::from_str(&info_plist);
                    let mut out_plist = std::fs::OpenOptions::new()
                        .create(true)
                        .truncate(true)
                        .write(true)
                        .open(plist_path)?;
                    let mut writer = quick_xml::Writer::new(&mut out_plist);
                    loop {
                        match reader.read_event() {
                            Err(e) => return Err(e.into()),
                            Ok(event) => {
                                match event {
                                    Event::Eof => break,
                                    Event::Start(e) => {
                                        writer.write_event(Event::Start(e.clone()))?;

                                        if let b"dict" = e.name().as_ref() {
                                            let mut stack: usize = 1;
                                            while stack > 0 {
                                                match reader.read_event()? {
                                                    Event::Start(e) => {
                                                        if e.name().as_ref() == b"key" && stack < 2 {
                                                            let text = reader.read_text(e.name())?;
                                                            _ = reader.read_event()?;

                                                            match text.as_ref() {
                                                                "CFBundleIdentifier" => {
                                                                    writer.write_event(Event::Start(e.clone()))?;
                                                                    writer.write_event(Event::Text(BytesText::new(text.as_ref())))?;
                                                                    writer.write_event(Event::End(e.to_end()))?;

                                                                    _ = reader.read_event()?;
                                                                    _ = reader.read_text(QName(b"string"))?;
                                                                    _ = reader.read_event()?;
                                                                    writer.write_event(Event::Start(BytesStart::new("string")))?;
                                                                    writer.write_event(Event::Text(BytesText::new(&cfg.id)))?;
                                                                    writer.write_event(Event::End(BytesEnd::new("string")))?;
                                                                }
                                                                "CFBundleName" => {
                                                                    writer.write_event(Event::Start(e.clone()))?;
                                                                    writer.write_event(Event::Text(BytesText::new(text.as_ref())))?;
                                                                    writer.write_event(Event::End(e.to_end()))?;

                                                                    _ = reader.read_event()?;
                                                                    _ = reader.read_text(QName(b"string"))?;
                                                                    _ = reader.read_event()?;
                                                                    writer.write_event(Event::Start(BytesStart::new("string")))?;
                                                                    writer.write_event(Event::Text(BytesText::new(&cfg.name)))?;
                                                                    writer.write_event(Event::End(BytesEnd::new("string")))?;
                                                                },
                                                                "UTExportedTypeDeclarations" => {
                                                                    _ = reader.read_event()?;
                                                                    _ = reader.read_text(QName(b"array"))?;
                                                                    _ = reader.read_event()?;
                                                                }
                                                                _ => {
                                                                    writer.write_event(Event::Start(e.clone()))?;
                                                                    writer.write_event(Event::Text(BytesText::new(text.as_ref())))?;
                                                                    writer.write_event(Event::End(e.to_end()))?;
                                                                }
                                                            }
                                                        } else if e.name().as_ref() == b"dict" {
                                                            stack += 1;
                                                            writer.write_event(Event::Start(e))?
                                                        } else {
                                                            writer.write_event(Event::Start(e))?
                                                        }
                                                    },
                                                    Event::End(e) if e.name().as_ref() == b"dict" => {
                                                        stack = stack.saturating_sub(1);
                                                        writer.write_event(Event::End(e))?
                                                    }
                                                    other => writer.write_event(other)?,
                                                }
                                            }
                                        }
                                    },
                                    other => writer.write_event(other)?
                                }
                            }
                        }
                    }

                    zip_all(target_path, dest.join(format!("{}-osx.zip", cfg.name)), vec![])?;
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

                    let dest = current_dir.join("target").join("release");
                    let target_path = dest.join(target.to_string());
                    if !target_path.exists() {
                        std::fs::create_dir_all(&target_path)?;
                    }

                    let love_archive = target_path.join(format!("{}.love", cfg.name));
                    zip_all(&current_dir, &love_archive, vec!["target", "love.toml"])?;

                    let output = Process::new("node")
                            .arg(&executable)
                            .arg(&love_archive)
                            .arg(&target_path)
                            .arg("-c")
                            .arg("-t")
                            .arg(&cfg.name)
                            .output()?;

                    if !output.status.success() {
                        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                        std::process::exit(1);
                    }

                    zip_all(&target_path, dest.join(format!("{}-web.zip", cfg.name)), vec![])?;
                },
                other => unimplemented!("{other} is not yet implemented for building love projects")
            }


            // TODO: Get path to bundles executable

            // TODO: Run executable given root path to files
        }
    }

    Ok(())
}

fn copy_all(from: impl AsRef<Path>, to: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut stack = Vec::new();
    stack.push(from.as_ref().to_path_buf());

    while let Some(wkd) = stack.pop() {
        let src = wkd.strip_prefix(from.as_ref())?;

        let dest = if src.components().count() == 0 {
            to.as_ref().to_path_buf()
        } else {
            to.as_ref().join(src)
        };

        if !dest.exists() {
            std::fs::create_dir_all(&dest)?;
        }

        for entry in wkd.read_dir()?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Some(filename) = path.file_name()  {
                std::fs::copy(&path, dest.join(filename))?;
            }
        }
    }

    Ok(())
}

fn zip<P: AsRef<Path>, D: AsRef<Path>>(dest: impl AsRef<Path>, files: impl IntoIterator<Item = (P, D)>) -> anyhow::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(dest.as_ref())?;

    let mut directories = BTreeSet::new();

    let mut writer = zip::ZipWriter::new(&mut file);
    let options = SimpleFileOptions::default();
    let mut buffer: Vec<u8> = Vec::new();

    for (src, dest) in files.into_iter() {
        if let Ok(mut file) = std::fs::File::open(src.as_ref()) {
            if let Some(parent) = dest.as_ref().parent() {
                let mut current = PathBuf::new();
                for component in parent.components() {
                    current.push(component);
                    if !directories.contains(&current) {
                        println!("[DIR] Add Parent Directory: {}", current.display());
                        writer.add_directory(current.display().to_string(), options)?;
                        directories.insert(current.clone());
                    }
                }
            }

            println!("[FILE] Add File: {}", dest.as_ref().display());
            let path_str = dest.as_ref().display().to_string();
            writer.start_file(path_str, options)?;
            file.read_to_end(&mut buffer)?;
            writer.write_all(&buffer)?;
            buffer.clear();
        }
    }

    Ok(())
}

fn zip_all(dir: impl AsRef<Path>, dest: impl AsRef<Path>, exclude: Vec<&'static str>) -> anyhow::Result<()> {
    let base = dir.as_ref().to_path_buf();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(dest.as_ref())?;

    let walk = ignore::WalkBuilder::new(dir.as_ref())
        .filter_entry(move |p| {
            let path = p.path().strip_prefix(&base).unwrap().display().to_string();
            !exclude.contains(&path.as_str())
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
