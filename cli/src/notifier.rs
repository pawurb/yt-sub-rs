use eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::logger::Logger;

#[non_exhaustive]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Notifier {
    Log(LogConfig),
    Slack(SlackConfig),
    Telegram,
}

impl Default for Notifier {
    fn default() -> Self {
        Self::Log(LogConfig { notify: true })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct LogConfig {
    notify: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct SlackConfig {
    webhook_url: String,
    channel: String,
}

impl Notifier {
    pub async fn notify(&self, messages: Vec<String>, cron: bool) -> Result<()> {
        match self {
            Notifier::Log(log_config) => {
                if !log_config.notify {
                    return Ok(());
                }

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

    let _ = client.post(&config.webhook_url).json(&payload).send().await;

    Ok(())
}
