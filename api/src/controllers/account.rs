use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
    Json,
};
use eyre::Result;
use reqwest::StatusCode;
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;
use yt_sub_core::UserSettings;

use crate::{
    config::routes::{invalid_req, AppState},
    lite_helpers::UserRow,
    user_settings_api::UserSettingsAPI,
};

pub async fn update(
    State(state): State<AppState>,
    Json(settings): Json<UserSettings>,
) -> impl IntoResponse {
    let conn = &state.conn.clone();
    let Some(api_key) = settings.api_key.clone() else {
        return invalid_req("Missing API key!");
    };

    let exists = match UserRow::exists(&api_key, conn).await {
        Ok(exists) => exists,
        Err(e) => {
            return invalid_req(&e.to_string());
        }
    };

    if !exists {
        return invalid_req("Invalid API key!");
    }

    if settings.get_slack_notifier().is_none() {
        return invalid_req("Missing Slack notifier settings");
    }

    match settings.save(conn).await {
        Ok(_) => {}
        Err(e) => {
            return invalid_req(&e.to_string());
        }
    }

    "UPDATED".into_response()
}

pub async fn delete(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let conn = &state.conn.clone();
    let api_key = match headers.get("X-API-KEY") {
        Some(api_key) => api_key.to_str().unwrap(),
        None => return invalid_req("Missing X-API-KEY header"),
    };

    match UserSettings::read(api_key, conn).await {
        Ok(_) => {}
        Err(e) => {
            return invalid_req(&e.to_string());
        }
    }

    // TODO macro
    match UserSettings::delete(api_key, conn).await {
        Ok(_) => {}
        Err(e) => {
            return invalid_req(&e.to_string());
        }
    }

    "DELETED".into_response()
}

pub async fn create(
    State(state): State<AppState>,
    Json(settings): Json<UserSettings>,
) -> impl IntoResponse {
    let conn = &state.conn.clone();
    let response = match create_impl(settings, conn).await {
        Ok(response) => response,
        Err(e) => return invalid_req(&e.to_string()).into_response(),
    };

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    (StatusCode::CREATED, headers, response).into_response()
}

async fn create_impl(settings: UserSettings, conn: &SqlitePool) -> Result<String> {
    if let Some(api_key) = settings.api_key {
        if UserRow::exists(&api_key, conn).await? {
            eyre::bail!("Already registered with this API key");
        } else {
            eyre::bail!("Invalid API key present. Please remove it and try again.");
        }
    }

    let notifier = match settings.get_slack_notifier() {
        Some(notifier) => notifier,
        None => {
            eyre::bail!("Missing Slack notifier settings");
        }
    };

    notifier
        .notify(
            vec![
                "Registered remote account. You'll receive notifications about new videos."
                    .to_string(),
            ],
            false,
        )
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

    settings.save(conn).await?;

    let response = json!({
        "api_key": api_key,
    });

    Ok(response.to_string())
}

#[cfg(test)]
pub mod tests {
    use mockito::Server;
    use serde_json::Value;
    use std::path::PathBuf;
    use yt_sub_core::notifier::{Notifier, SlackConfig};

    use crate::lite_helpers::tests::setup_test_db;

    use super::*;
    #[tokio::test]
    async fn create_happy_path() -> Result<()> {
        let (conn, _cl) = setup_test_db().await;

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
        let json = create_impl(settings, &conn)
            .await
            .expect("Failed to register user");

        let json: Value = serde_json::from_str(&json).expect("Failed to parse JSON");
        let api_key = json["api_key"].as_str().unwrap();

        let exists = UserRow::exists(api_key, &conn).await?;
        assert!(exists);

        m.assert_async().await;

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_api_key() -> Result<()> {
        let (conn, _cl) = setup_test_db().await;
        let settings = build_settings(true, None);

        if let Err(e) = create_impl(settings, &conn).await {
            assert!(e.to_string().contains("Invalid API key present"));
        } else {
            panic!("Expected an error!");
        }

        Ok(())
    }

    #[tokio::test]
    async fn create_slack_not_configured() -> Result<()> {
        let (conn, _cl) = setup_test_db().await;
        let settings = build_settings(false, None);

        if let Err(e) = create_impl(settings, &conn).await {
            assert!(e.to_string().contains("Missing Slack notifier settings"));
        } else {
            panic!("Expected an error!");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_slack_notification_failed() -> Result<()> {
        let (conn, _cl) = setup_test_db().await;

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

        if let Err(e) = create_impl(settings, &conn).await {
            assert!(e.to_string().contains("Invalid slack webhook URL"));
        } else {
            panic!("Expected an error!");
        }

        m.assert_async().await;

        Ok(())
    }

    pub fn build_settings(with_api_key: bool, slack_webhook: Option<String>) -> UserSettings {
        let settings = UserSettings::default(PathBuf::from("test.toml"));

        let settings = if with_api_key {
            UserSettings {
                api_key: Some(Uuid::new_v4().to_string()),
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
