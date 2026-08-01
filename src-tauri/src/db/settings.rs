//! The single settings schema described in `design.md` ("Settings as a
//! single-row table, not a KV store") — one place the shape is defined, so
//! the frontend never parses an independent copy that can drift from this
//! one (the `store.rs` `UserSettings` vs. `App.d.ts` `UserSettings` problem
//! this change replaces).

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)]
    pub folders_to_watch: Vec<String>,
    #[serde(default)]
    pub album_artwork_filenames: Vec<String>,
    #[serde(default = "default_mini_player_location")]
    pub mini_player_location: String,
    #[serde(default)]
    pub is_artists_toolkit_enabled: bool,
    #[serde(default)]
    pub is_cover_full_check_enabled: bool,
    #[serde(default)]
    pub scrapbook_location: Option<String>,
    #[serde(default)]
    pub songbook_location: Option<String>,
    #[serde(default)]
    pub download_location: Option<String>,
    #[serde(default)]
    pub playlists_location: Option<String>,
    #[serde(default)]
    pub generated_stems_location: Option<String>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_true")]
    pub follow_system_theme: bool,
    #[serde(default)]
    pub output_device: Option<String>,
    #[serde(default = "default_true")]
    pub follow_system_output: bool,
    #[serde(default)]
    pub genius_api_key: Option<String>,
    #[serde(default)]
    pub discogs_api_key: Option<String>,
    #[serde(default)]
    pub beets_db_location: Option<String>,
    /// Set once the one-time Dexie → SQLite migration (`db::migration`) has
    /// cut over. `None` means "not yet migrated" — the frontend's signal to
    /// still trust its Dexie data over this database.
    #[serde(default)]
    pub migrated_from_dexie_at: Option<i64>,
}

fn default_mini_player_location() -> String {
    "bottom-right".to_string()
}

fn default_theme() -> String {
    "auto".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        // Deserializing an empty object exercises the same `#[serde(default)]`
        // paths a real old-row-missing-a-field read would, instead of a second,
        // separately-maintained list of defaults.
        serde_json::from_str("{}").expect("Settings must be constructible from {}")
    }
}

pub fn get_settings(conn: &Connection) -> rusqlite::Result<Settings> {
    let data: Option<String> = conn
        .query_row("SELECT data FROM settings WHERE id = 1", [], |row| row.get(0))
        .optional()?;
    Ok(match data {
        Some(json) => serde_json::from_str(&json).unwrap_or_default(),
        None => Settings::default(),
    })
}

pub fn update_settings(conn: &Connection, settings: &Settings) -> rusqlite::Result<()> {
    let json = serde_json::to_string(settings).expect("Settings always serializes");
    conn.execute(
        "INSERT INTO settings (id, data) VALUES (1, ?1)
         ON CONFLICT(id) DO UPDATE SET data = excluded.data",
        [json],
    )?;
    Ok(())
}
