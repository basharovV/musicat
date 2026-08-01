//! Tauri command surface for the storage/query foundation. Every DB call is
//! pushed onto a blocking thread (`spawn_blocking`) per the "idiomatic Rust
//! concurrency" decision — reads via the pool, writes via the single writer
//! actor — so the async executor is never blocked on SQLite I/O.
//!
//! Not wired into the frontend yet: this change ships the foundation only
//! (see `openspec/changes/sqlite-storage-foundation/design.md`, Migration
//! Plan step 1). Call sites land with the "Frontend integration" tasks.

use log::error;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::metadata::{Album, Song};

use super::migration::{MigrationReport, VerificationReport};
use super::models::Playlist;
use super::query::{Page, QuerySpec};
use super::settings::Settings;
use super::Database;

#[tauri::command]
pub async fn query_tracks(spec: QuerySpec, db: State<'_, Database>) -> Result<Page<Song>, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::query::query_tracks(&conn, &spec)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn query_albums(spec: QuerySpec, db: State<'_, Database>) -> Result<Page<Album>, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::query::query_albums(&conn, &spec)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn query_playlists(spec: QuerySpec, db: State<'_, Database>) -> Result<Page<Playlist>, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::query::query_playlists(&conn, &spec)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Insert-or-refresh freshly-scanned tracks. See `db::mutations` for what
/// "refresh" preserves on an already-existing track.
#[tauri::command]
pub async fn upsert_tracks(tracks: Vec<Song>, db: State<'_, Database>) -> Result<Vec<Song>, String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || writer.execute(move |conn| super::mutations::upsert_tracks(conn, &tracks)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn upsert_albums(albums: Vec<Album>, db: State<'_, Database>) -> Result<Vec<Album>, String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || writer.execute(move |conn| super::mutations::upsert_albums(conn, &albums)))
        .await
        .map_err(|e| e.to_string())?
}

