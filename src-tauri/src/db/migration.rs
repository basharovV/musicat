//! One-time Dexie → SQLite migration (tasks.md group 5): parse a
//! `dexie-export-import` export, write it into a *shadow* SQLite file (not
//! the live one), verify it against the source, and only then fold it into
//! the live database — see "Migration tool: shadow-migrate, diff, then cut
//! over" in `design.md`.
//!
//! Cutover doesn't swap files on disk: by the time this can run, the app's
//! writer actor already holds the live file open (it's created at Tauri
//! startup — see `db::Database::init`), and renaming over open handles is
//! fragile, especially on Windows. Instead `cutover` `ATTACH`es the shadow
//! file onto the writer's existing connection and copies verified rows
//! across in one transaction, which sidesteps the problem entirely.

use std::collections::HashMap;

use rusqlite::Connection;
use serde::Deserialize;

use crate::metadata::Marker;
use crate::stem_separator::Stem;

// ---------------------------------------------------------------------------
// dexie-export-import envelope
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct DexieExport {
    #[serde(rename = "formatName")]
    pub format_name: String,
    pub data: DexieExportData,
}

#[derive(Debug, Deserialize)]
pub struct DexieExportData {
    pub data: Vec<DexieTableRows>,
}

#[derive(Debug, Deserialize)]
pub struct DexieTableRows {
    #[serde(rename = "tableName")]
    pub table_name: String,
    pub rows: Vec<serde_json::Value>,
}

