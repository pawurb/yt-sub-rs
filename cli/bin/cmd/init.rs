use clap::Parser;
use eyre::Result;
use std::path::PathBuf;
use yt_sub_core::UserSettings;

#[derive(Debug, Parser)]
pub struct InitArgs {
    #[arg(
        long,
        help = "Path to config file, deafult '~/.config/yt-sub-rs/config.toml'"
    )]
    config: Option<PathBuf>,
}

impl InitArgs {
    pub fn run(self) -> Result<()> {
        let Self { config } = self;

        let settings = UserSettings::init(config.as_ref())?;
        println!(
            "Config file created at: {config_path}",
            config_path = settings.path.display()
        );
        Ok(())
    }
}
