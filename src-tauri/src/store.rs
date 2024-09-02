use std::fs;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub folders_to_watch: Vec<String>,
    pub album_artwork_filenames: Vec<String>,
    pub mini_player_location: MiniPlayerLocation,
    pub ai_features_enabled: bool,
    pub llm: LLM,
    pub open_ai_api_key: Option<String>,
    pub genius_api_key: Option<String>,
    pub is_artists_toolkit_enabled: bool,
    pub download_location: Option<String>,
    pub theme: String,
    pub output_device: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum MiniPlayerLocation {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LLM {
    #[serde(rename = "gpt-3.5-turbo")]
    Gpt35Turbo,
    #[serde(rename = "gpt-4")]
    Gpt4,
    #[serde(rename = "ollama")]
    Ollama,
}


pub fn load_settings(app: &AppHandle) -> Result<UserSettings, anyhow::Error> {
    let config_dir = app.path().app_config_dir().unwrap();
    // join the config dir with the path to the settings file
    let settings_path = config_dir.join("settings.json");
    let settings_data = fs::read_to_string(settings_path)?;
    let settings: UserSettings = serde_json::from_str(&settings_data)?;
    Ok(settings)
}