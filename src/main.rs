use clap::Parser;
use lbt::{
    build::Builder,
    config::{Build, Config, Framework, Target},
    git, Progress, Version,
};

#[derive(Parser)]
pub struct LBT {
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    Build,
    Run,
    Init {
        framework: Framework,
        version: Option<Version>,
    },
    New {
        name: String,
        framework: Framework,
        version: Option<Version>,
    },
    Pass,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = git::Client::new("love-build-tools");

    let mut config = Config::parse_or_default()?;

    // TODO: Convert from install command to pull from a config
    #[allow(clippy::single_match)]
    match LBT::parse().command {
        Subcommand::Build => {
            for (framework, build) in config.build.iter() {
                Builder::new(framework, build, &config)
                    .bundle(&client)
                    .await?;
            }
        }
        Subcommand::Run => {
            let target = Target::default();
            if let Some((key, _value)) = config.build.first_key_value() {
                let exe = key.exe(target);
                let output = std::process::Command::new(exe.display().to_string())
                    .arg(
                        std::env::current_dir()
                            .unwrap()
                            .join("src")
                            .display()
                            .to_string(),
                    )
                    .output()?;
                std::process::exit(output.status.code().unwrap());
            }
        }
        Subcommand::Init { framework, version } => {
            let dir = std::env::current_dir()?;

            config.build.insert(
                framework,
                Build {
                    version: version.unwrap_or(framework.latest()),
                    targets: Vec::default(),
                },
            );
            std::fs::write(dir.join("lbt.toml"), toml::to_string_pretty(&config)?)?;
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

            config.build.insert(
                framework,
                Build {
                    version: version.unwrap_or(framework.latest()),
                    targets: Vec::default(),
                },
            );
            std::fs::write(dir.join("lbt.toml"), toml::to_string_pretty(&config)?)?;
        }
        _ => {}
    }

    Ok(())
}
