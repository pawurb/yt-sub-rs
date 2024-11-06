use chrono::{DateTime, Duration, Utc};
use eyre::{OptionExt, Result};
use sqlx::SqlitePool;
use yt_sub_core::UserSettings;

use crate::lite_helpers::UserRow;

#[allow(async_fn_in_trait)]
pub trait UserSettingsAPI {
    fn api_key(&self) -> String;
    async fn read(api_key: &str, conn: &SqlitePool) -> Result<UserSettings>;
    async fn last_run_at(&self, conn: &SqlitePool) -> Result<DateTime<Utc>>;
    async fn touch_last_run_at(self, conn: &SqlitePool) -> Result<()>;
    async fn ids(conn: &SqlitePool) -> Result<Vec<String>>;
    async fn save(&self, conn: &SqlitePool) -> Result<()>;
    async fn delete(api_key: &str, conn: &SqlitePool) -> Result<()>;
}

impl UserSettingsAPI for UserSettings {
    fn api_key(&self) -> String {
        self.api_key.clone().expect("Missing API key")
    }

    async fn read(api_key: &str, conn: &SqlitePool) -> Result<Self> {
        let row = UserRow::get(api_key, conn)
            .await?
            .ok_or_eyre("No settings found for user")?;

        let settings: UserSettings =
            serde_json::from_str(&row.settings_json).expect("Failed to parse settings");

        Ok(settings)
    }

    async fn delete(api_key: &str, conn: &SqlitePool) -> Result<()> {
        let exists = UserRow::exists(api_key, conn).await?;

        if !exists {
            eyre::bail!("API key not found")
        }

        UserRow::delete(api_key, conn).await?;

        Ok(())
    }

    async fn ids(conn: &SqlitePool) -> Result<Vec<String>> {
        UserRow::ids(conn).await
    }

    async fn save(&self, conn: &SqlitePool) -> Result<()> {
        if self.channels.len() > 100 {
            eyre::bail!("Too many channels!")
        }

        if self.notifiers.len() > 5 {
            eyre::bail!("Too many notifiers!")
        }

        let json = serde_json::to_string(&self)?;

        let user_row = UserRow {
            id: self.api_key(),
            settings_json: json,
            last_run_at: None,
        };
        user_row.save(conn).await?;

        Ok(())
    }

    async fn last_run_at(&self, conn: &SqlitePool) -> Result<DateTime<Utc>> {
        let row = UserRow::get(&self.api_key(), conn)
            .await?
            .ok_or_eyre("No settings found for user")?;

        Ok(row
            .last_run_at
            .unwrap_or_else(|| Utc::now() - Duration::days(7)))
    }

    async fn touch_last_run_at(self, conn: &SqlitePool) -> Result<()> {
        let updated = UserRow {
            id: self.api_key(),
            settings_json: serde_json::to_string(&self)?,
            last_run_at: Some(Utc::now()),
        };
        updated.save(conn).await?;

        Ok(())
    }
}

#[cfg(test)]

mod tests {
    use crate::{controllers::account::tests::build_settings, lite_helpers::tests::setup_test_db};
    use UserSettingsAPI;

    use super::*;

    #[tokio::test]
    async fn test_add_list_users() -> Result<()> {
        let (conn, _cl) = setup_test_db().await;

        let before: Vec<String> = UserSettings::ids(&conn).await?;
        assert_eq!(before.len(), 0);

        let settings = build_settings(true, None);

        settings.save(&conn).await?;

        let after = UserSettings::ids(&conn).await?;

        assert_eq!(after.len(), 1);
        assert_eq!(after[0], settings.api_key());

        Ok(())
    }

    #[tokio::test]
    async fn test_last_run_at() -> Result<()> {
        let (conn, _cl) = setup_test_db().await;

        let settings = build_settings(true, None);
        settings.save(&conn).await?;

        assert!(settings.last_run_at(&conn).await? < Utc::now() - Duration::days(6));

        let api_key = settings.api_key();
        settings.touch_last_run_at(&conn).await?;

        let settings = UserSettings::read(&api_key, &conn).await?;

        assert!(settings.last_run_at(&conn).await? > Utc::now() - Duration::hours(1));
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_user_settings() -> Result<()> {
        let (conn, _cl) = setup_test_db().await;

        let settings = build_settings(true, None);
        settings.save(&conn).await?;

        let before = UserSettings::ids(&conn).await?;
        assert_eq!(before.len(), 1);

        UserSettings::delete(&settings.api_key(), &conn).await?;

        let after = UserSettings::ids(&conn).await?;
        assert_eq!(after.len(), 0);

        Ok(())
    }
}
