use std::path::PathBuf;

use clap::Parser;
use eyre::Result;
use yt_sub::user_settings_cli::UserSettingsCLI;
use yt_sub_core::UserSettings;

use crate::CONFIG_DESC;

#[derive(Debug, Parser)]
pub struct UnregisterArgs {
    #[arg(long, help = CONFIG_DESC)]
    config: Option<PathBuf>,
}

impl UnregisterArgs {
    pub async fn run(self) -> Result<()> {
        let Self { config } = self;

        let settings = UserSettings::read(config.as_ref())?;
        settings.delete_account(None).await?;
        println!("Your remote account has been removed.");

        let settings = UserSettings {
            api_key: None,
            ..settings
        };

        settings.save(config.as_ref())?;
        Ok(())
    }
}
