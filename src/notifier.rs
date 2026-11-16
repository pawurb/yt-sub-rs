use eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::logger::Logger;

#[non_exhaustive]
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum Notifier {
    Log(),
    Slack(SlackConfig),
    Telegram,
}

impl Default for Notifier {
    fn default() -> Self {
        Self::Log()
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct SlackConfig {
    pub webhook_url: String,
    pub channel: String,
}

impl Notifier {
    pub async fn notify(&self, messages: Vec<String>, cron: bool) -> Result<()> {
        match self {
            Notifier::Log() => {
                let logger = Logger::new(cron);
                for message in messages {
                    logger.info(&message);
                }
                Ok(())
            }
            Notifier::Slack(slack_config) => {
                notify_slack(&messages.join("\n\n"), slack_config).await?;
                Ok(())
            }
            Notifier::Telegram => todo!(),
        }
    }
}

async fn notify_slack(message: &str, config: &SlackConfig) -> Result<()> {
    let client = Client::new();

    let payload = json!({
        "channel": config.channel,
        "icon_emoji": ":exclamation:",
        "username": "yt-sub-rs",
        "text": message,
        "unfurl_links": false,
    });

    let res = client
        .post(&config.webhook_url)
        .json(&payload)
        .send()
        .await?;

    if res.status() == 200 {
        return Ok(());
    }

    let err_msg = res.text().await?;
    eyre::bail!("Failed to send message to Slack: {err_msg}");
}

#[cfg(test)]
mod tests {
    use mockito::{Matcher, Server};
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_notify_slack() -> Result<()> {
        let mut server = Server::new_async().await;
        let notifier = Notifier::Slack(SlackConfig {
            webhook_url: format!("{}/services/XXX", server.url()),
            channel: "yt-videos".to_string(),
        });

        let m = server
            .mock("POST", "/services/XXX")
            .match_body(Matcher::PartialJson(json!({
                "channel": "yt-videos",
                "text": "first\n\nsecond",
            })))
            .with_status(200)
            .create_async()
            .await;

        notifier
            .notify(vec!["first".to_string(), "second".to_string()], false)
            .await?;

        m.assert_async().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_notify_slack_error() -> Result<()> {
        let mut server = Server::new_async().await;
        let notifier = Notifier::Slack(SlackConfig {
            webhook_url: format!("{}/services/XXX", server.url()),
            channel: "yt-videos".to_string(),
        });

        let m = server
            .mock("POST", "/services/XXX")
            .with_status(404)
            .with_body("no_service")
            .create_async()
            .await;

        let err = notifier
            .notify(vec!["msg".to_string()], false)
            .await
            .expect_err("Slack error should propagate");
        assert!(err.to_string().contains("no_service"));

        m.assert_async().await;
        Ok(())
    }
}
