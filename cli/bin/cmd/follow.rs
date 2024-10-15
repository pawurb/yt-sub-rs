use std::path::PathBuf;

use clap::Parser;
use eyre::Result;
use yt_sub_core::{channel::Channel, UserSettings};

#[derive(Debug, Parser)]
pub struct FollowArgs {
    #[arg(
        long,
        help = "Path to config file, deafult '~/.config/yt-sub-rs/config.toml'"
    )]
    config: Option<PathBuf>,

    #[arg(long)]
    handle: String,
    #[arg(long)]
    channel_id: Option<String>,
    #[arg(long)]
    desc: Option<String>,
}

impl FollowArgs {
    pub async fn run(self) -> Result<()> {
        let Self {
            channel_id,
            desc,
            handle,
            config,
        } = self;

        if (channel_id.is_none() && desc.is_some()) || (channel_id.is_some() && desc.is_none()) {
            eyre::bail!("You must provide only --handle or both --channel-id and --desc");
        }

        let (channel_id, desc) = if channel_id.is_none() && desc.is_none() {
            Channel::get_data(&handle, None).await?
        } else {
            (channel_id.unwrap(), desc.unwrap())
        };

        let settings = UserSettings::read(config.as_ref())?;
        let already_following_id = settings.get_channel_by_id(&channel_id);
        let already_following_handle = settings.get_channel_by_handle(&handle);

        if already_following_id.is_some() || already_following_handle.is_some() {
            let following = already_following_id.unwrap_or(already_following_handle.unwrap());

            eyre::bail!("You are already following this channel! \n\n{following}");
        }

        if !Channel::validate_id(&channel_id, None).await? {
            eyre::bail!("Provided channel-id is invalid!");
        }

        let mut channels = settings.channels;

        let channel = Channel {
            handle,
            description: desc.clone(),
            channel_id,
        };

        channels.push(channel.clone());
        let settings = UserSettings {
            channels,
            ..settings
        };

        settings.sync(config.as_ref())?;

        println!(
            "You are now following:

{channel}!"
        );

        Ok(())
    }
}
