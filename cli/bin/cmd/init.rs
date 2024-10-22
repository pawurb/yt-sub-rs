use clap::Parser;
use eyre::Result;
use std::path::PathBuf;
use yt_sub::user_settings_cli::UserSettingsCLI;
use yt_sub_core::UserSettings;

use crate::CONFIG_DESC;

#[derive(Debug, Parser)]
pub struct InitArgs {
    #[arg(long, help = CONFIG_DESC)]
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
