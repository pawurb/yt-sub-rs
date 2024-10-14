use chrono::{DateTime, Duration, Utc};
use eyre::{eyre, Result};
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{channel::Channel, notifier::Notifier};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UserSettings {
    pub channels: Vec<Channel>,
    pub notifiers: Vec<Notifier>,
    #[serde(skip_serializing, skip_deserializing)]
    pub path: PathBuf,
    pub last_run_at_path: PathBuf,
}

impl Display for UserSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let toml_file = toml::to_string(self).expect("Failed to serialize TOML");
        write!(f, "{}\n\n{}", self.path.display(), toml_file)
    }
}

impl UserSettings {
    fn default(path: PathBuf) -> Self {
        Self {
            path,
            notifiers: vec![Notifier::default()],
            channels: vec![],
            last_run_at_path: home_dir().unwrap().join(".yt-sub-rs/last_run_at.txt"),
        }
    }

    pub fn init(path: Option<&PathBuf>) -> Result<Self> {
        let default_path = Self::default_path();
        let path = path.unwrap_or(&default_path);
        if Path::new(path).exists() {
            return Err(eyre!(
                "Config file at '{}' is already initialized!",
                path.display()
            ));
        }

        let settings = Self::default(path.clone());
        settings.sync(Some(path))?;

        Ok(settings)
    }

    pub fn read(path: Option<&PathBuf>) -> Result<Self> {
        let default_path = Self::default_path();
        let path = path.unwrap_or(&default_path);

        if !Path::new(path).exists() {
            return Err(eyre!("Config file at '{}' does not exist!", path.display()));
        }
        let mut settings: Self = toml::from_str(&std::fs::read_to_string(path)?)?;
        settings.path = path.clone();
        Ok(settings)
    }

    pub fn sync(&self, path: Option<&PathBuf>) -> Result<()> {
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

    pub fn default_path() -> PathBuf {
        home_dir().unwrap().join(".config/yt-sub-rs/config.toml")
    }

    pub fn get_last_run_at(&self) -> DateTime<Utc> {
        if Path::new(&self.last_run_at_path).exists() {
            let last_run_at = std::fs::read_to_string(&self.last_run_at_path)
                .expect("Failed to read last_run_at file");
            DateTime::parse_from_rfc3339(&last_run_at)
                .expect("Failed to parse last_run_at file")
                .with_timezone(&Utc)
        } else {
            Utc::now() - Duration::days(7)
        }
    }

    pub fn update_last_run_at(&self) -> Result<()> {
        let last_run_at = Utc::now().to_rfc3339();
        if let Some(parent) = Path::new(&self.last_run_at_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&self.last_run_at_path)?;
        file.write_all(last_run_at.as_bytes())?;
        Ok(())
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
}

#[cfg(test)]
mod tests {
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

        let run_at_path = home_dir().unwrap().join(".yt-sub-rs/other_last_run_at.txt");

        let settings = UserSettings {
            last_run_at_path: run_at_path.clone(),
            ..settings
        };

        settings.sync(Some(&path))?;

        let updated = UserSettings::read(Some(&path))?;

        assert_eq!(updated.last_run_at_path, run_at_path);

        Ok(())
    }
}
