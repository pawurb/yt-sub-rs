use eyre::Result;
use yt_sub_api::{
    check_videos::check_videos, lite_helpers::sqlite_conn, user_settings_api::UserSettingsAPI,
};
use yt_sub_core::UserSettings;

#[tokio::main]
async fn main() -> Result<()> {
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
