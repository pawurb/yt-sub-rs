use eyre::Result;
use uuid::Uuid;
use yt_sub_core::UserSettings;

use crate::{store::KvWrapper, users::User};

pub async fn register_user(settings: UserSettings, kv: &mut impl KvWrapper) -> Result<String> {
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

    let users = User::list(kv).await?;

    let user_ids: Vec<&str> = users.iter().map(|u| u.api_key.as_str()).collect();

    if user_ids.contains(&api_key.as_str()) {
        panic!("It should never happen!");
    } else {
        let new_user = User::new(&api_key);
        User::save(&new_user, kv).await?;
    }

    Ok(api_key.to_string())
}

#[cfg(test)]
mod tests {
    use mockito::Server;
    use std::path::PathBuf;
    use yt_sub_core::notifier::{Notifier, SlackConfig};

    use crate::store::tests::MockKvStore;

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
        let mut kv = MockKvStore::default();

        let api_key = register_user(settings, &mut kv)
            .await
            .expect("Failed to register user");

        let settings_json = kv.get_val(&api_key).await.unwrap().unwrap();

        let _settings: UserSettings =
            serde_json::from_str(&settings_json).expect("Failed to parse settings JSON");

        let ids = kv.get_val("user_ids").await.unwrap().unwrap();
        assert_eq!(ids, api_key);

        m.assert_async().await;

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_api_key() -> Result<()> {
        let settings = build_settings(true, None);
        let mut kv = MockKvStore::default();

        if let Err(e) = register_user(settings, &mut kv).await {
            assert!(e.to_string().contains("Invalid API key present"));
        } else {
            panic!("Expected an error!");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_registered_api_key() -> Result<()> {
        let settings = build_settings(true, None);
        let mut kv = MockKvStore::default();

        kv.put_val(&settings.api_key.clone().unwrap(), "true")
            .await?;

        if let Err(e) = register_user(settings, &mut kv).await {
            assert!(e.to_string().contains("Already registered"));
        } else {
            panic!("Expected an error!");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_slack_not_configured() -> Result<()> {
        let settings = build_settings(false, None);
        let mut kv = MockKvStore::default();

        if let Err(e) = register_user(settings, &mut kv).await {
            assert!(e.to_string().contains("Missing Slack notifier settings"));
        } else {
            panic!("Expected an error!");
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
        let mut kv = MockKvStore::default();

        if let Err(e) = register_user(settings, &mut kv).await {
            assert!(e.to_string().contains("Invalid slack webhook URL"));
        } else {
            panic!("Expected an error!");
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
