use clap::Parser;
use eyre::Result;
use std::path::PathBuf;
use yt_sub::user_settings_cli::UserSettingsCLI;
use yt_sub_core::UserSettings;

use crate::CONFIG_DESC;

#[derive(Debug, Parser)]
pub struct SettingsArgs {
    #[arg(long, help = CONFIG_DESC)]
    config: Option<PathBuf>,
}

impl SettingsArgs {
    pub fn run(self) -> Result<()> {
        let Self { config } = self;

        let settings = UserSettings::read(config.as_ref())?;
        println!("{settings}");
        Ok(())
    }
}
