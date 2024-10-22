use eyre::{OptionExt, Result};
use reqwest::Client;
use serde_json::Value;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Duration, Utc};
use home::home_dir;
use yt_sub_core::{user_settings::API_HOST, UserSettings};

#[allow(async_fn_in_trait)]
pub trait UserSettingsCLI {
    fn last_run_at(&self) -> DateTime<Utc>;
    fn touch_last_run_at(&self) -> Result<()>;
    fn init(path: Option<&PathBuf>) -> Result<UserSettings>;
    fn read(path: Option<&PathBuf>) -> Result<UserSettings>;
    fn save(&self, path: Option<&PathBuf>) -> Result<()>;
    fn default_path() -> PathBuf;
    async fn create_account(self, host: Option<&str>) -> Result<()>;
    async fn delete_account(&self, host: Option<&str>) -> Result<()>;
}

impl UserSettingsCLI for UserSettings {
    fn last_run_at(&self) -> DateTime<Utc> {
        let path = home_dir().unwrap().join(".yt-sub-rs/last_run_at.txt");
        if Path::new(&path).exists() {
            let last_run_at =
                std::fs::read_to_string(&path).expect("Failed to read last_run_at file");
            DateTime::parse_from_rfc3339(&last_run_at)
                .expect("Failed to parse last_run_at file")
                .with_timezone(&Utc)
        } else {
            Utc::now() - Duration::days(7)
        }
    }

