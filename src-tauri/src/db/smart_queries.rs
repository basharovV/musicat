//! CRUD for saved smart queries. The predicate shape itself (`queryParts`)
//! stays owned by the frontend (tasks.md 3.6, porting it into `QuerySpec`,
//! is separate work) — this just persists whatever JSON blob it hands us and
//! hands it back, plus the `name` column Rust needs to list/sort by.

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartQuerySummary {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartQueryRow {
    pub id: i64,
    pub name: String,
    pub query_parts: serde_json::Value,
}

pub fn list_smart_queries(conn: &Connection) -> Result<Vec<SmartQuerySummary>, String> {
    let mut stmt = conn.prepare("SELECT id, name FROM smart_queries ORDER BY name").map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| Ok(SmartQuerySummary { id: row.get(0)?, name: row.get(1)? }))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn get_smart_query(conn: &Connection, id: i64) -> Result<Option<SmartQueryRow>, String> {
    conn.query_row("SELECT id, name, query_json FROM smart_queries WHERE id = ?1", [id], |row| {
        let query_json: String = row.get(2)?;
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, query_json))
    })
    .optional()
    .map_err(|e| e.to_string())?
    .map(|(id, name, query_json)| {
        Ok(SmartQueryRow {
            id,
            name,
            query_parts: serde_json::from_str(&query_json).map_err(|e| e.to_string())?,
        })
    })
    .transpose()
}

/// Insert (no `id`) or update (`id` given) a saved smart query. Returns its id.
pub fn save_smart_query(
    conn: &Connection,
    id: Option<i64>,
    name: &str,
    query_parts: &serde_json::Value,
) -> Result<i64, String> {
    let query_json = serde_json::to_string(query_parts).map_err(|e| e.to_string())?;
    match id {
        Some(id) => {
            conn.execute(
                "UPDATE smart_queries SET name = ?1, query_json = ?2 WHERE id = ?3",
                rusqlite::params![name, query_json, id],
            )
            .map_err(|e| e.to_string())?;
            Ok(id)
        }
        None => {
            conn.execute(
                "INSERT INTO smart_queries (name, query_json) VALUES (?1, ?2)",
                rusqlite::params![name, query_json],
            )
            .map_err(|e| e.to_string())?;
            Ok(conn.last_insert_rowid())
        }
    }
}

pub fn delete_smart_query(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM smart_queries WHERE id = ?1", [id]).map_err(|e| e.to_string())?;
    Ok(())
}
