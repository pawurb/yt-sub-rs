use eyre::Result;
use uuid::Uuid;
use worker::kv::KvStore;
use yt_sub_core::UserSettings;

pub async fn register_user<T: KvWrapper>(settings: UserSettings, kv: &mut T) -> Result<String> {
    if let Some(api_key) = settings.api_key {
        if kv.get_val(&api_key).await?.is_some() {
            eyre::bail!("Already registered with API key: {}", api_key)
        } else {
            eyre::bail!(
                "Invalid API key present: '{}'. Please remove it and try again",
                api_key
            );
        }
    }

    let notifier = match settings.get_slack_notifier() {
        Some(notifier) => notifier,
        None => {
            eyre::bail!("Missing Slack notifier settings");
        }
    };

    notifier
        .notify(vec!["Registering remote account.".to_string()], false)
        .await
        .map_err(|e| {
            eyre::eyre!(
                "Invalid Slack notifier settings. Sending message failed: {}",
                e
            )
        })?;

    let api_key = Uuid::new_v4().to_string();

    let settings = UserSettings {
        api_key: Some(api_key.clone()),
        ..settings
    };

    let settings_json = serde_json::to_string(&settings)?;
    kv.put_val(&api_key, &settings_json).await?;

    Ok(api_key.to_string())
}

pub trait KvWrapper {
    async fn put_val(&mut self, key: &str, value: &str) -> Result<()>;
    async fn get_val(&self, key: &str) -> Result<Option<String>>;
}

impl KvWrapper for KvStore {
    async fn put_val(&mut self, key: &str, value: &str) -> Result<()> {
        self.put(key, value)
            .map_err(|e| eyre::eyre!("Failed to put key: {}", e))?
            .execute()
            .await
            .or_else(|e| eyre::bail!("Failed to put key: {}", e))?;

        Ok(())
    }

    async fn get_val(&self, key: &str) -> Result<Option<String>> {
        let res = match self.get(key).text().await {
            Ok(value) => value,
            Err(e) => {
                eyre::bail!("Failed to get key: {}", e)
            }
        };

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    struct MockKvStore {
        pub store: HashMap<String, String>,
    }

    impl MockKvStore {
        pub fn new() -> Self {
            Self {
                store: HashMap::new(),
            }
        }
    }

    impl KvWrapper for MockKvStore {
        async fn put_val(&mut self, key: &str, value: &str) -> Result<()> {
            self.store.insert(key.to_string(), value.to_string());
            Ok(())
        }

        async fn get_val(&self, key: &str) -> Result<Option<String>> {
            Ok(self.store.get(key).map(|v| v.to_string()))
        }
    }

    use mockito::Server;
    use std::{collections::HashMap, path::PathBuf};
    use yt_sub_core::notifier::{Notifier, SlackConfig};

    use super::*;
    #[tokio::test]
    async fn test_happy_path() -> Result<()> {
        let mut server = Server::new_async().await;
        let host = server.host_with_port();
        let host = format!("http://{}", host);
        let m = server
            .mock("POST", "/slack_webhook")
            .with_body("OK")
            .with_status(200)
            .create_async()
            .await;

        let settings = build_settings(false, Some(format!("{}/slack_webhook", host)));
        let mut kv = MockKvStore::new();

        let api_key = register_user(settings, &mut kv)
            .await
            .expect("Failed to register user");

        let settings_json = kv.get_val(&api_key).await.unwrap().unwrap();

        let _settings: UserSettings =
            serde_json::from_str(&settings_json).expect("Failed to parse settings JSON");

        m.assert_async().await;

        Ok(())
    }

    #[tokio::test]
    async fn test_kv_wrapper() -> Result<()> {
        let mut kv = MockKvStore::new();

        let empty = kv.get_val("test").await?;
        assert!(empty.is_none());

        kv.put_val("test_key", "test_val").await?;

        let present = kv.get_val("test_key").await?;
        assert_eq!(present, Some("test_val".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_api_key() -> Result<()> {
        let settings = build_settings(true, None);
        let mut kv = MockKvStore::new();

        if let Err(e) = register_user(settings, &mut kv).await {
            let error_message = e.to_string();

            assert!(error_message.contains("Invalid API key present"));
        } else {
            panic!("Expected an error, but got a success result");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_registered_api_key() -> Result<()> {
        let settings = build_settings(true, None);
        let mut kv = MockKvStore::new();

        kv.put_val(&settings.api_key.clone().unwrap(), "true")
            .await?;

        if let Err(e) = register_user(settings, &mut kv).await {
            let error_message = e.to_string();

            assert!(error_message.contains("Already registered"));
        } else {
            panic!("Expected an error, but got a success result");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_slack_not_configured() -> Result<()> {
        let settings = build_settings(false, None);
        let mut kv = MockKvStore::new();

        if let Err(e) = register_user(settings, &mut kv).await {
            let error_message = e.to_string();

            assert!(error_message.contains("Missing Slack notifier settings"));
        } else {
            panic!("Expected an error, but got a success result");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_slack_notification_failed() -> Result<()> {
        let mut server = Server::new_async().await;
        let host = server.host_with_port();
        let host = format!("http://{}", host);
        let m = server
            .mock("POST", "/slack_webhook")
            .with_body("Invalid slack webhook URL")
            .with_status(401)
            .create_async()
            .await;

        let settings = build_settings(false, Some(format!("{}/slack_webhook", host)));
        let mut kv = MockKvStore::new();

        if let Err(e) = register_user(settings, &mut kv).await {
            let error_message = e.to_string();

            assert!(error_message.contains("Invalid slack webhook URL"));
        } else {
            panic!("Expected an error, but got a success result");
        }

        m.assert_async().await;

        Ok(())
    }

    fn build_settings(with_api_key: bool, slack_webhook: Option<String>) -> UserSettings {
        let settings = UserSettings::default(PathBuf::from("test.toml"));

        let settings = if with_api_key {
            UserSettings {
                api_key: Some("test".to_string()),
                ..settings
            }
        } else {
            settings
        };

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
