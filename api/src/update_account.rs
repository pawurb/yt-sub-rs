use eyre::Result;
use yt_sub_core::UserSettings;

use crate::{store::KvWrapper, user_settings_api::UserSettingsAPI};

pub async fn update_account(settings: UserSettings, kv: &mut impl KvWrapper) -> Result<()> {
    let Some(api_key) = settings.api_key.clone() else {
        eyre::bail!("Missing API key!")
    };

    if kv.get_val(&api_key).await?.is_none() {
        eyre::bail!("Invalid API key present!");
    }

    if settings.get_slack_notifier().is_none() {
        eyre::bail!("Missing Slack notifier settings");
    };

    settings.save(kv).await?;

    Ok(())
}
