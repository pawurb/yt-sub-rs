use clap::{Parser, Subcommand};
mod cmd;
use cmd::{
    channel_data::ChannelDataArgs, follow::FollowArgs, init::InitArgs, list::ListArgs,
    run::RunArgs, settings::SettingsArgs, unfollow::UnfollowArgs,
};
use eyre::Result;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct SubArgs {
    #[command(subcommand)]
    pub cmd: SubSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum SubSubcommand {
    #[command(visible_alias = "i", about = "Initialize config file")]
    Init(InitArgs),
    #[command(visible_alias = "s", about = "Display current settings")]
    Settings(SettingsArgs),
    #[command(visible_alias = "r", about = "Check and notify about fresh videos")]
    Run(RunArgs),
    #[command(visible_alias = "d", about = "Get a channel data based on its handle")]
    ChannelData(ChannelDataArgs),
    #[command(visible_alias = "f", about = "Subscribe to a channel")]
    Follow(FollowArgs),
    #[command(visible_alias = "u", about = "Unsubscribe")]
    Unfollow(UnfollowArgs),
    #[command(visible_alias = "l", about = "List followed channels")]
    List(ListArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = SubArgs::parse();
    let res = match args.cmd {
        SubSubcommand::Init(args) => args.run(),
        SubSubcommand::Settings(args) => args.run(),
        SubSubcommand::Run(args) => args.run().await,
        SubSubcommand::ChannelData(args) => args.run().await,
        SubSubcommand::Follow(args) => args.run().await,
        SubSubcommand::Unfollow(args) => args.run().await,
        SubSubcommand::List(args) => args.run().await,
    };

    match res {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e)
        }
    };

    Ok(())
}
