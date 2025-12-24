use log::info;
use rusqlite::{Connection, OpenFlags, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tauri::Manager;

use crate::{
    metadata::{Album, FileInfo, Song},
    store::load_settings,
};

#[derive(Serialize, Deserialize)]
pub struct BeetsTrack {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub path: String,
}

#[tauri::command]
pub async fn search_beets(
    query: String,
    sort_by: Option<String>,
    descending: Option<bool>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<Song>, String> {
    info!("[Beets] Searching for {}", query);
    let db_path =
        get_beets_db_path(app_handle).ok_or_else(|| "Beets database not found".to_string())?;
    let sort = sort_by.unwrap_or_else(|| "artist".to_string());
    let desc = descending.unwrap_or(false);
    let songs = query_beets_to_songs(&db_path, &query, &sort, desc).map_err(|e| e.to_string());
    if songs.is_err() {
        return Err(songs.unwrap_err());
    }
    info!("[Beets] Found {} results", songs.as_ref().unwrap().len());
    songs
}

pub fn get_beets_db_path(app_handle: tauri::AppHandle) -> Option<PathBuf> {
    // Logic to find the DB (check default paths or config)
    let settings = load_settings(&app_handle);
    if settings.is_err() {
        return None;
    }
    if let Some(path) = settings.unwrap().beets_db_location {
        if Path::new(&path).exists() {
            return Some(PathBuf::from(path));
        }
        None
    } else {
        None
    }
}

fn map_sort_column(field: &str) -> &'static str {
    match field {
        "title" => "title COLLATE NOCASE",
        "artist" => "artist COLLATE NOCASE",
        "albumartist" => "albumartist COLLATE NOCASE",
        "album" => "album COLLATE NOCASE",
        "year" => "year",
        "genre" => "genre COLLATE NOCASE",
        "duration" => "length",
        _ => "artist COLLATE NOCASE", // Default
    }
}

pub fn query_beets_to_songs(
    db_path: &PathBuf,
    search_term: &str,
    sort_by: &str,
    descending: bool,
) -> Result<Vec<Song>> {
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    // Construct the SQL string safely
    let order_dir = if descending { "DESC" } else { "ASC" };
    let order_col = map_sort_column(sort_by);

    // We use format! for the ORDER BY clause because it's not a parameterizable part of SQL
    let query_sql = format!(
        "SELECT 
            id, path, title, artist, album, albumartist, comp, 
            year, genre, composer, track, tracktotal, disc, disctotal, 
            length, bitrate, samplerate, bitdepth, channels, format, added
         FROM items 
         WHERE title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1
         ORDER BY {} {}",
        order_col, order_dir
    );

    let mut stmt = conn.prepare(&query_sql)?;
    let search_pattern = format!("%{}%", search_term);

    let song_iter = stmt.query_map([&search_pattern], |row| {
        // PATH: Beets uses BLOB for paths. String::from_utf8_lossy is essential.
        let path_bytes: Vec<u8> = row.get(1).unwrap_or_default();
        let path_str = String::from_utf8_lossy(&path_bytes).into_owned();

        // DURATION: length is REAL (f64)
        let duration_secs: f64 = row.get::<_, f64>(14).unwrap_or(0.0);

        // BITRATE/SAMPLE: INTEGER (i64)
        let file_info = FileInfo {
            duration: Some(duration_secs),
            duration_display: Some(format_duration(duration_secs)),
            overall_bitrate: Some(row.get::<_, i64>(15).unwrap_or(0) as u32),
            audio_bitrate: None,
            sample_rate: Some(row.get::<_, i64>(16).unwrap_or(0) as u32),
            bit_depth: Some(row.get::<_, i64>(17).unwrap_or(0) as u8),
            channels: Some(row.get::<_, i64>(18).unwrap_or(0) as u8),
            lossless: is_lossless(&row.get::<_, String>(19).unwrap_or_default()),
            tag_type: None,
            codec: row.get::<_, Option<String>>(19).ok().flatten(),
        };

        let title_from_db: String = row
            .get::<_, Option<String>>(2)
            .unwrap_or_default()
            .unwrap_or_default();

        let final_title = if title_from_db.is_empty() {
            // Fallback to filename if title is empty
            PathBuf::from(&path_str)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            title_from_db
        };

        Ok(Song {
            id: format!("beets-{}", row.get::<_, i64>(0)?), // id is INTEGER
            path: path_str.clone(),
            file: PathBuf::from(&path_str)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default(),
            file_info,
            metadata: HashMap::new(),
            title: final_title,
            artist: row
                .get::<_, Option<String>>(3)
                .unwrap_or_default()
                .unwrap_or_else(|| "Unknown".to_string()),
            album: row
                .get::<_, Option<String>>(4)
                .unwrap_or_default()
                .unwrap_or_else(|| "Unknown".to_string()),
            album_artist: row.get::<_, Option<String>>(5).ok().flatten(),
            compilation: row.get::<_, i64>(6).unwrap_or(0) as i32, // comp is INTEGER
            year: row.get::<_, i64>(7).unwrap_or(0) as i32,        // year is INTEGER
            genre: vec![row
                .get::<_, Option<String>>(8)
                .unwrap_or_default()
                .unwrap_or_default()],
            composer: vec![row
                .get::<_, Option<String>>(9)
                .unwrap_or_default()
                .unwrap_or_default()],
            track_number: row.get::<_, i64>(10).unwrap_or(0) as i32,
            track_total: row.get::<_, i64>(11).unwrap_or(0) as i32,
            disc_number: row.get::<_, i64>(12).unwrap_or(0) as i32,
            disc_total: row.get::<_, i64>(13).unwrap_or(0) as i32,
            duration: format_duration(duration_secs),
            artwork: None,
            artwork_origin: None,
            origin_country: None,
            // added is REAL (f64) Unix timestamp in seconds
            date_added: Some((row.get::<_, f64>(20).unwrap_or(0.0) * 1000.0) as u128),
        })
    })?;

    // Use collect to resolve the iterator.
    // If one row fails, it will stop and return Err, which is better for debugging.
    song_iter.collect()
}

fn format_duration(seconds: f64) -> String {
    let mins = (seconds / 60.0).floor() as i32;
    let secs = (seconds % 60.0).floor() as i32;
    format!("{}:{:02}", mins, secs)
}

fn is_lossless(format: &str) -> bool {
    matches!(
        format.to_lowercase().as_str(),
        "flac" | "alac" | "wav" | "aiff"
    )
}
