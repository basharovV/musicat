//! CRUD and filesystem-folder reconciliation for the Artist's Toolkit
//! (`song_projects`, `artist_projects`, `scrapbook`) — see
//! `openspec/changes/artists-toolkit-storage/design.md`. Mirrors
//! `db/playlists.rs`'s structure: plain functions over `&Connection`/
//! `&mut Connection`, wrapped as Tauri commands in `db/commands.rs`.

use rusqlite::Connection;

use super::models::{
    row_to_artist_project, row_to_scrapbook_item, row_to_song_project, ArtistProject,
    ScrapbookItem, SongProjectRow, ARTIST_PROJECT_COLUMNS, SCRAPBOOK_COLUMNS, SONG_PROJECT_COLUMNS,
};

fn unique_violation_message(err: &rusqlite::Error, message: &str) -> String {
    if err.to_string().contains("UNIQUE constraint failed") {
        message.to_string()
    } else {
        err.to_string()
    }
}

// ---------------------------------------------------------------------------
// Artist projects
// ---------------------------------------------------------------------------

fn get_artist_project(conn: &Connection, id: i64) -> rusqlite::Result<ArtistProject> {
    conn.query_row(
        &format!("SELECT {ARTIST_PROJECT_COLUMNS} FROM artist_projects WHERE id = ?1"),
        [id],
        row_to_artist_project,
    )
}

/// Reconciles `artist_projects` against a folder scan's discovered artist names: creates a row
/// for every name that doesn't have one yet, deletes any row whose name is no longer discovered
/// (cascading to that artist's `song_projects`), and returns the full reconciled list. Mirrors
/// `db/playlists.rs`'s `ensure_playlist_tracked` — "the folder is the entity" — generalized with
/// the delete-stale half `scanScrapbook` already does for scrapbook files.
pub fn sync_artist_projects(
    conn: &mut Connection,
    discovered_names: &[String],
) -> rusqlite::Result<Vec<ArtistProject>> {
    let tx = conn.transaction()?;
    for name in discovered_names {
        tx.execute(
            "INSERT INTO artist_projects (name, members) VALUES (?1, '[]')
             ON CONFLICT(name) DO NOTHING",
            [name],
        )?;
    }
    {
        let mut stmt = tx.prepare("SELECT id, name FROM artist_projects")?;
        let existing: Vec<(i64, String)> =
            stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<rusqlite::Result<_>>()?;
        for (id, name) in existing {
            if !discovered_names.iter().any(|n| n == &name) {
                tx.execute("DELETE FROM artist_projects WHERE id = ?1", [id])?;
            }
        }
    }
    let result = {
        let mut stmt = tx.prepare(&format!(
            "SELECT {ARTIST_PROJECT_COLUMNS} FROM artist_projects ORDER BY name"
        ))?;
        let rows = stmt.query_map([], row_to_artist_project)?.collect::<rusqlite::Result<Vec<_>>>()?;
        rows
    };
    tx.commit()?;
    Ok(result)
}

pub fn update_artist_members(
    conn: &Connection,
    id: i64,
    members: &[String],
) -> rusqlite::Result<ArtistProject> {
    conn.execute(
        "UPDATE artist_projects SET members = ?1 WHERE id = ?2",
        rusqlite::params![serde_json::to_string(members).unwrap(), id],
    )?;
    get_artist_project(conn, id)
}