impl DexieExportData {
    fn table(&self, name: &str) -> Vec<&serde_json::Value> {
        self.data
            .iter()
            .filter(|t| t.table_name == name)
            .flat_map(|t| t.rows.iter())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Per-table row shapes. `#[serde(default)]` throughout: exports can come from
// any Dexie schema version back to v14 (see `src/data/db.ts`), so a field
// added in a later version must not fail parsing of an older export.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct DexieSong {
    id: String,
    path: String,
    file: String,
    title: String,
    artist: String,
    album: String,
    album_id: Option<String>,
    album_artist: Option<String>,
    year: i64,
    compilation: i64,
    genre: Vec<String>,
    composer: Vec<String>,
    track_number: i64,
    track_total: i64,
    disc_number: i64,
    disc_total: i64,
    duration: String,
    origin_country: Option<String>,
    song_project_id: Option<i64>,
    play_count: i64,
    is_favourite: bool,
    tags: Vec<String>,
    markers: Vec<Marker>,
    stems: Vec<Stem>,
    date_added: Option<i64>,
    file_info: DexieFileInfo,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct DexieFileInfo {
    duration: Option<f64>,
    duration_display: Option<String>,
    overall_bitrate: Option<i64>,
    audio_bitrate: Option<i64>,
    sample_rate: Option<i64>,
    bit_depth: Option<i64>,
    channels: Option<i64>,
    lossless: bool,
    tag_type: Option<String>,
    codec: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct DexieAlbumArtwork {
    // Matches `ArtworkSrc` (App.d.ts) / `AlbumArtwork` (metadata.rs): artwork
    // bytes live on disk (see artwork::cache_artwork), this is just the
    // pointer to that cache. There is no raw byte array on this record.
    format: String,
    src: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct DexieAlbum {
    id: String,
    title: String,
    display_title: Option<String>,
    artist: String,
    year: i64,
    compilation: i64,
    genre: Vec<String>,
    path: Option<String>,
    lossless: bool,
    artwork: Option<DexieAlbumArtwork>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct DexiePlaylist {
    id: serde_json::Value, // Dexie auto-increments `playlists`, gives explicit string ids to `internalPlaylists`
    title: String,
    path: Option<String>,
    tracks: Vec<String>,
}

impl DexiePlaylist {
    fn id_string(&self) -> String {
        match &self.id {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct DexieSongProject {
    id: serde_json::Value,
    song_filepath: Option<String>,
    title: String,
    artist: String,
    album: Option<String>,
    key: Option<String>,
    bpm: Option<f64>,
    music_composed_by: Vec<String>,
    lyrics_written_by: Vec<String>,
    artwork_file_path: Option<String>,
    lyrics: Option<String>,
    recordings: serde_json::Value,
    other_content_items: serde_json::Value,
    song_id: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct DexieArtistProject {
    id: serde_json::Value,
    name: String,
    members: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct DexieScrapbookItem {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    name: String,
    tags: Vec<String>,
    song_id: Option<String>,
    url: Option<String>,
    image_url: Option<String>,
    file_type: Option<DexieFileType>,
    path: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct DexieFileType {
    #[serde(rename = "type")]
    kind: String,
    extension: String,
}

// ---------------------------------------------------------------------------
// Migration
// ---------------------------------------------------------------------------

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationReport {
    pub tracks: usize,
    pub albums: usize,
    pub playlists: usize,
    pub smart_queries: usize,
    pub song_projects: usize,
    pub artist_projects: usize,
    pub scrapbook_items: usize,
    pub warnings: Vec<String>,
}

/// Writes `export` into `conn` (expected to be a freshly-migrated, empty
/// shadow database) in one transaction — either the whole export lands, or
/// none of it does.
pub fn migrate(conn: &mut Connection, export: &DexieExport) -> Result<MigrationReport, String> {
    if export.format_name != "dexie" {
        return Err(format!("unrecognized export format '{}'", export.format_name));
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut report = MigrationReport::default();

    for row in export.data.table("albums") {
        let album: DexieAlbum = serde_json::from_value(row.clone()).map_err(|e| e.to_string())?;
        let (artwork_format, artwork_src) = match album.artwork {
            Some(a) => (Some(a.format), a.src),
            None => (None, None),
        };
        tx.execute(
            "INSERT INTO albums (id, title, display_title, artist, year, compilation, genre, path,
                lossless, artwork_format, artwork_src)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                album.id,
                album.title,
                album.display_title.unwrap_or_else(|| album.title.clone()),
                album.artist,
                album.year,
                album.compilation,
                serde_json::to_string(&album.genre).unwrap(),
                album.path,
                album.lossless as i64,
                artwork_format,
                artwork_src,
            ],
        )
        .map_err(|e| e.to_string())?;
        report.albums += 1;
    }

    // Cache tag name -> id so repeated tags across songs share one row.
    let mut tag_ids: HashMap<String, i64> = HashMap::new();
    let mut tag_id_for = |tx: &rusqlite::Transaction, name: &str| -> rusqlite::Result<i64> {
        if let Some(id) = tag_ids.get(name) {
            return Ok(*id);
        }
        tx.execute("INSERT OR IGNORE INTO tags (name) VALUES (?1)", [name])?;
        let id: i64 = tx.query_row("SELECT id FROM tags WHERE name = ?1", [name], |r| r.get(0))?;
        tag_ids.insert(name.to_string(), id);
        Ok(id)
    };

    for row in export.data.table("songs") {
        let song: DexieSong = match serde_json::from_value(row.clone()) {
            Ok(s) => s,
            Err(e) => {
                report.warnings.push(format!("skipped a song row: {e}"));
                continue;
            }
        };
        tx.execute(
            "INSERT INTO tracks (id, path, file, title, artist, album, album_id, album_artist,
                year, compilation, genre, composer, track_number, track_total, disc_number,
                disc_total, duration_secs, duration_display, overall_bitrate, audio_bitrate,
                sample_rate, bit_depth, channels, lossless, tag_type, codec, play_count,
                is_favourite, origin_country, song_project_id, markers, stems, date_added)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,
                ?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32,?33)",
            rusqlite::params![
                song.id,
                song.path,
                song.file,
                song.title,
                song.artist,
                song.album,
                song.album_id,
                song.album_artist,
                song.year,
                song.compilation,
                serde_json::to_string(&song.genre).unwrap(),
                serde_json::to_string(&song.composer).unwrap(),
                song.track_number,
                song.track_total,
                song.disc_number,
                song.disc_total,
                song.file_info.duration,
                song.file_info.duration_display.or(Some(song.duration.clone())),
                song.file_info.overall_bitrate,
                song.file_info.audio_bitrate,
                song.file_info.sample_rate,
                song.file_info.bit_depth,
                song.file_info.channels,
                song.file_info.lossless as i64,
                song.file_info.tag_type,
                song.file_info.codec,
                song.play_count,
                song.is_favourite as i64,
                song.origin_country,
                song.song_project_id,
                serde_json::to_string(&song.markers).unwrap(),
                serde_json::to_string(&song.stems).unwrap(),
                song.date_added.unwrap_or(0),
            ],
        )
        .map_err(|e| e.to_string())?;

        for tag in &song.tags {
            let tag_id = tag_id_for(&tx, tag).map_err(|e| e.to_string())?;
            tx.execute(
                "INSERT OR IGNORE INTO track_tags (track_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![song.id, tag_id],
            )
            .map_err(|e| e.to_string())?;
        }

        report.tracks += 1;
    }

    for table_name in ["playlists", "internalPlaylists"] {
        for row in export.data.table(table_name) {
            let playlist: DexiePlaylist = serde_json::from_value(row.clone()).map_err(|e| e.to_string())?;
            let id = playlist.id_string();
            tx.execute(
                "INSERT OR IGNORE INTO playlists (id, title, file_path) VALUES (?1, ?2, ?3)",
                rusqlite::params![id, playlist.title, playlist.path],
            )
            .map_err(|e| e.to_string())?;
            for (position, track_id) in playlist.tracks.iter().enumerate() {
                tx.execute(
                    "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
                    rusqlite::params![id, track_id, position as i64],
                )
                .map_err(|e| e.to_string())?;
            }
            report.playlists += 1;
        }
    }

    for row in export.data.table("smartQueries") {
        // The predicate shape is still owned by the frontend (see tasks.md
        // 3.6 — porting it into `QuerySpec` is separate work), so the whole
        // row is kept as-is; only `name` is pulled out for the column Rust
        // itself needs to sort/list by.
        let name = row.get("name").and_then(|v| v.as_str()).unwrap_or("Untitled query").to_string();
        tx.execute(
            "INSERT INTO smart_queries (name, query_json) VALUES (?1, ?2)",
            rusqlite::params![name, row.to_string()],
        )
        .map_err(|e| e.to_string())?;
        report.smart_queries += 1;
    }

    // `artist_projects.name` is UNIQUE and `song_projects` links to it via `artist_id` (see
    // `openspec/changes/artists-toolkit-storage/design.md`) — the shadow db already has this shape
    // by the time this runs (migration 0002 applies before any row is written). A Dexie export can
    // still contain duplicate rows for the same artist/song (the exact dead-write bug this change
    // fixes produced them), so both loops upsert by natural key rather than plain INSERT, same
    // "keep the latest" resolution the schema migration itself uses for pre-existing installs.
    let artist_id_for = |tx: &rusqlite::Transaction, name: &str| -> rusqlite::Result<i64> {
        tx.execute(
            "INSERT INTO artist_projects (name, members) VALUES (?1, '[]')
             ON CONFLICT(name) DO NOTHING",
            [name],
        )?;
        tx.query_row("SELECT id FROM artist_projects WHERE name = ?1", [name], |r| r.get(0))
    };

    for row in export.data.table("artistProjects") {
        let ap: DexieArtistProject = serde_json::from_value(row.clone()).map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO artist_projects (name, members) VALUES (?1, ?2)
             ON CONFLICT(name) DO UPDATE SET members = excluded.members",
            rusqlite::params![ap.name, serde_json::to_string(&ap.members).unwrap()],
        )
        .map_err(|e| e.to_string())?;
        report.artist_projects += 1;
    }

    for row in export.data.table("songProjects") {
        let sp: DexieSongProject = serde_json::from_value(row.clone()).map_err(|e| e.to_string())?;
        let artist_id = artist_id_for(&tx, &sp.artist).map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO song_projects (artist_id, title, song_filepath, album, key, bpm,
                music_composed_by, lyrics_written_by, artwork_file_path, lyrics, recordings,
                other_content_items, song_id)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
             ON CONFLICT(artist_id, title) DO UPDATE SET
                song_filepath = excluded.song_filepath, album = excluded.album, key = excluded.key,
                bpm = excluded.bpm, music_composed_by = excluded.music_composed_by,
                lyrics_written_by = excluded.lyrics_written_by,
                artwork_file_path = excluded.artwork_file_path, lyrics = excluded.lyrics,
                recordings = excluded.recordings, other_content_items = excluded.other_content_items,
                song_id = excluded.song_id",
            rusqlite::params![
                artist_id,
                sp.title,
                sp.song_filepath,
                sp.album,
                sp.key,
                sp.bpm,
                serde_json::to_string(&sp.music_composed_by).unwrap(),
                serde_json::to_string(&sp.lyrics_written_by).unwrap(),
                sp.artwork_file_path,
                sp.lyrics,
                sp.recordings.to_string(),
                sp.other_content_items.to_string(),
                sp.song_id,
            ],
        )
        .map_err(|e| e.to_string())?;
        report.song_projects += 1;
    }

    for row in export.data.table("scrapbook") {
        let item: DexieScrapbookItem = serde_json::from_value(row.clone()).map_err(|e| e.to_string())?;
        let (file_type_type, file_type_extension) = match item.file_type {
            Some(ft) => (Some(ft.kind), Some(ft.extension)),
            None => (None, None),
        };
        tx.execute(
            "INSERT INTO scrapbook (id, type, name, tags, song_id, url, image_url,
                file_type_type, file_type_extension, path)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            rusqlite::params![
                item.id,
                item.kind,
                item.name,
                serde_json::to_string(&item.tags).unwrap(),
                item.song_id,
                item.url,
                item.image_url,
                file_type_type,
                file_type_extension,
                item.path,
            ],
        )
        .map_err(|e| e.to_string())?;
        report.scrapbook_items += 1;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(report)
}

// ---------------------------------------------------------------------------
// Verification — record counts plus a checksum of the user-editable fields
// that must survive migration byte-for-byte (design.md 5.3).
// ---------------------------------------------------------------------------

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationReport {
    pub ok: bool,
    pub mismatches: Vec<String>,
}

pub fn verify(export: &DexieExport, conn: &Connection) -> Result<VerificationReport, String> {
    let mut mismatches = Vec::new();

    let source_songs: Vec<DexieSong> = export
        .data
        .table("songs")
        .into_iter()
        .filter_map(|r| serde_json::from_value(r.clone()).ok())
        .collect();

    let live_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if live_count as usize != source_songs.len() {
        mismatches.push(format!(
            "track count mismatch: source had {}, shadow db has {live_count}",
            source_songs.len()
        ));
    }

    for song in &source_songs {
        let row: Option<(i64, i64, Option<String>)> = conn
            .query_row(
                "SELECT play_count, is_favourite,
                    (SELECT group_concat(t.name, char(1)) FROM track_tags tt
                       JOIN tags t ON t.id = tt.tag_id WHERE tt.track_id = tracks.id)
                 FROM tracks WHERE id = ?1",
                [&song.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .ok();

        match row {
            None => mismatches.push(format!("track '{}' missing from shadow db", song.id)),
            Some((play_count, is_favourite, tags_concat)) => {
                if play_count != song.play_count {
                    mismatches.push(format!(
                        "track '{}' playCount mismatch: source {}, shadow {play_count}",
                        song.id, song.play_count
                    ));
                }
                if (is_favourite != 0) != song.is_favourite {
                    mismatches.push(format!("track '{}' isFavourite mismatch", song.id));
                }
                let mut shadow_tags: Vec<String> =
                    tags_concat.map(|s| s.split('\u{1}').map(str::to_string).collect()).unwrap_or_default();
                let mut source_tags = song.tags.clone();
                shadow_tags.sort();
                source_tags.sort();
                if shadow_tags != source_tags {
                    mismatches.push(format!("track '{}' tags mismatch", song.id));
                }
            }
        }
    }

    Ok(VerificationReport { ok: mismatches.is_empty(), mismatches })
}

/// Copies every migrated table from the shadow database (already migrated
/// and verified) into the live one and marks the live settings row as
/// migrated. Attaches by path rather than swapping files — see the module
/// doc comment for why.
pub fn cutover(conn: &mut Connection, shadow_db_path: &std::path::Path) -> Result<(), String> {
    // ATTACH/DETACH are not allowed inside an explicit transaction (SQLite
    // rejects it), so they bracket the transaction instead of living inside
    // it. The copy itself is still one atomic transaction.
    conn.execute("ATTACH DATABASE ?1 AS shadow", [shadow_db_path.to_string_lossy().to_string()])
        .map_err(|e| e.to_string())?;

    let copy_result = (|| -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let copy_order = [
            "albums",
            "tracks",
            "tags",
            "track_tags",
            "playlists",
            "playlist_tracks",
            "smart_queries",
            // artist_projects before song_projects: song_projects.artist_id now references it
            // (see openspec/changes/artists-toolkit-storage/design.md) and foreign_keys is ON.
            "artist_projects",
            "song_projects",
            "scrapbook",
        ];
        for table in copy_order {
            tx.execute(&format!("INSERT INTO {table} SELECT * FROM shadow.{table}"), [])
                .map_err(|e| format!("copying table '{table}': {e}"))?;
        }
        tx.commit().map_err(|e| e.to_string())
    })();

    conn.execute("DETACH DATABASE shadow", []).map_err(|e| e.to_string())?;
    copy_result
}
