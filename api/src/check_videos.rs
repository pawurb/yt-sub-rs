use crate::{store::KvWrapper, user_settings_api::UserSettingsAPI};
use chrono::{Timelike, Utc};
use eyre::Result;
use wasm_rs_dbg::dbg as wdbg;
use yt_sub_core::UserSettings;

pub async fn check_videos(api_key: String, kv: &mut impl KvWrapper) -> Result<()> {
    let settings = UserSettings::read(&api_key, kv).await?;

    if !matching_schedule(&settings) {
        return Ok(());
    }

    let last_run_at = settings.last_run_at(kv).await?;
    let mut new_videos = vec![];

    for channel in &settings.channels {
        match channel.get_fresh_videos(last_run_at).await {
            Ok(videos) => {
                new_videos.extend(videos);
            }
            Err(e) => {
                wdbg!(format!("Error: {}", e));
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
                wdbg!(&format!("Error: {e}"));
            }
        }
    }

    settings.touch_last_run_at(kv).await?;

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
