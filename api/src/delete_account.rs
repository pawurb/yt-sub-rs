use eyre::Result;
use yt_sub_core::UserSettings;

use crate::{store::KvWrapper, user_settings_api::UserSettingsAPI};

pub async fn delete_account(api_key: String, kv: &mut impl KvWrapper) -> Result<()> {
    UserSettings::read(&api_key, kv).await?;
    UserSettings::delete(&api_key, kv).await?;

    Ok(())
}
