use std::path::PathBuf;

use clap::Parser;
use eyre::Result;
use yt_sub::{channel::Channel, user_settings_cli::UserSettingsCLI, UserSettings};

use crate::CONFIG_DESC;

#[derive(Debug, Parser)]
pub struct ChannelDataArgs {
    #[arg(long, help = CONFIG_DESC)]
    config: Option<PathBuf>,

    #[arg(long)]
    handle: String,
}

impl ChannelDataArgs {
    pub async fn run(self) -> Result<()> {
        let Self { config, handle } = self;
        let settings = UserSettings::read(config.as_ref())?;
        let (channel_id, channel_name) =
            Channel::get_data(&handle, settings.youtube_api_key()?, None).await?;

        let channel = Channel {
            handle: handle.clone(),
            description: channel_name.clone(),
            channel_id: channel_id.clone(),
        };

        println!(
            "{channel}

Run: 

ytsub follow --handle {handle} --channel-id {channel_id} --desc '{channel_name}'

to subscribe to this channel.",
            handle = channel.handle,
            channel_id = channel.channel_id,
        );
        Ok(())
    }
}