/// Updates `name` in place, preserving the row's id (and therefore its `members` and every
/// `song_projects` row referencing it via `artist_id`) — see design.md's "Identity: surrogate
/// keys, reconciled against folder scans, renamed in place" decision.
pub fn rename_artist_project(conn: &Connection, id: i64, new_name: &str) -> Result<ArtistProject, String> {
    conn.execute("UPDATE artist_projects SET name = ?1 WHERE id = ?2", rusqlite::params![new_name, id])
        .map_err(|e| unique_violation_message(&e, &format!("an artist named '{new_name}' already exists")))?;
    get_artist_project(conn, id).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Song projects
// ---------------------------------------------------------------------------

fn get_song_project(conn: &Connection, id: i64) -> rusqlite::Result<SongProjectRow> {
    conn.query_row(
        &format!("SELECT {SONG_PROJECT_COLUMNS} FROM song_projects WHERE id = ?1"),
        [id],
        row_to_song_project,
    )
}

fn get_song_project_by_natural_key(
    conn: &Connection,
    artist_id: i64,
    title: &str,
) -> rusqlite::Result<SongProjectRow> {
    conn.query_row(
        &format!("SELECT {SONG_PROJECT_COLUMNS} FROM song_projects WHERE artist_id = ?1 AND title = ?2"),
        rusqlite::params![artist_id, title],
        row_to_song_project,
    )
}

/// Reconciles an artist's `song_projects` against a folder scan's discovered `(title,
/// song_filepath)` pairs, same shape as `sync_artist_projects`. Returns titles only, matching
/// `loadSongProjectsForArtist`'s existing contract — full project bodies are loaded individually
/// via `ensure_song_project`/`get_song_project`.
pub fn sync_song_projects_for_artist(
    conn: &mut Connection,
    artist_id: i64,
    discovered: &[(String, String)],
) -> rusqlite::Result<Vec<String>> {
    let tx = conn.transaction()?;
    for (title, song_filepath) in discovered {
        tx.execute(
            "INSERT INTO song_projects (artist_id, title, song_filepath) VALUES (?1, ?2, ?3)
             ON CONFLICT(artist_id, title) DO UPDATE SET song_filepath = excluded.song_filepath",
            rusqlite::params![artist_id, title, song_filepath],
        )?;
    }
    {
        let mut stmt = tx.prepare("SELECT id, title FROM song_projects WHERE artist_id = ?1")?;
        let existing: Vec<(i64, String)> = stmt
            .query_map([artist_id], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        for (id, title) in existing {
            if !discovered.iter().any(|(t, _)| t == &title) {
                tx.execute("DELETE FROM song_projects WHERE id = ?1", [id])?;
            }
        }
    }
    let titles: Vec<String> = {
        let mut stmt = tx.prepare("SELECT title FROM song_projects WHERE artist_id = ?1 ORDER BY title")?;
        let rows = stmt.query_map([artist_id], |r| r.get(0))?.collect::<rusqlite::Result<_>>()?;
        rows
    };
    tx.commit()?;
    Ok(titles)
}

/// Find-or-create by `(artist_id, title)`. Unlike `sync_song_projects_for_artist`, this doesn't
/// delete anything — it's used by `loadSongProject` (loading a single song), including right
/// after `createSongProject` writes a brand-new file, before any list-level scan has had a chance
/// to reconcile it. See design.md's "Find-or-create on individual song load" decision.
pub fn ensure_song_project(
    conn: &Connection,
    artist_id: i64,
    title: &str,
    song_filepath: &str,
) -> rusqlite::Result<SongProjectRow> {
    conn.execute(
        "INSERT INTO song_projects (artist_id, title, song_filepath) VALUES (?1, ?2, ?3)
         ON CONFLICT(artist_id, title) DO UPDATE SET song_filepath = excluded.song_filepath",
        rusqlite::params![artist_id, title, song_filepath],
    )?;
    get_song_project_by_natural_key(conn, artist_id, title)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongProjectUpdate {
    pub album: Option<String>,
    pub key: Option<String>,
    pub bpm: Option<f64>,
    #[serde(default)]
    pub music_composed_by: Vec<String>,
    #[serde(default)]
    pub lyrics_written_by: Vec<String>,
    pub artwork_file_path: Option<String>,
    pub lyrics: Option<String>,
    #[serde(default)]
    pub recordings: serde_json::Value,
    #[serde(default)]
    pub other_content_items: serde_json::Value,
    pub song_id: Option<String>,
}

pub fn update_song_project(
    conn: &Connection,
    id: i64,
    update: &SongProjectUpdate,
) -> rusqlite::Result<SongProjectRow> {
    conn.execute(
        "UPDATE song_projects SET album = ?1, key = ?2, bpm = ?3, music_composed_by = ?4,
            lyrics_written_by = ?5, artwork_file_path = ?6, lyrics = ?7, recordings = ?8,
            other_content_items = ?9, song_id = ?10
         WHERE id = ?11",
        rusqlite::params![
            update.album,
            update.key,
            update.bpm,
            serde_json::to_string(&update.music_composed_by).unwrap(),
            serde_json::to_string(&update.lyrics_written_by).unwrap(),
            update.artwork_file_path,
            update.lyrics,
            update.recordings.to_string(),
            update.other_content_items.to_string(),
            update.song_id,
            id,
        ],
    )?;
    get_song_project(conn, id)
}

/// Updates `title`/`song_filepath` in place, preserving the row's id (and its `recordings`/
/// `otherContentItems`) — the song-project counterpart to `rename_artist_project`.
pub fn rename_song_project(
    conn: &Connection,
    id: i64,
    new_title: &str,
    new_song_filepath: &str,
) -> Result<SongProjectRow, String> {
    conn.execute(
        "UPDATE song_projects SET title = ?1, song_filepath = ?2 WHERE id = ?3",
        rusqlite::params![new_title, new_song_filepath, id],
    )
    .map_err(|e| {
        unique_violation_message(&e, &format!("a song named '{new_title}' already exists for this artist"))
    })?;
    get_song_project(conn, id).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Scrapbook
// ---------------------------------------------------------------------------

fn get_scrapbook_item(conn: &Connection, id: &str) -> rusqlite::Result<ScrapbookItem> {
    conn.query_row(
        &format!("SELECT {SCRAPBOOK_COLUMNS} FROM scrapbook WHERE id = ?1"),
        [id],
        row_to_scrapbook_item,
    )
}

fn upsert_scrapbook_item_preserving_tags(
    tx: &rusqlite::Transaction,
    item: &ScrapbookItem,
) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT INTO scrapbook (id, type, name, tags, song_id, url, image_url, file_type_type,
            file_type_extension, path)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
         ON CONFLICT(id) DO UPDATE SET
            type = excluded.type, name = excluded.name, song_id = excluded.song_id,
            url = excluded.url, image_url = excluded.image_url,
            file_type_type = excluded.file_type_type,
            file_type_extension = excluded.file_type_extension, path = excluded.path",
        rusqlite::params![
            item.id,
            item.kind,
            item.name,
            serde_json::to_string(&item.tags).unwrap(),
            item.song_id,
            item.url,
            item.image_url,
            item.file_type.as_ref().map(|f| f.kind.clone()),
            item.file_type.as_ref().map(|f| f.extension.clone()),
            item.path,
        ],
    )?;
    Ok(())
}

/// Reconciles `scrapbook` against a folder scan's discovered files: upserts each discovered item
/// (never overwriting an existing row's `tags` — the `ON CONFLICT` clause omits that column, same
/// as the Dexie `scanScrapbook` restore-then-merge step it replaces), deletes rows for files no
/// longer present, and returns the full reconciled list.
pub fn sync_scrapbook_items(
    conn: &mut Connection,
    discovered: &[ScrapbookItem],
) -> rusqlite::Result<Vec<ScrapbookItem>> {
    let tx = conn.transaction()?;
    for item in discovered {
        upsert_scrapbook_item_preserving_tags(&tx, item)?;
    }
    {
        let mut stmt = tx.prepare("SELECT id FROM scrapbook")?;
        let existing: Vec<String> = stmt.query_map([], |r| r.get(0))?.collect::<rusqlite::Result<_>>()?;
        for id in existing {
            if !discovered.iter().any(|i| i.id == id) {
                tx.execute("DELETE FROM scrapbook WHERE id = ?1", [&id])?;
            }
        }
    }
    let result = {
        let mut stmt = tx.prepare(&format!("SELECT {SCRAPBOOK_COLUMNS} FROM scrapbook ORDER BY name"))?;
        let rows = stmt.query_map([], row_to_scrapbook_item)?.collect::<rusqlite::Result<Vec<_>>>()?;
        rows
    };
    tx.commit()?;
    Ok(result)
}

/// For pasted links (`Scrapbook.svelte`'s `addLink`) — not folder-discovered, so not covered by
/// `sync_scrapbook_items`.
pub fn add_scrapbook_item(conn: &Connection, item: &ScrapbookItem) -> rusqlite::Result<ScrapbookItem> {
    let tx_conn = conn;
    tx_conn.execute(
        "INSERT INTO scrapbook (id, type, name, tags, song_id, url, image_url, file_type_type,
            file_type_extension, path)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
         ON CONFLICT(id) DO UPDATE SET
            type = excluded.type, name = excluded.name, tags = excluded.tags,
            song_id = excluded.song_id, url = excluded.url, image_url = excluded.image_url,
            file_type_type = excluded.file_type_type,
            file_type_extension = excluded.file_type_extension, path = excluded.path",
        rusqlite::params![
            item.id,
            item.kind,
            item.name,
            serde_json::to_string(&item.tags).unwrap(),
            item.song_id,
            item.url,
            item.image_url,
            item.file_type.as_ref().map(|f| f.kind.clone()),
            item.file_type.as_ref().map(|f| f.extension.clone()),
            item.path,
        ],
    )?;
    get_scrapbook_item(tx_conn, &item.id)
}

pub fn update_scrapbook_tags(conn: &Connection, id: &str, tags: &[String]) -> rusqlite::Result<ScrapbookItem> {
    conn.execute(
        "UPDATE scrapbook SET tags = ?1 WHERE id = ?2",
        rusqlite::params![serde_json::to_string(tags).unwrap(), id],
    )?;
    get_scrapbook_item(conn, id)
}

pub fn delete_scrapbook_item(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM scrapbook WHERE id = ?1", [id])?;
    Ok(())
}
