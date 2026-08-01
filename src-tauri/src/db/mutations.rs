//! Track/album write commands — the canonical merge logic that tasks.md 6.4
//! calls out to collapse: the old frontend had two independently-maintained
//! "preserve user data across a metadata refresh" implementations
//! (`LibraryUtils.reImportTracks`, which restored 5 fields, and
//! `ImportWorker.handleImport`, which restored a *different* 7 fields —
//! losing `songProjectId`/`dateAdded` on the `reImportTracks` path was a
//! real bug). Here it's structural instead of remembered: an existing
//! track's UPDATE only ever touches the columns a fresh file scan produces
//! (title, artist, tags-from-file, duration, ...); every user-owned column
//! (`play_count`, `is_favourite`, `tags`, `markers`, `stems`,
//! `song_project_id`, `date_added`, `origin_country`) is simply never in the
//! SET list, so there's no field list to keep in sync by hand.

use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension};

use crate::metadata::{Album, Song};

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

/// Ensures a row exists in `albums` for `track.album_id`, deriving a minimal
/// one from the track itself if it's missing. Needed because Rust's scan
/// pipeline streams songs and albums as separate, independently-timed event
/// chunks (`import_chunk` / `import_albums`) — a track's album can arrive
/// before the album record does. `upsert_albums` supersedes this placeholder
/// with the authoritative data (artwork, cross-track lossless flag, ...) once
/// it lands; until then this keeps the `album_id` foreign key satisfiable.
fn ensure_album_placeholder(tx: &rusqlite::Transaction, track: &Song) -> rusqlite::Result<()> {
    let Some(album_id) = &track.album_id else { return Ok(()) };
    let exists: bool =
        tx.query_row("SELECT 1 FROM albums WHERE id = ?1", [album_id], |_| Ok(())).optional()?.is_some();
    if exists {
        return Ok(());
    }
    let dir = track.path.rsplit_once('/').map(|(dir, _)| dir.to_string());
    tx.execute(
        "INSERT INTO albums (id, title, display_title, artist, year, compilation, genre, path, lossless)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            album_id,
            track.album.to_lowercase(),
            track.album,
            track.album_artist.clone().unwrap_or_else(|| track.artist.clone()),
            track.year,
            track.compilation,
            serde_json::to_string(&track.genre).unwrap(),
            dir,
            track.file_info.lossless as i64,
        ],
    )?;
    Ok(())
}

