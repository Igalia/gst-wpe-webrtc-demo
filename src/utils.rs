use std::path::PathBuf;

use crate::settings::Settings;
use crate::APPLICATION_NAME;

// Get the default path for the settings file
pub fn get_settings_file_path() -> PathBuf {
    let mut path = glib::get_user_config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(APPLICATION_NAME);
    path.push("settings.toml");
    path
}

// Load the current settings
pub fn load_settings() -> Settings {
    let s = get_settings_file_path();
    if s.exists() && s.is_file() {
        match serde_any::from_file::<Settings, _>(&s) {
            Ok(s) => s,
            Err(e) => {
                panic!("Error while opening '{}': {}", s.display(), e);
            }
        }
    } else {
        Settings::default()
    }
}
