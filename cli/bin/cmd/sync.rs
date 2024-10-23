use std::path::PathBuf;

use clap::Parser;
use eyre::Result;
use yt_sub::user_settings_cli::UserSettingsCLI;
use yt_sub_core::UserSettings;

use crate::CONFIG_DESC;

#[derive(Debug, Parser)]
pub struct SyncArgs {
    #[arg(long, help = CONFIG_DESC)]
    config: Option<PathBuf>,
}

impl SyncArgs {
    pub async fn run(self) -> Result<()> {
        let Self { config } = self;

        let settings = UserSettings::read(config.as_ref())?;
        settings.sync_account(None).await?;
        println!("Remote account data was updated.");

        Ok(())
    }
}