/// Insert-or-refresh a batch of freshly-scanned tracks. New tracks get
/// inserted with default user-owned fields; existing tracks get only their
/// extracted-metadata columns refreshed (see module docs for why that alone
/// is what "preserves user data" means here). Returns the resulting rows —
/// same shape the frontend used to get back from Dexie's `bulkPut`.
///
/// If a track's album changed and its old album is left with no tracks, that
/// stale album is deleted (this subsumes what `reImportAlbum`'s
/// `songIdToOldAlbumId` bookkeeping did in the old frontend code — it's
/// computed here from the database's own prior state instead of the caller
/// having to track it).
pub fn upsert_tracks(conn: &mut Connection, tracks: &[Song]) -> Result<Vec<Song>, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut albums_to_check: HashSet<String> = HashSet::new();

    for track in tracks {
        ensure_album_placeholder(&tx, track).map_err(|e| e.to_string())?;

        let existing: Option<Option<String>> = tx
            .query_row("SELECT album_id FROM tracks WHERE id = ?1", [&track.id], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;

        let duration_display = track.file_info.duration_display.clone().or_else(|| Some(track.duration.clone()));

        match existing {
            Some(old_album_id) => {
                tx.execute(
                    "UPDATE tracks SET path=?1, file=?2, title=?3, artist=?4, album=?5, album_id=?6,
                        album_artist=?7, year=?8, compilation=?9, genre=?10, composer=?11,
                        track_number=?12, track_total=?13, disc_number=?14, disc_total=?15,
                        duration_secs=?16, duration_display=?17, overall_bitrate=?18, audio_bitrate=?19,
                        sample_rate=?20, bit_depth=?21, channels=?22, lossless=?23, tag_type=?24, codec=?25
                     WHERE id = ?26",
                    rusqlite::params![
                        track.path,
                        track.file,
                        track.title,
                        track.artist,
                        track.album,
                        track.album_id,
                        track.album_artist,
                        track.year,
                        track.compilation,
                        serde_json::to_string(&track.genre).unwrap(),
                        serde_json::to_string(&track.composer).unwrap(),
                        track.track_number,
                        track.track_total,
                        track.disc_number,
                        track.disc_total,
                        track.file_info.duration,
                        duration_display,
                        track.file_info.overall_bitrate,
                        track.file_info.audio_bitrate,
                        track.file_info.sample_rate,
                        track.file_info.bit_depth,
                        track.file_info.channels,
                        track.file_info.lossless as i64,
                        track.file_info.tag_type,
                        track.file_info.codec,
                        track.id,
                    ],
                )
                .map_err(|e| e.to_string())?;

                if let Some(old_album_id) = old_album_id {
                    if Some(&old_album_id) != track.album_id.as_ref() {
                        albums_to_check.insert(old_album_id);
                    }
                }
            }
            None => {
                tx.execute(
                    "INSERT INTO tracks (id, path, file, title, artist, album, album_id, album_artist,
                        year, compilation, genre, composer, track_number, track_total, disc_number,
                        disc_total, duration_secs, duration_display, overall_bitrate, audio_bitrate,
                        sample_rate, bit_depth, channels, lossless, tag_type, codec, play_count,
                        is_favourite, origin_country, song_project_id, markers, stems, date_added)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,
                        ?21,?22,?23,?24,?25,?26,0,0,'',NULL,'[]','[]',?27)",
                    rusqlite::params![
                        track.id,
                        track.path,
                        track.file,
                        track.title,
                        track.artist,
                        track.album,
                        track.album_id,
                        track.album_artist,
                        track.year,
                        track.compilation,
                        serde_json::to_string(&track.genre).unwrap(),
                        serde_json::to_string(&track.composer).unwrap(),
                        track.track_number,
                        track.track_total,
                        track.disc_number,
                        track.disc_total,
                        track.file_info.duration,
                        duration_display,
                        track.file_info.overall_bitrate,
                        track.file_info.audio_bitrate,
                        track.file_info.sample_rate,
                        track.file_info.bit_depth,
                        track.file_info.channels,
                        track.file_info.lossless as i64,
                        track.file_info.tag_type,
                        track.file_info.codec,
                        track.date_added.map(|d| d as i64).unwrap_or_else(now_ms),
                    ],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    let mut deleted_albums = Vec::new();
    for album_id in albums_to_check {
        let remaining: i64 = tx
            .query_row("SELECT COUNT(*) FROM tracks WHERE album_id = ?1", [&album_id], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if remaining == 0 {
            tx.execute("DELETE FROM albums WHERE id = ?1", [&album_id]).map_err(|e| e.to_string())?;
            deleted_albums.push(album_id);
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    let ids: Vec<String> = tracks.iter().map(|t| t.id.clone()).collect();
    let rows = super::query::get_tracks_by_ids(conn, &ids)?;
    Ok(rows.into_iter().flatten().collect())
}

/// Insert-or-replace album metadata (artwork, genre, lossless-across-tracks,
/// ...). Unlike tracks, albums carry no user-owned fields, so this is a
/// plain overwrite — matches the old `reImportAlbum`'s behavior for every
/// field except `tracksIds`, which no longer exists (see
/// `models::row_to_album`).
pub fn upsert_albums(conn: &mut Connection, albums: &[Album]) -> Result<Vec<Album>, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    for album in albums {
        let (artwork_format, artwork_src) = match &album.artwork {
            Some(a) => (Some(a.format.clone()), Some(a.src.clone())),
            None => (None, None),
        };
        tx.execute(
            "INSERT INTO albums (id, title, display_title, artist, year, compilation, genre, path,
                lossless, artwork_format, artwork_src)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title, display_title = excluded.display_title,
                artist = excluded.artist, year = excluded.year, compilation = excluded.compilation,
                genre = excluded.genre, path = excluded.path, lossless = excluded.lossless,
                artwork_format = excluded.artwork_format, artwork_src = excluded.artwork_src",
            rusqlite::params![
                album.id,
                album.title,
                album.display_title,
                album.artist,
                album.year,
                album.compilation,
                serde_json::to_string(&album.genre).unwrap(),
                if album.path.is_empty() { None } else { Some(album.path.clone()) },
                album.lossless as i64,
                artwork_format,
                artwork_src,
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;

    let ids: Vec<String> = albums.iter().map(|a| a.id.clone()).collect();
    let rows = super::query::get_albums_by_ids(conn, &ids)?;
    Ok(rows.into_iter().flatten().collect())
}

/// Deletes tracks and any album that's left with none of its tracks
/// remaining as a result. Returns the ids of albums that were deleted, so
/// the frontend knows to drop them from any album view it's showing.
pub fn delete_tracks(conn: &mut Connection, track_ids: &[String]) -> Result<Vec<String>, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut albums_to_check: HashSet<String> = HashSet::new();
    {
        let placeholders = std::iter::repeat("?").take(track_ids.len()).collect::<Vec<_>>().join(",");
        let sql = format!("SELECT DISTINCT album_id FROM tracks WHERE album_id IS NOT NULL AND id IN ({placeholders})");
        let mut stmt = tx.prepare(&sql).map_err(|e| e.to_string())?;
        let params: Vec<&dyn rusqlite::ToSql> = track_ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let rows = stmt
            .query_map(params.as_slice(), |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        albums_to_check.extend(rows);
    }

    {
        let placeholders = std::iter::repeat("?").take(track_ids.len()).collect::<Vec<_>>().join(",");
        let sql = format!("DELETE FROM tracks WHERE id IN ({placeholders})");
        let params: Vec<&dyn rusqlite::ToSql> = track_ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        tx.execute(&sql, params.as_slice()).map_err(|e| e.to_string())?;
    }

    let mut deleted_albums = Vec::new();
    for album_id in albums_to_check {
        let remaining: i64 = tx
            .query_row("SELECT COUNT(*) FROM tracks WHERE album_id = ?1", [&album_id], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if remaining == 0 {
            tx.execute("DELETE FROM albums WHERE id = ?1", [&album_id]).map_err(|e| e.to_string())?;
            deleted_albums.push(album_id);
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(deleted_albums)
}

/// Appends `stem` to a track's stem list (a no-op if a stem with that name
/// is already recorded) — used after stem separation finishes.
pub fn add_track_stem(conn: &Connection, track_id: &str, stem: &crate::stem_separator::Stem) -> Result<(), String> {
    let stems_json: Option<String> =
        conn.query_row("SELECT stems FROM tracks WHERE id = ?1", [track_id], |r| r.get(0)).map_err(|e| e.to_string())?;
    let mut stems: Vec<crate::stem_separator::Stem> =
        stems_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    if stems.iter().any(|s| s.name == stem.name) {
        return Ok(());
    }
    stems.push(stem.clone());
    conn.execute(
        "UPDATE tracks SET stems = ?1 WHERE id = ?2",
        rusqlite::params![serde_json::to_string(&stems).unwrap(), track_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Collapses the several near-duplicate "toggle favourite" implementations
/// that used to exist across the frontend (some did `db.songs.update(song,
/// {isFavourite})`, others read-modified-wrote the whole `$current.song`
/// object via `.put()`) into one canonical partial update.
pub fn update_track_favourite(conn: &Connection, track_id: &str, is_favourite: bool) -> Result<(), String> {
    conn.execute(
        "UPDATE tracks SET is_favourite = ?1 WHERE id = ?2",
        rusqlite::params![is_favourite as i64, track_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Atomic increment — avoids the old frontend's read-then-write race
/// (`db.songs.update(song, {playCount: song.playCount + 1})` using a
/// possibly-stale in-memory `song.playCount`).
pub fn increment_track_play_count(conn: &Connection, track_id: &str) -> Result<i64, String> {
    conn.execute("UPDATE tracks SET play_count = play_count + 1 WHERE id = ?1", [track_id])
        .map_err(|e| e.to_string())?;
    conn.query_row("SELECT play_count FROM tracks WHERE id = ?1", [track_id], |r| r.get(0)).map_err(|e| e.to_string())
}

/// Replaces a track's tag set.
pub fn update_track_tags(conn: &Connection, track_id: &str, tags: &[String]) -> Result<(), String> {
    conn.execute("DELETE FROM track_tags WHERE track_id = ?1", [track_id]).map_err(|e| e.to_string())?;
    for tag in tags {
        conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?1)", [tag]).map_err(|e| e.to_string())?;
        let tag_id: i64 =
            conn.query_row("SELECT id FROM tags WHERE name = ?1", [tag], |r| r.get(0)).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO track_tags (track_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![track_id, tag_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn update_track_origin_country(conn: &Connection, track_id: &str, origin_country: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE tracks SET origin_country = ?1 WHERE id = ?2",
        rusqlite::params![origin_country, track_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_track_markers(conn: &Connection, track_id: &str, markers: &[crate::metadata::Marker]) -> Result<(), String> {
    conn.execute(
        "UPDATE tracks SET markers = ?1 WHERE id = ?2",
        rusqlite::params![serde_json::to_string(markers).unwrap(), track_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
