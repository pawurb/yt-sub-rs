use clap::Parser;
use eyre::Result;
use yt_sub::channel::Channel;

#[derive(Debug, Parser)]
pub struct ChannelDataArgs {
    #[arg(long)]
    handle: String,
}

impl ChannelDataArgs {
    pub async fn run(self) -> Result<()> {
        let Self { handle } = self;
        let (channel_id, channel_name) = Channel::get_data(&handle, None).await?;

        let channel = Channel {
            handle: handle.clone(),
            description: channel_name.clone(),
            channel_id: channel_id.clone(),
        };

        println!(
            "{channel}

Run: 

sub follow --handle {handle} --channel-id {channel_id} --desc '{channel_name}'

to subscribe to this channel.",
            handle = channel.handle,
            channel_id = channel.channel_id,
        );
        Ok(())
    }
}
