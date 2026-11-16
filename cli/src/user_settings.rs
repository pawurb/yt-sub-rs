use eyre::{OptionExt, Result};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use crate::{channel::Channel, notifier::Notifier};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UserSettings {
    pub channels: Vec<Channel>,
    pub notifiers: Vec<Notifier>,
    pub youtube_api_key: Option<String>,
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
            youtube_api_key: None,
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

    pub fn youtube_api_key(&self) -> Result<&str> {
        self.youtube_api_key.as_deref().ok_or_eyre(
            "Missing 'youtube_api_key' in the config file. It is required to look up a channel by its handle:
https://github.com/pawurb/yt-sub-rs#youtube-api-key

Alternatively, provide --channel-id and --desc manually.",
        )
    }
}
