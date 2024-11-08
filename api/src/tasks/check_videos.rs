use crate::{lite_helpers::sqlite_conn, user_settings_api::UserSettingsAPI};
use chrono::{Timelike, Utc};
use eyre::Result;
use yt_sub_core::UserSettings;

pub async fn run_check_videos() -> Result<()> {
    let conn = sqlite_conn(None).await?;

    let ids = match UserSettings::ids(&conn).await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("Failed to list ids: {}", &e);
            return Ok(());
        }
    };

    tracing::error!("Checking videos for {} ids", ids.len());

    for user_id in ids {
        match check_videos(user_id).await {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to check videos: {}", &e);
            }
        }
    }
    Ok(())
}

async fn check_videos(api_key: String) -> Result<()> {
    let conn = sqlite_conn(None).await?;

    let settings = UserSettings::read(&api_key, &conn).await?;

    if !matching_schedule(&settings) {
        return Ok(());
    }

    let last_run_at = settings.last_run_at(&conn).await?;
    let mut new_videos = vec![];

    for channel in &settings.channels {
        match channel.get_fresh_videos(last_run_at).await {
            Ok(videos) => {
                new_videos.extend(videos);
            }
            Err(e) => {
                tracing::error!("Error: {}", e);
            }
        }
    }

    if new_videos.is_empty() {
        return Ok(());
    }

    for notifier in &settings.notifiers {
        let notifications = new_videos
            .iter()
            .map(|video| video.notification_text(notifier))
            .collect::<Vec<String>>();

        match notifier.notify(notifications, false).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Error: {e}");
            }
        }
    }

    settings.touch_last_run_at(&conn).await?;

    Ok(())
}

fn matching_schedule(settings: &UserSettings) -> bool {
    let schedule = &settings.schedule;

    if schedule.is_none() {
        return true;
    }

    let schedule = &schedule.clone().unwrap();
    let current_utc_hour = Utc::now().hour();

    schedule
        .iter()
        .any(|&scheduled_hour| current_utc_hour == scheduled_hour)
}
