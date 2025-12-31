use log::info;
use rusqlite::{Connection, OpenFlags, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tauri::{Emitter, Manager};

use crate::{
    metadata::{
        artwork_cacher::get_image_format, convert_file_src, country_name, Album, AlbumArtwork,
        FileInfo, Song,
    },
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
        get_beets_db_path(&app_handle).ok_or_else(|| "Beets database not found".to_string())?;
    let sort = sort_by.unwrap_or_else(|| "artist".to_string());
    let desc = descending.unwrap_or(false);
    let songs = query_beets_to_songs(&db_path, &query, &sort, desc).map_err(|e| {
        app_handle.emit("error", e.to_string());
        e.to_string()
    });
    if songs.is_err() {
        return Err(songs.unwrap_err());
    }
    info!("[Beets] Found {} results", songs.as_ref().unwrap().len());
    songs
}

#[tauri::command]
pub async fn search_beets_albums(
    query: String,
    sort_by: Option<String>,
    descending: Option<bool>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<Album>, String> {
    info!("[Beets] Searching albums for {}", query);
    let sort = sort_by.unwrap_or_else(|| "artist".to_string());

    let db_path =
        get_beets_db_path(&app_handle).ok_or_else(|| "Beets database not found".to_string())?;

    query_beets_albums(&db_path, &query, &sort, descending.unwrap_or(false)).map_err(|e| {
        app_handle.emit("error", e.to_string());
        e.to_string()
    })
}

#[tauri::command]
pub async fn get_albums_by_id(
    album_ids: Vec<String>,
    sort_by: Option<String>,
    descending: Option<bool>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<Album>, String> {
    info!("[Beets] Searching albums by ids {:?}", album_ids);
    let sort = sort_by.unwrap_or_else(|| "album".to_string());

    let db_path =
        get_beets_db_path(&app_handle).ok_or_else(|| "Beets database not found".to_string())?;

    query_beets_albums_by_ids(&db_path, album_ids, &sort, descending.unwrap_or(false)).map_err(
        |e| {
            app_handle.emit("error", e.to_string());
            e.to_string()
        },
    )
}

#[tauri::command]
pub async fn get_beets_album_tracks(
    album_id: String,
    app_handle: tauri::AppHandle,
) -> Result<Vec<Song>, String> {
    info!("[Beets] Fetching tracks for album {}", album_id);

    let db_path =
        get_beets_db_path(&app_handle).ok_or_else(|| "Beets database not found".to_string())?;

    query_beets_album_tracks(&db_path, album_id).map_err(|e| {
        app_handle.emit("error", e.to_string());
        e.to_string()
    })
}

pub fn get_beets_db_path(app_handle: &tauri::AppHandle) -> Option<PathBuf> {
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

fn map_sort_column_album(field: &str) -> &'static str {
    match field {
        "title" => "album COLLATE NOCASE",
        "artist" => "albumartist COLLATE NOCASE",
        "year" => "year",
        _ => "album COLLATE NOCASE", // Default
    }
}

const SONG_FIELDS: &str = "id, path, title, artist, album, albumartist, comp, 
            year, genre, composer, track, tracktotal, disc, disctotal, 
            length, bitrate, samplerate, bitdepth, channels, format, added, country, album_id";

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
        "SELECT {}
         FROM items 
         WHERE title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1
         ORDER BY {} {}",
        SONG_FIELDS, order_col, order_dir
    );

    let mut stmt = conn.prepare(&query_sql)?;
    let search_pattern = format!("%{}%", search_term);

    let song_iter = stmt.query_map([&search_pattern], row_to_song)?;

    // Use collect to resolve the iterator.
    // If one row fails, it will stop and return Err, which is better for debugging.
    song_iter.collect()
}

pub fn query_beets_albums(
    db_path: &PathBuf,
    search_term: &str,
    sort_by: &str,
    descending: bool,
) -> Result<Vec<Album>> {
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    let order_dir = if descending { "DESC" } else { "ASC" };
    let order_col = map_sort_column_album(sort_by);

    let search_pattern = format!("%{}%", search_term);

    let album_sql = format!(
        "
        SELECT
            a.id,
            a.album,
            a.albumartist,
            a.comp,
            a.year,
            a.genre,
            a.artpath
        FROM albums a
        WHERE a.album LIKE ?1 OR a.albumartist LIKE ?1
        ORDER BY {} {}",
        order_col, order_dir
    );

    let mut stmt = conn.prepare(&album_sql)?;

    let albums_iter = stmt.query_map([&search_pattern], |row| {
        let album_db_id: i64 = row.get(0)?;
        let title: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
        let artist: String = row
            .get::<_, Option<String>>(2)?
            .unwrap_or_else(|| "Unknown".into());
        let compilation: i32 = row.get::<_, i64>(3).unwrap_or(0) as i32;
        let year: i32 = row.get::<_, i64>(4).unwrap_or(0) as i32;
        let genre_str: String = row.get::<_, Option<String>>(5)?.unwrap_or_default();
        let mut art_path: String = row
            .get::<_, Option<Vec<u8>>>(6)?
            .and_then(|bytes| {
                let s = String::from_utf8_lossy(&bytes).into_owned();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            })
            .unwrap_or_default();

        // Fetch tracks for this album
        let mut track_stmt = conn.prepare(
            "
            SELECT id, format
            FROM items
            WHERE album_id = ?1
            ORDER BY disc, track
            ",
        )?;

        let mut tracks_ids = Vec::new();
        let mut lossless = true;

        let track_iter = track_stmt.query_map([album_db_id], |r| {
            let id: i64 = r.get(0)?;
            let format: String = r.get::<_, Option<String>>(1)?.unwrap_or_default();
            Ok((id, format))
        })?;

        for track in track_iter {
            let (id, format) = track?;
            tracks_ids.push(format!("{}", id));
            if !is_lossless(&format) {
                lossless = false;
            }
        }

        let album_id = album_db_id.to_string();

        // Artwork
        let artwork = Path::new(&art_path)
            .exists()
            .then(|| Path::new(&art_path))
            .and_then(|p| p.extension())
            .and_then(|ext| ext.to_str())
            .and_then(get_image_format)
            .map(|format| AlbumArtwork {
                src: convert_file_src(art_path),
                format: format.to_string(),
            });

        Ok(Album {
            id: album_id,
            title: title.to_lowercase(),
            display_title: title,
            artist,
            compilation,
            year,
            genre: if genre_str.is_empty() {
                vec![]
            } else {
                vec![genre_str]
            },
            tracks_ids,
            path: "".to_string(),
            artwork,
            lossless,
        })
    })?;

    albums_iter.collect()
}

pub fn query_beets_albums_by_ids(
    db_path: &PathBuf,
    album_ids: Vec<String>,
    sort_by: &str,
    descending: bool,
) -> Result<Vec<Album>> {
    if album_ids.is_empty() {
        return Ok(vec![]);
    }

    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    let order_dir = if descending { "DESC" } else { "ASC" };
    let order_col = map_sort_column_album(sort_by);

    // Build placeholders: (?, ?, ?, ...)
    let placeholders = std::iter::repeat("?")
        .take(album_ids.len())
        .collect::<Vec<_>>()
        .join(",");

    let album_sql = format!(
        "
        SELECT
            a.id,
            a.album,
            a.albumartist,
            a.comp,
            a.year,
            a.genre,
            a.artpath
        FROM albums a
        WHERE a.id IN ({})
        ORDER BY {} {}
        ",
        placeholders, order_col, order_dir
    );

    let mut stmt = conn.prepare(&album_sql)?;

    let albums_iter = stmt.query_map(rusqlite::params_from_iter(album_ids.iter()), |row| {
        let album_db_id: i64 = row.get(0)?;
        let title: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
        let artist: String = row
            .get::<_, Option<String>>(2)?
            .unwrap_or_else(|| "Unknown".into());
        let compilation: i32 = row.get::<_, i64>(3).unwrap_or(0) as i32;
        let year: i32 = row.get::<_, i64>(4).unwrap_or(0) as i32;
        let genre_str: String = row.get::<_, Option<String>>(5)?.unwrap_or_default();
        let art_path: String = row
            .get::<_, Option<Vec<u8>>>(6)?
            .and_then(|bytes| {
                let s = String::from_utf8_lossy(&bytes).into_owned();
                (!s.is_empty()).then_some(s)
            })
            .unwrap_or_default();

        // Tracks
        let mut track_stmt = conn.prepare(
            "
                SELECT id, format
                FROM items
                WHERE album_id = ?1
                ORDER BY disc, track
                ",
        )?;

        let mut tracks_ids = Vec::new();
        let mut lossless = true;

        let track_iter = track_stmt.query_map([album_db_id], |r| {
            let id: i64 = r.get(0)?;
            let format: String = r.get::<_, Option<String>>(1)?.unwrap_or_default();
            Ok((id, format))
        })?;

        for track in track_iter {
            let (id, format) = track?;
            tracks_ids.push(id.to_string());
            if !is_lossless(&format) {
                lossless = false;
            }
        }

        // Artwork
        let artwork = Path::new(&art_path)
            .exists()
            .then(|| Path::new(&art_path))
            .and_then(|p| p.extension())
            .and_then(|ext| ext.to_str())
            .and_then(get_image_format)
            .map(|format| AlbumArtwork {
                src: convert_file_src(art_path),
                format: format.to_string(),
            });

        Ok(Album {
            id: album_db_id.to_string(),
            title: title.to_lowercase(),
            display_title: title,
            artist,
            compilation,
            year,
            genre: if genre_str.is_empty() {
                vec![]
            } else {
                vec![genre_str]
            },
            tracks_ids,
            path: "".to_string(),
            artwork,
            lossless,
        })
    })?;

    albums_iter.collect()
}

pub fn query_beets_album_tracks(db_path: &PathBuf, album_id: String) -> Result<Vec<Song>> {
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    let query_sql = format!(
        "
        SELECT {}
        FROM items
        WHERE album_id = ?1
        ORDER BY disc, track
    ",
        SONG_FIELDS
    );

    let mut stmt = conn.prepare(query_sql.as_str())?;
    let song_iter = stmt.query_map([album_id], row_to_song)?;

    song_iter.collect()
}

fn row_to_song(row: &rusqlite::Row) -> rusqlite::Result<Song> {
    // --- PATH (BLOB → String) ---
    let path_bytes: Vec<u8> = row.get(1).unwrap_or_default();
    let path_str = String::from_utf8_lossy(&path_bytes).into_owned();

    // --- DURATION ---
    let duration_secs: f64 = row.get::<_, f64>(14).unwrap_or(0.0);

    // --- FILE INFO ---
    let format = row
        .get::<_, Option<String>>(19)
        .ok()
        .flatten()
        .unwrap_or_default();

    let file_info = FileInfo {
        duration: Some(duration_secs),
        duration_display: Some(format_duration(duration_secs)),
        overall_bitrate: Some(row.get::<_, i64>(15).unwrap_or(0) as u32),
        audio_bitrate: None,
        sample_rate: Some(row.get::<_, i64>(16).unwrap_or(0) as u32),
        bit_depth: Some(row.get::<_, i64>(17).unwrap_or(0) as u8),
        channels: Some(row.get::<_, i64>(18).unwrap_or(0) as u8),
        lossless: is_lossless(&format),
        tag_type: None,
        codec: if format.is_empty() {
            None
        } else {
            Some(format)
        },
    };

    // --- TITLE FALLBACK ---
    let title_from_db: String = row.get::<_, Option<String>>(2)?.unwrap_or_default();

    let final_title = if title_from_db.is_empty() {
        PathBuf::from(&path_str)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Unknown".to_string())
    } else {
        title_from_db
    };

    Ok(Song {
        id: format!("beets-{}", row.get::<_, i64>(0)?),
        path: path_str.clone(),
        file: PathBuf::from(&path_str)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        file_info,
        metadata: HashMap::new(),
        title: final_title,
        artist: row
            .get::<_, Option<String>>(3)?
            .unwrap_or_else(|| "Unknown".to_string()),
        album: row
            .get::<_, Option<String>>(4)?
            .unwrap_or_else(|| "Unknown".to_string()),
        album_id: row
            .get::<_, Option<i64>>(22)
            .ok()
            .flatten()
            .map(|f| f.to_string()),
        album_artist: row.get::<_, Option<String>>(5).ok().flatten(),
        compilation: row.get::<_, i64>(6).unwrap_or(0) as i32,
        year: row.get::<_, i64>(7).unwrap_or(0) as i32,
        genre: vec![row.get::<_, Option<String>>(8)?.unwrap_or_default()],
        composer: vec![row.get::<_, Option<String>>(9)?.unwrap_or_default()],
        track_number: row.get::<_, i64>(10).unwrap_or(0) as i32,
        track_total: row.get::<_, i64>(11).unwrap_or(0) as i32,
        disc_number: row.get::<_, i64>(12).unwrap_or(0) as i32,
        disc_total: row.get::<_, i64>(13).unwrap_or(0) as i32,
        duration: format_duration(duration_secs),
        artwork: None,
        artwork_origin: None,
        origin_country: Some(row.get::<_, Option<String>>(21)?.unwrap_or_default()),
        origin_country_name: row
            .get::<_, Option<String>>(21)?
            .and_then(|c| country_name(c.as_str()))
            .map(|c| c.to_string()),
        date_added: Some((row.get::<_, f64>(20).unwrap_or(0.0) * 1000.0) as u128),
    })
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