/// Deletes tracks and any album left with none of its tracks remaining.
/// Returns the ids of albums that were deleted as a result.
#[tauri::command]
pub async fn delete_tracks(track_ids: Vec<String>, db: State<'_, Database>) -> Result<Vec<String>, String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || writer.execute(move |conn| super::mutations::delete_tracks(conn, &track_ids)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn add_track_stem(
    track_id: String,
    stem: crate::stem_separator::Stem,
    db: State<'_, Database>,
) -> Result<(), String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || writer.execute(move |conn| super::mutations::add_track_stem(conn, &track_id, &stem)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_smart_queries(db: State<'_, Database>) -> Result<Vec<super::smart_queries::SmartQuerySummary>, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::smart_queries::list_smart_queries(&conn)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_smart_query(id: i64, db: State<'_, Database>) -> Result<Option<super::smart_queries::SmartQueryRow>, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::smart_queries::get_smart_query(&conn, id)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn save_smart_query(
    id: Option<i64>,
    name: String,
    query_parts: serde_json::Value,
    db: State<'_, Database>,
) -> Result<i64, String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        writer.execute(move |conn| super::smart_queries::save_smart_query(conn, id, &name, &query_parts))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_smart_query(id: i64, db: State<'_, Database>) -> Result<(), String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || writer.execute(move |conn| super::smart_queries::delete_smart_query(conn, id)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_track_favourite(track_id: String, is_favourite: bool, db: State<'_, Database>) -> Result<(), String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        writer.execute(move |conn| super::mutations::update_track_favourite(conn, &track_id, is_favourite))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn increment_track_play_count(track_id: String, db: State<'_, Database>) -> Result<i64, String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || writer.execute(move |conn| super::mutations::increment_track_play_count(conn, &track_id)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_track_tags(track_id: String, tags: Vec<String>, db: State<'_, Database>) -> Result<(), String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || writer.execute(move |conn| super::mutations::update_track_tags(conn, &track_id, &tags)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_track_origin_country(track_id: String, origin_country: String, db: State<'_, Database>) -> Result<(), String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        writer.execute(move |conn| super::mutations::update_track_origin_country(conn, &track_id, &origin_country))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_track_markers(track_id: String, markers: Vec<crate::metadata::Marker>, db: State<'_, Database>) -> Result<(), String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || writer.execute(move |conn| super::mutations::update_track_markers(conn, &track_id, &markers)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_distinct_track_values(field: String, db: State<'_, Database>) -> Result<Vec<String>, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::query::get_distinct_track_values(&conn, &field)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_library_counts(db: State<'_, Database>) -> Result<super::query::LibraryCounts, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::query::get_library_counts(&conn)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_playlist_by_id(id: String, db: State<'_, Database>) -> Result<Option<Playlist>, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::query::get_playlist_by_id(&conn, &id)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn upsert_playlist_header(
    id: String,
    title: String,
    file_path: Option<String>,
    db: State<'_, Database>,
) -> Result<(), String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        writer.execute(move |conn| super::playlists::upsert_playlist_header(conn, &id, &title, file_path.as_deref()))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_playlist_tracks(
    playlist_id: String,
    track_ids: Vec<String>,
    db: State<'_, Database>,
) -> Result<(), String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        writer.execute(move |conn| super::playlists::set_playlist_tracks(conn, &playlist_id, &track_ids))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_playlist(id: String, db: State<'_, Database>) -> Result<(), String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || writer.execute(move |conn| super::playlists::delete_playlist(conn, &id)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_track_by_id(id: String, db: State<'_, Database>) -> Result<Option<Song>, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::query::get_track_by_id(&conn, &id)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_tracks_by_ids(ids: Vec<String>, db: State<'_, Database>) -> Result<Vec<Option<Song>>, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::query::get_tracks_by_ids(&conn, &ids)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_album_by_id(id: String, db: State<'_, Database>) -> Result<Option<Album>, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::query::get_album_by_id(&conn, &id)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_albums_by_ids(ids: Vec<String>, db: State<'_, Database>) -> Result<Vec<Option<Album>>, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::query::get_albums_by_ids(&conn, &ids)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_settings(db: State<'_, Database>) -> Result<Settings, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::settings::get_settings(&conn).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_settings(settings: Settings, db: State<'_, Database>) -> Result<(), String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        writer.execute(move |conn| super::settings::update_settings(conn, &settings))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_queue(db: State<'_, Database>) -> Result<Vec<String>, String> {
    let pool = db.pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        super::queue::get_queue(&conn).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_queue(track_ids: Vec<String>, db: State<'_, Database>) -> Result<(), String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        writer.execute(move |conn| super::queue::set_queue(conn, &track_ids))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistDivergedEvent {
    pub playlist_id: String,
}

/// Called by the playlists-folder watcher (today driven from the frontend,
/// see `src/data/FolderWatcher.ts`) whenever a linked `.m3u` file changes.
/// Emits `playlist_diverged` if the change didn't originate from the app
/// itself — see the "Playlist file divergence detection" requirement.
#[tauri::command]
pub async fn check_playlist_divergence(
    playlist_id: String,
    file_path: String,
    db: State<'_, Database>,
    app: AppHandle,
) -> Result<bool, String> {
    let writer = db.writer.clone();
    let event_playlist_id = playlist_id.clone();
    let diverged = tauri::async_runtime::spawn_blocking(move || {
        let content = std::fs::read(&file_path).map_err(|e| e.to_string())?;
        writer.execute(move |conn| super::playlists::check_for_divergence(conn, &playlist_id, &content))
    })
    .await
    .map_err(|e| e.to_string())??;

    if diverged {
        if let Err(e) = app.emit("playlist_diverged", PlaylistDivergedEvent { playlist_id: event_playlist_id }) {
            error!("[db] failed to emit playlist_diverged: {e}");
        }
    }
    Ok(diverged)
}

/// Replaces `playlist_id`'s tracks from its linked file's current contents —
/// the user accepted a reimport prompt after `playlist_diverged` fired.
#[tauri::command]
pub async fn reimport_playlist_from_file(playlist_id: String, db: State<'_, Database>) -> Result<usize, String> {
    let writer = db.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        writer.execute(move |conn| super::playlists::reimport_playlist_from_file(conn, &playlist_id))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationOutcome {
    pub report: MigrationReport,
    pub verification: VerificationReport,
}

/// Parses a `dexie-export-import` export (produced headlessly by the
/// frontend calling `exportDB(db)` — see `src/data/db.ts`) and writes it into
/// a shadow SQLite file, separate from the live database. Does not touch the
/// live database at all; call `cutover_migrated_database` once the caller has
/// inspected `verification.ok`.
#[tauri::command]
pub async fn run_shadow_migration_from_dexie(export_json: String, app: AppHandle) -> Result<MigrationOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let export: super::migration::DexieExport =
            serde_json::from_str(&export_json).map_err(|e| format!("invalid Dexie export: {e}"))?;

        let shadow_path = super::shadow_db_path(&app)?;
        if shadow_path.exists() {
            std::fs::remove_file(&shadow_path).map_err(|e| e.to_string())?;
        }
        let mut shadow_conn = rusqlite::Connection::open(&shadow_path).map_err(|e| e.to_string())?;
        super::schema::run_migrations(&mut shadow_conn).map_err(|e| e.to_string())?;

        let report = super::migration::migrate(&mut shadow_conn, &export)?;
        let verification = super::migration::verify(&export, &shadow_conn)?;

        Ok(MigrationOutcome { report, verification })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Folds a shadow-migrated, verified database into the live one. Errors if
/// there's no shadow file (i.e. `run_shadow_migration_from_dexie` hasn't run)
/// — callers are expected to have checked `verification.ok` themselves before
/// calling this, since a failed verification is not this function's problem
/// to detect twice.
#[tauri::command]
pub async fn cutover_migrated_database(db: State<'_, Database>, app: AppHandle) -> Result<(), String> {
    let shadow_path = super::shadow_db_path(&app)?;
    if !shadow_path.exists() {
        return Err("no pending migration: run_shadow_migration_from_dexie hasn't produced a shadow database".to_string());
    }

    let writer = db.writer.clone();
    let cutover_path = shadow_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        writer.execute(move |conn| super::migration::cutover(conn, &cutover_path))?;
        writer.execute(move |conn| {
            let mut settings = super::settings::get_settings(conn)?;
            settings.migrated_from_dexie_at = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64,
            );
            super::settings::update_settings(conn, &settings)
        })
    })
    .await
    .map_err(|e| e.to_string())??;

    std::fs::remove_file(&shadow_path).map_err(|e| e.to_string())?;
    Ok(())
}
