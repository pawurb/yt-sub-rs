use std::path::PathBuf;

use clap::Parser;
use eyre::Result;
use yt_sub::user_settings_cli::UserSettingsCLI;
use yt_sub_core::UserSettings;

use crate::CONFIG_DESC;

#[derive(Debug, Parser)]
pub struct RegisterArgs {
    #[arg(long, help = CONFIG_DESC)]
    config: Option<PathBuf>,
}

impl RegisterArgs {
    pub async fn run(self) -> Result<()> {
        let Self { config } = self;

        let settings = UserSettings::read(config.as_ref())?;
        settings.create_account(None).await?;
        println!("Registered successfully! You'll receive Slack notifications about new videos.");
        Ok(())
    }
}
