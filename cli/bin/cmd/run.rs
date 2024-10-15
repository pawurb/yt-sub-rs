use chrono::Utc;
use clap::Parser;
use eyre::Result;
use std::path::PathBuf;
use yt_sub_core::{logger::Logger, UserSettings};

#[derive(Debug, Parser)]
pub struct RunArgs {
    #[arg(
        long,
        help = "Path to config file, deafult '~/.config/yt-sub-rs/config.toml'"
    )]
    config: Option<PathBuf>,

    #[arg(long, help = "Produce cron-style std logs")]
    cron: bool,

    #[arg(long, help = "Fresh videos hours offset")]
    hours_offset: Option<u16>,
}

impl RunArgs {
    pub async fn run(self) -> Result<()> {
        let Self {
            config,
            cron,
            hours_offset,
        } = self;

        let logger = Logger::new(cron);

        let settings = UserSettings::read(config.as_ref())?;

        let last_run_at = if let Some(hours_offset) = hours_offset {
            Utc::now() - chrono::Duration::hours(hours_offset as i64)
        } else {
            settings.get_last_run_at()
        };

        let mut new_videos = vec![];

        for channel in &settings.channels {
            match channel.get_fresh_videos(last_run_at).await {
                Ok(videos) => {
                    new_videos.extend(videos);
                }
                Err(e) => {
                    logger.error(&format!("Error: {e}"));
                }
            }
        }

        for notifier in &settings.notifiers {
            let notifications = new_videos
                .iter()
                .map(|video| video.notification_text(notifier))
                .collect::<Vec<String>>();

            match notifier.notify(notifications, cron).await {
                Ok(_) => {}
                Err(e) => {
                    logger.error(&format!("Error: {e}"));
                }
            }
        }

        settings.update_last_run_at()?;

        Ok(())
    }
}
