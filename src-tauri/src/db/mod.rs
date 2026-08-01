//! The SQLite-backed storage/query foundation from
//! `openspec/changes/sqlite-storage-foundation/`. This is the `library-storage`
//! and `library-query` capabilities: a single Rust-owned database, a
//! versioned migration runner, a pooled-readers/single-writer connection
//! strategy, and the generic `QuerySpec`/`Page<T>` query interface.
//!
//! Additive only for now — see `commands.rs` for why nothing here is called
//! from the frontend yet.

pub mod commands;
pub mod models;
pub mod playlists;
pub mod pool;
pub mod query;
pub mod queue;
pub mod schema;
pub mod settings;
#[cfg(test)]
mod tests;
mod writer;

use std::path::PathBuf;

use log::info;
use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use pool::{build_pool, Pool};
use writer::WriterHandle;

pub struct Database {
    pub pool: Pool,
    pub writer: WriterHandle,
}

impl Database {
    /// Opens (creating if needed) the app's SQLite database, enables WAL
    /// mode, applies any pending migrations, and starts the writer actor.
    /// Call once at startup and `.manage()` the result as Tauri app state.
    pub fn init(app: &AppHandle) -> Result<Self, String> {
        let db_path = db_path(app)?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let mut writer_conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
        writer_conn
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| e.to_string())?;
        writer_conn
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| e.to_string())?;
        schema::run_migrations(&mut writer_conn).map_err(|e| e.to_string())?;

        let pool = build_pool(db_path).map_err(|e| e.to_string())?;
        let writer = WriterHandle::spawn(writer_conn);

        info!("[db] SQLite storage foundation ready");
        Ok(Self { pool, writer })
    }
}

fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let file_name = if cfg!(debug_assertions) {
        "musicat-dev.sqlite3"
    } else {
        "musicat.sqlite3"
    };
    Ok(data_dir.join(file_name))
}
