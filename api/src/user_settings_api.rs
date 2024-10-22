use chrono::{DateTime, Duration, Utc};
use eyre::Result;
use yt_sub_core::UserSettings;

use crate::store::KvWrapper;
static USER_IDS_KEY: &str = "user_ids";

#[allow(async_fn_in_trait)]
pub trait UserSettingsAPI {
    async fn last_run_at(&self, kv: &impl KvWrapper) -> Result<DateTime<Utc>>;
    async fn touch_last_run_at(&self, kv: &mut impl KvWrapper) -> Result<()>;
    fn api_key(&self) -> String;
    async fn list_ids(kv: &impl KvWrapper) -> Result<Vec<String>>;
    async fn save(&self, kv: &mut impl KvWrapper) -> Result<()>;
}

impl UserSettingsAPI for UserSettings {
    fn api_key(&self) -> String {
        self.api_key.clone().expect("Missing API key")
    }

    async fn list_ids(kv: &impl KvWrapper) -> Result<Vec<String>> {
        let user_ids = kv
            .get_val(USER_IDS_KEY)
            .await?
            .unwrap_or_else(|| "".to_string());

        let user_ids: Vec<String> = user_ids
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(user_ids)
    }

    async fn save(&self, kv: &mut impl KvWrapper) -> Result<()> {
        let mut user_ids = Self::list_ids(kv).await?;

        if user_ids.contains(&self.api_key()) {
            eyre::bail!("Duplicate API key")
        }

        user_ids.push(self.api_key().clone());
        kv.put_val(USER_IDS_KEY, &user_ids.join(",")).await?;

        let json = serde_json::to_string(&self)?;
        kv.put_val(&self.api_key(), &json).await?;

        Ok(())
    }

    async fn last_run_at(&self, kv: &impl KvWrapper) -> Result<DateTime<Utc>> {
        let key = format!("last_run_at_{}", self.api_key());

        let Some(last_run_at) = kv.get_val(&key).await? else {
            return Ok(Utc::now() - Duration::days(7));
        };
        let last_run_at = DateTime::parse_from_rfc3339(&last_run_at)
            .expect("Failed to parse last_run_at file")
            .with_timezone(&Utc);

        Ok(last_run_at)
    }

    async fn touch_last_run_at(&self, kv: &mut impl KvWrapper) -> Result<()> {
        let key = format!("last_run_at_{}", self.api_key());
        kv.put_val(&key, &Utc::now().to_rfc3339()).await?;

        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use crate::{registration::tests::build_settings, store::tests::MockKvStore};
    use UserSettingsAPI;

    use super::*;

    #[tokio::test]
    async fn test_add_list_users() -> Result<()> {
        let mut kv = MockKvStore::default();

        let before: Vec<String> = UserSettings::list_ids(&kv).await?;
        assert_eq!(before.len(), 0);

        let settings = build_settings(true, None);

        settings.save(&mut kv).await?;

        let after: Vec<String> = UserSettings::list_ids(&kv).await?;

        assert_eq!(after.len(), 1);
        assert_eq!(after[0], settings.api_key());

        Ok(())
    }

    #[tokio::test]
    async fn test_last_run_at() -> Result<()> {
        let settings = build_settings(true, None);
        let mut kv = MockKvStore::default();

        assert!(settings.last_run_at(&kv).await? < Utc::now() - Duration::days(6));

        settings.touch_last_run_at(&mut kv).await?;

        assert!(settings.last_run_at(&kv).await? > Utc::now() - Duration::hours(1));
        Ok(())
    }
}
