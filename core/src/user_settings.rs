use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use crate::{
    channel::Channel,
    notifier::{Notifier, SlackConfig},
};
pub const API_HOST: &str = "https://yt-sub-api.apki.workers.dev";

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UserSettings {
    pub channels: Vec<Channel>,
    pub notifiers: Vec<Notifier>,
    pub api_key: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    pub path: PathBuf,
}

impl Display for UserSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let toml_file = toml::to_string(self).expect("Failed to serialize TOML");
        write!(f, "{}\n\n{}", self.path.display(), toml_file)
    }
}

impl UserSettings {
    pub fn default(path: PathBuf) -> Self {
        Self {
            path,
            notifiers: vec![Notifier::default()],
            channels: vec![],
            api_key: None,
        }
    }

    pub fn get_channel_by_id(&self, channel_id: &str) -> Option<Channel> {
        self.channels
            .iter()
            .find(|channel| channel.channel_id == channel_id)
            .cloned()
    }

    pub fn get_channel_by_handle(&self, handle: &str) -> Option<Channel> {
        self.channels
            .iter()
            .find(|channel| channel.handle == handle)
            .cloned()
    }

    pub fn get_slack_config(&self) -> Option<&SlackConfig> {
        let notifier = self.notifiers.iter().find(|n| n.is_slack());

        match notifier {
            Some(Notifier::Slack(config)) => Some(config),
            _ => None,
        }
    }
}
