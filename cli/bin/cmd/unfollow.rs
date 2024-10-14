use std::path::PathBuf;

use clap::Parser;
use eyre::Result;
use yt_sub::user_settings::UserSettings;

#[derive(Debug, Parser)]
pub struct UnfollowArgs {
    #[arg(
        long,
        help = "Path to config file, deafult '~/.config/yt-sub-rs/config.toml'"
    )]
    config: Option<PathBuf>,

    #[arg(long)]
    handle: String,
}

impl UnfollowArgs {
    pub async fn run(self) -> Result<()> {
        let Self { handle, config } = self;

        let settings = UserSettings::read(config.as_ref())?;

        let to_unfollow = settings.get_channel_by_handle(&handle);

        if to_unfollow.is_none() {
            eyre::bail!("You are not following a channel with the provided handle!")
        }

        let to_unfollow = to_unfollow.unwrap();

        let settings = UserSettings {
            channels: settings
                .channels
                .iter()
                .filter(|channel| channel.handle != handle)
                .cloned()
                .collect(),
            ..settings
        };

        settings.sync(config.as_ref())?;

        println!("You've unfollowed {desc}!", desc = to_unfollow.description);

        Ok(())
    }
}
