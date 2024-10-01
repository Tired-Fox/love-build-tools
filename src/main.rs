use clap::Parser;
use lbt::{build::Builder, config::{Config, Target}, git};

#[derive(Parser)]
pub struct LBT {
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    Build,
    Run,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = git::Client::new("love-build-tools");

    let config = Config::parse_or_default()?;

    // TODO: Convert from install command to pull from a config
    #[allow(clippy::single_match)]
    match LBT::parse().command {
        Subcommand::Build => {
            for (framework, build) in config.build.iter() {
                Builder::new(framework, build, &config)
                    .bundle(&client)
                    .await?;
            }
        },
        Subcommand::Run => {
            let target = Target::default();
            if let Some((key, _value)) = config.build.first_key_value() {
                let exe = key.exe(target);
                let output = std::process::Command::new(exe.display().to_string())
                    .arg(std::env::current_dir().unwrap().join("src").display().to_string())
                    .output()?;
                std::process::exit(output.status.code().unwrap());
            }
        }
    }

    Ok(())
}
