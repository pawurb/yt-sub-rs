use std::{
    fs,
    path::{Path, PathBuf},
};

use home::home_dir;
use uuid::Uuid;

use crate::UserSettings;

pub fn test_config_path() -> PathBuf {
    let uuid = Uuid::new_v4();
    home_dir()
        .unwrap()
        .join(format!(".config/yt-sub-rs-test/config-{}.toml", uuid))
}

pub fn init_test_settings() -> (UserSettings, Cleaner) {
    let path = test_config_path();
    let cl = Cleaner { path: path.clone() };
    (UserSettings::init(Some(&path)).unwrap(), cl)
}

pub struct Cleaner {
    pub path: PathBuf,
}

impl Drop for Cleaner {
    fn drop(&mut self) {
        if !Path::new(&self.path).exists() {
            return;
        }

        fs::remove_file(&self.path).expect("Failed to remove file");
    }
}