    fn touch_last_run_at(&self) -> Result<()> {
        let last_run_at_path = last_run_at_path();
        let last_run_at = Utc::now().to_rfc3339();
        if let Some(parent) = Path::new(&last_run_at_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = File::create(last_run_at_path)?;
        file.write_all(last_run_at.as_bytes())?;
        Ok(())
    }

    fn init(path: Option<&PathBuf>) -> Result<Self> {
        let default_path = Self::default_path();
        let path = path.unwrap_or(&default_path);
        if Path::new(path).exists() {
            eyre::bail!(
                "Config file at '{}' is already initialized!",
                path.display()
            );
        }

        let settings = Self::default(path.clone());
        settings.save(Some(path))?;

        Ok(settings)
    }

    fn read(path: Option<&PathBuf>) -> Result<Self> {
        let default_path = Self::default_path();
        let path = path.unwrap_or(&default_path);

        if !Path::new(path).exists() {
            eyre::bail!(
                "Config file at '{}' does not exist! Run 'ytsub init' to initialize it.",
                path.display()
            )
        }
        let mut settings: Self = toml::from_str(&std::fs::read_to_string(path)?)?;
        settings.path = path.clone();
        Ok(settings)
    }

    fn save(&self, path: Option<&PathBuf>) -> Result<()> {
        let res = toml::to_string(self).expect("Failed to serialize TOML");

        let default_path = Self::default_path();
        let path = path.unwrap_or(&default_path);

        if let Some(parent) = Path::new(&path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = File::create(path).expect("Failed to create file");
        file.write_all(res.as_bytes())
            .expect("Failed to write to file");
        Ok(())
    }

    fn default_path() -> PathBuf {
        home_dir().unwrap().join(".config/yt-sub-rs/config.toml")
    }

    async fn create_account(self, host: Option<&str>) -> Result<()> {
        if self.api_key.is_some() {
            eyre::bail!("Remote account is already registered.")
        }

        _ = self.get_slack_notifier().ok_or_eyre(
            "You must configure a Slack notifier to register a remote account:
https://github.com/pawurb/yt-sub-rs#notifiers-configuration",
        )?;

        let client = Client::new();
        let host = host.unwrap_or(API_HOST);

        let res = client
            .post(format!("{}/account", host))
            .json(&self)
            .send()
            .await?;

        if res.status() != 201 {
            let err_msg = res.text().await?;
            eyre::bail!("Failed to register remote account: {err_msg}")
        }

        let res_json: Value = res.json().await?;
        let remote_api_key = res_json["api_key"].as_str().unwrap().to_string();

        let config_path = self.path.clone();

        let settings = Self {
            api_key: Some(remote_api_key),
            ..self
        };

        settings.save(Some(&config_path))?;

        Ok(())
    }

    async fn delete_account(&self, host: Option<&str>) -> Result<()> {
        if self.api_key.is_none() {
            eyre::bail!("Remote account is not registered.")
        }

        let client = Client::new();
        let host = host.unwrap_or(API_HOST);

        let res = client
            .delete(format!("{}/account", host))
            .header("X-API-KEY", self.api_key.clone().unwrap())
            .send()
            .await?;

        if !res.status().is_success() {
            let err_msg = res.text().await?;
            eyre::bail!("Failed to delete remote account: {err_msg}")
        }

        Ok(())
    }
}

fn last_run_at_path() -> PathBuf {
    home_dir().unwrap().join(".yt-sub-rs/last_run_at.txt")
}

#[cfg(test)]
mod tests {
    use mockito::Server;
    use yt_sub_core::{
        channel::Channel,
        notifier::{Notifier, SlackConfig},
    };

    use crate::test_helpers::{test_config_path, Cleaner};

    use super::*;

    #[tokio::test]
    async fn test_init_config_file() -> Result<()> {
        let path = test_config_path();
        let _cl = Cleaner { path: path.clone() };

        let settings = UserSettings::init(Some(&path))?;
        assert_eq!(settings, UserSettings::default(path));

        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn test_init_twice() {
        let path = test_config_path();
        let _cl = Cleaner { path: path.clone() };

        UserSettings::init(Some(&path)).expect("1st should not panic");
        UserSettings::init(Some(&path)).expect("2nd should panic");
    }

    #[tokio::test]
    async fn test_sync_settings_file() -> Result<()> {
        let path = test_config_path();
        let _cl = Cleaner { path: path.clone() };
        let settings = UserSettings::init(Some(&path))?;

        assert_eq!(settings.channels.len(), 0);

        let channel = Channel {
            channel_id: "CHANNEL_ID".to_string(),
            handle: "CHANNEL_HANDLE".to_string(),
            description: "CHANNEL_DESC".to_string(),
        };

        let mut channels = settings.channels.clone();
        channels.extend(vec![channel]);

        let settings = UserSettings {
            channels,
            ..settings
        };

        settings.save(Some(&path))?;

        let updated = UserSettings::read(Some(&path))?;
        assert_eq!(updated.channels.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_account_ok() -> Result<()> {
        let mut server = Server::new_async().await;
        let host = server.host_with_port();
        let host = format!("http://{}", host);

        let path = test_config_path();
        let _cl = Cleaner { path: path.clone() };

        let settings = build_settings(
            Some(path.clone()),
            Some("https://slack.com/XXX".to_string()),
        );

        let settings = UserSettings {
            api_key: Some("test".to_string()),
            ..settings
        };

        let m = server
            .mock("DELETE", "/account")
            .match_header("X-API-KEY", "test")
            .with_body("OK")
            .create_async()
            .await;

        settings.delete_account(Some(&host)).await?;
        m.assert_async().await;

        Ok(())
    }

    #[tokio::test]
    async fn test_create_account_ok() -> Result<()> {
        let mut server = Server::new_async().await;
        let host = server.host_with_port();
        let host = format!("http://{}", host);

        let path = test_config_path();
        let _cl = Cleaner { path: path.clone() };

        let settings = build_settings(
            Some(path.clone()),
            Some("https://slack.com/XXX".to_string()),
        );

        let m = server
            .mock("POST", "/account")
            .with_body(r#"{"api_key": "REMOTE_API_KEY" }"#)
            .with_status(201)
            .create_async()
            .await;

        settings.create_account(Some(&host)).await?;
        m.assert_async().await;

        let settings = UserSettings::read(Some(&path))?;

        assert_eq!(settings.api_key, Some("REMOTE_API_KEY".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_create_account_invalid() -> Result<()> {
        let mut server = Server::new_async().await;
        let host = server.host_with_port();
        let host = format!("http://{}", host);
        let m = server
            .mock("POST", "/account")
            .with_body(r#"Registration failed"#)
            .with_status(400)
            .create_async()
            .await;

        let settings = build_settings(None, Some("https://slack.com/XXX".to_string()));

        if let Err(e) = settings.create_account(Some(&host)).await {
            assert!(e.to_string().contains("Registration failed"));
        } else {
            panic!("Expected an error!");
        }

        m.assert_async().await;
        Ok(())
    }

    fn build_settings(path: Option<PathBuf>, slack_webhook: Option<String>) -> UserSettings {
        let path = path.unwrap_or(test_config_path());
        let settings = UserSettings::default(path);

        if let Some(webhook) = slack_webhook {
            let notifier = Notifier::Slack(SlackConfig {
                webhook_url: webhook,
                channel: "test".to_string(),
            });

            UserSettings {
                notifiers: vec![notifier],
                ..settings
            }
        } else {
            settings
        }
    }
}
