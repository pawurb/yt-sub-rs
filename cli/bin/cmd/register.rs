use std::path::PathBuf;

use clap::Parser;
use eyre::Result;
use yt_sub::user_settings_cli::UserSettingsCLI;
use yt_sub_core::UserSettings;

#[derive(Debug, Parser)]
pub struct RegisterArgs {
    #[arg(
        long,
        help = "Path to config file, deafult '~/.config/yt-sub-rs/config.toml'"
    )]
    config: Option<PathBuf>,
}

impl RegisterArgs {
    pub async fn run(self) -> Result<()> {
        let Self { config } = self;

        let settings = UserSettings::read(config.as_ref())?;
        settings.register_remote(None).await?;
        println!("Registered successfully! You'll receive notifications about new videos to your configured Slack channel.");
        Ok(())
    }
}
