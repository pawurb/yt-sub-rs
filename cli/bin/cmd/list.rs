use std::path::PathBuf;

use clap::Parser;
use eyre::Result;
use yt_sub::user_settings_cli::UserSettingsCLI;
use yt_sub_core::UserSettings;

#[derive(Debug, Parser)]
pub struct ListArgs {
    #[arg(
        long,
        help = "Path to config file, deafult '~/.config/yt-sub-rs/config.toml'"
    )]
    config: Option<PathBuf>,
}

impl ListArgs {
    pub async fn run(self) -> Result<()> {
        let Self { config } = self;

        let settings = UserSettings::read(config.as_ref())?;
        let channels = settings.channels;

        if channels.is_empty() {
            println!("Currently you are not following any channels.");
            return Ok(());
        }

        let channels_str = channels
            .iter()
            .map(|channel| channel.to_string())
            .collect::<Vec<_>>()
            .join("\n\n");

        println!(
            "You are following:

{channels_str}",
        );

        Ok(())
    }
}
