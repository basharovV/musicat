//! Playlist CRUD plus the file-divergence detection described in
//! `openspec/changes/sqlite-storage-foundation/design.md` ("Playlist file
//! divergence: detected via content hash, resolved by prompting — never
//! silent") and specced in
//! `specs/library-storage/spec.md` ("Playlist file divergence detection").

use std::time::{SystemTime, UNIX_EPOCH};

use crc::{Crc, CRC_32_ISO_HDLC};
use rusqlite::Connection;
use serde_m3u::Playlist as M3uPlaylist;

const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

/// Content-hash of a playlist file's bytes. Change detection, not integrity —
/// a fast, already-vendored CRC32 is enough (see the design decision above).
pub fn hash_file_contents(bytes: &[u8]) -> String {
    format!("{:08x}", CRC32.checksum(bytes))
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

// No command wraps this yet — playlist creation lands with the
// Dexie-migration/mutation commands (tasks.md groups 5-6). Kept here since
// `set_playlist_tracks`/`reimport_playlist_from_file` already depend on rows
// this inserts.
#[allow(dead_code)]
pub fn create_playlist(
    conn: &mut Connection,
    id: &str,
    title: &str,
    file_path: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO playlists (id, title, file_path) VALUES (?1, ?2, ?3)",
        rusqlite::params![id, title, file_path],
    )?;
    Ok(())
}

/// Replaces a playlist's track list, in order. Used both for normal edits
/// (reordering, add/remove) and for reimporting from a diverged file.
pub fn set_playlist_tracks(
    conn: &mut Connection,
    playlist_id: &str,
    track_ids: &[String],
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM playlist_tracks WHERE playlist_id = ?1", [playlist_id])?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
        )?;
        for (position, track_id) in track_ids.iter().enumerate() {
            stmt.execute(rusqlite::params![playlist_id, track_id, position as i64])?;
        }
    }
    tx.commit()
}

/// Call after the app itself writes `content` to `playlist_id`'s linked file
/// (export, reorder, etc). Stores the hash of what was just written so the
/// watcher's next scan recognizes it as our own write, not external drift —
/// see "App-initiated write does not self-trigger a prompt" in the spec.
pub fn record_own_write(conn: &Connection, playlist_id: &str, content: &[u8]) -> rusqlite::Result<()> {
    let hash = hash_file_contents(content);
    conn.execute(
        "UPDATE playlists SET content_hash = ?1, last_synced_at = ?2, is_diverged = 0 WHERE id = ?3",
        rusqlite::params![hash, now_ms(), playlist_id],
    )?;
    Ok(())
}

/// Called by the playlists-folder watcher on every change event for a linked
/// file. Returns `true` if this is external drift the frontend should prompt
/// about (i.e. the file's hash no longer matches what the app last wrote).
pub fn check_for_divergence(conn: &Connection, playlist_id: &str, content: &[u8]) -> rusqlite::Result<bool> {
    let stored_hash: Option<String> = conn
        .query_row(
            "SELECT content_hash FROM playlists WHERE id = ?1",
            [playlist_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    let current_hash = hash_file_contents(content);
    let diverged = stored_hash.as_deref() != Some(current_hash.as_str());
    if diverged {
        conn.execute(
            "UPDATE playlists SET is_diverged = 1 WHERE id = ?1",
            [playlist_id],
        )?;
    }
    Ok(diverged)
}

/// Replaces `playlist_id`'s tracks from its linked file's current contents
/// (the user accepted a reimport prompt), matching each M3U entry to a track
/// by exact path. Entries with no matching track are skipped, not fatal —
/// mirrors how playlist scanning already tolerates missing files elsewhere
/// in the app (`scan_playlist` in `metadata.rs`).
pub fn reimport_playlist_from_file(conn: &mut Connection, playlist_id: &str) -> Result<usize, String> {
    let file_path: Option<String> = conn
        .query_row(
            "SELECT file_path FROM playlists WHERE id = ?1",
            [playlist_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let file_path = file_path.ok_or_else(|| "playlist has no linked file".to_string())?;

    let content = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&content);
    let parsed = M3uPlaylist::from(text.as_ref());

    let mut track_ids = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT id FROM tracks WHERE path = ?1")
            .map_err(|e| e.to_string())?;
        for entry in &parsed.list {
            if let Ok(id) = stmt.query_row([&entry.url], |row| row.get::<_, String>(0)) {
                track_ids.push(id);
            }
        }
    }

    set_playlist_tracks(conn, playlist_id, &track_ids).map_err(|e| e.to_string())?;
    record_own_write(conn, playlist_id, &content).map_err(|e| e.to_string())?;

    Ok(track_ids.len())
}
