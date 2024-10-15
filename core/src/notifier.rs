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

    pub fn is_slack(&self) -> bool {
        matches!(self, Notifier::Slack(_))
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
