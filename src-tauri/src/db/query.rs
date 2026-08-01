//! The generic `QuerySpec`/`Page<T>` query interface described in
//! `openspec/changes/sqlite-storage-foundation/specs/library-query/spec.md`:
//! a serializable filter/sort/pagination request, interpreted into
//! parameterized SQL against an explicit per-entity field whitelist — no
//! query path ever string-concatenates a client-supplied field name, operator,
//! or value into SQL text.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rusqlite::{Connection, ToSql};
use serde::{Deserialize, Serialize};

use crate::metadata::{Album, Song};

use super::models::{
    row_to_album, row_to_playlist_header, row_to_track, Playlist, ALBUM_COLUMNS, TRACK_COLUMNS,
};

// ---------------------------------------------------------------------------
// Request/response shapes shared by every entity's query command
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    Text(String),
    Int(i64),
    Real(f64),
    Bool(bool),
}

impl ToSql for FilterValue {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            FilterValue::Text(s) => s.to_sql(),
            FilterValue::Int(i) => i.to_sql(),
            FilterValue::Real(r) => r.to_sql(),
            FilterValue::Bool(b) => b.to_sql(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CmpOp {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
}

impl CmpOp {
    fn sql(self) -> &'static str {
        match self {
            CmpOp::Eq => "=",
            CmpOp::Ne => "!=",
            CmpOp::Gt => ">",
            CmpOp::Gte => ">=",
            CmpOp::Lt => "<",
            CmpOp::Lte => "<=",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum FilterNode {
    And { nodes: Vec<FilterNode> },
    Or { nodes: Vec<FilterNode> },
    Cmp { field: String, op: CmpOp, value: FilterValue },
    Between { field: String, min: FilterValue, max: FilterValue },
    Contains { field: String, value: String },
    NotEmpty { field: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortKey {
    pub field: String,
    #[serde(default)]
    pub descending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRequest {
    /// Opaque keyset cursor from a previous `Page::next_cursor`. Used for
    /// tracks. Ignored by offset-paginated entities.
    #[serde(default)]
    pub cursor: Option<String>,
    /// Used for offset-paginated entities (albums, playlists).
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    100
}

// A derived `Default` would give `limit: 0` (the `u32` zero value), silently
// diverging from the `#[serde(default = "default_limit")]` used when a
// `QuerySpec` arrives over IPC as JSON. Implemented by hand so both paths
// agree — Rust-constructed and JSON-deserialized `PageRequest`s behave the
// same way.
impl Default for PageRequest {
    fn default() -> Self {
        Self { cursor: None, offset: None, limit: default_limit() }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySpec {
    #[serde(default)]
    pub filter: Option<FilterNode>,
    #[serde(default)]
    pub sort: Option<SortKey>,
    #[serde(default)]
    pub page: PageRequest,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

// ---------------------------------------------------------------------------
// Per-entity field whitelist — the only thing a `QuerySpec` is allowed to
// reference. Unknown fields are rejected before any SQL is built.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldKind {
    Text,
    Int,
    Real,
    Bool,
    /// A `'[]'`-default JSON array column — only meaningful for `NotEmpty`.
    JsonArray,
    /// Not a real column: resolved via a join/subquery against `track_tags`.
    TagJoin,
}

struct FieldSpec {
    name: &'static str,
    column: &'static str,
    kind: FieldKind,
    sortable: bool,
}

const TRACK_FIELDS: &[FieldSpec] = &[
    FieldSpec { name: "title", column: "title", kind: FieldKind::Text, sortable: true },
    FieldSpec { name: "artist", column: "artist", kind: FieldKind::Text, sortable: true },
    FieldSpec { name: "album", column: "album", kind: FieldKind::Text, sortable: true },
    FieldSpec { name: "albumArtist", column: "album_artist", kind: FieldKind::Text, sortable: true },
    FieldSpec { name: "albumId", column: "album_id", kind: FieldKind::Text, sortable: false },
    FieldSpec { name: "composer", column: "composer", kind: FieldKind::Text, sortable: false },
    FieldSpec { name: "genre", column: "genre", kind: FieldKind::Text, sortable: false },
    FieldSpec { name: "year", column: "year", kind: FieldKind::Int, sortable: true },
    FieldSpec { name: "trackNumber", column: "track_number", kind: FieldKind::Int, sortable: true },
    FieldSpec { name: "duration", column: "duration_secs", kind: FieldKind::Real, sortable: true },
    FieldSpec { name: "isFavourite", column: "is_favourite", kind: FieldKind::Bool, sortable: true },
    FieldSpec { name: "originCountry", column: "origin_country", kind: FieldKind::Text, sortable: true },
    FieldSpec { name: "dateAdded", column: "date_added", kind: FieldKind::Int, sortable: true },
    FieldSpec { name: "playCount", column: "play_count", kind: FieldKind::Int, sortable: true },
    FieldSpec { name: "stems", column: "stems", kind: FieldKind::JsonArray, sortable: false },
    FieldSpec { name: "tags", column: "", kind: FieldKind::TagJoin, sortable: false },
    FieldSpec { name: "id", column: "id", kind: FieldKind::Text, sortable: true },
];

const ALBUM_FIELDS: &[FieldSpec] = &[
    FieldSpec { name: "title", column: "title", kind: FieldKind::Text, sortable: true },
    FieldSpec { name: "artist", column: "artist", kind: FieldKind::Text, sortable: true },
    FieldSpec { name: "year", column: "year", kind: FieldKind::Int, sortable: true },
    FieldSpec { name: "genre", column: "genre", kind: FieldKind::Text, sortable: false },
    FieldSpec { name: "id", column: "id", kind: FieldKind::Text, sortable: true },
];

const PLAYLIST_FIELDS: &[FieldSpec] = &[
    FieldSpec { name: "title", column: "title", kind: FieldKind::Text, sortable: true },
    FieldSpec { name: "id", column: "id", kind: FieldKind::Text, sortable: true },
];

fn find_field<'a>(fields: &'a [FieldSpec], name: &str) -> Result<&'a FieldSpec, String> {
    fields
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| format!("unknown or non-queryable field '{name}'"))
}

fn kind_matches(kind: FieldKind, value: &FilterValue) -> bool {
    matches!(
        (kind, value),
        (FieldKind::Text, FilterValue::Text(_))
            | (FieldKind::Int, FilterValue::Int(_))
            | (FieldKind::Real, FilterValue::Real(_) | FilterValue::Int(_))
            | (FieldKind::Bool, FilterValue::Bool(_))
    )
}

// ---------------------------------------------------------------------------
// Filter tree -> parameterized SQL. Every value ends up bound as a `?`
// parameter — never interpolated into the SQL string.
// ---------------------------------------------------------------------------

fn build_where(
    node: &FilterNode,
    fields: &[FieldSpec],
    params: &mut Vec<FilterValue>,
) -> Result<String, String> {
    match node {
        FilterNode::And { nodes } => {
            if nodes.is_empty() {
                return Ok("1=1".to_string());
            }
            let parts = nodes
                .iter()
                .map(|n| build_where(n, fields, params))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("({})", parts.join(" AND ")))
        }
        FilterNode::Or { nodes } => {
            if nodes.is_empty() {
                return Ok("1=1".to_string());
            }
            let parts = nodes
                .iter()
                .map(|n| build_where(n, fields, params))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!("({})", parts.join(" OR ")))
        }
        FilterNode::Cmp { field, op, value } => {
            let spec = find_field(fields, field)?;
            if spec.kind == FieldKind::TagJoin {
                return Err(format!("field '{field}' does not support comparison operators"));
            }
            if !kind_matches(spec.kind, value) {
                return Err(format!("value type mismatch for field '{field}'"));
            }
            params.push(value.clone());
            Ok(format!("{} {} ?", spec.column, op.sql()))
        }
        FilterNode::Between { field, min, max } => {
            let spec = find_field(fields, field)?;
            if !kind_matches(spec.kind, min) || !kind_matches(spec.kind, max) {
                return Err(format!("value type mismatch for field '{field}'"));
            }
            params.push(min.clone());
            params.push(max.clone());
            Ok(format!("{} BETWEEN ? AND ?", spec.column))
        }
        FilterNode::Contains { field, value } => {
            let spec = find_field(fields, field)?;
            if spec.kind == FieldKind::TagJoin {
                params.push(FilterValue::Text(value.clone()));
                return Ok(
                    "EXISTS (SELECT 1 FROM track_tags tt JOIN tags t ON t.id = tt.tag_id \
                     WHERE tt.track_id = tracks.id AND t.name = ?)"
                        .to_string(),
                );
            }
            if spec.kind != FieldKind::Text {
                return Err(format!("field '{field}' does not support 'contains'"));
            }
            params.push(FilterValue::Text(format!("%{value}%")));
            Ok(format!("{} LIKE ? ESCAPE '\\'", spec.column))
        }
        FilterNode::NotEmpty { field } => {
            let spec = find_field(fields, field)?;
            match spec.kind {
                FieldKind::JsonArray => Ok(format!("{} != '[]'", spec.column)),
                FieldKind::Text => Ok(format!("{} IS NOT NULL AND {} != ''", spec.column, spec.column)),
                FieldKind::TagJoin => Ok(
                    "EXISTS (SELECT 1 FROM track_tags tt WHERE tt.track_id = tracks.id)".to_string(),
                ),
                _ => Ok(format!("{} IS NOT NULL", spec.column)),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Keyset pagination — the default for potentially-large result sets. The
// cursor is a base64'd `sort_value\u{1}tiebreaker_id`, opaque to the caller.
// ---------------------------------------------------------------------------

fn encode_cursor(sort_value: &FilterValue, id: &str) -> String {
    let raw = match sort_value {
        FilterValue::Text(s) => format!("t:{s}\u{1}{id}"),
        FilterValue::Int(i) => format!("i:{i}\u{1}{id}"),
        FilterValue::Real(r) => format!("r:{r}\u{1}{id}"),
        FilterValue::Bool(b) => format!("b:{b}\u{1}{id}"),
    };
    URL_SAFE_NO_PAD.encode(raw)
}

fn decode_cursor(cursor: &str) -> Result<(FilterValue, String), String> {
    let raw = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| "invalid cursor".to_string())?;
    let raw = String::from_utf8(raw).map_err(|_| "invalid cursor".to_string())?;
    let (typed, id) = raw.split_once('\u{1}').ok_or_else(|| "invalid cursor".to_string())?;
    let (tag, val) = typed.split_at(2);
    let value = match tag {
        "t:" => FilterValue::Text(val.to_string()),
        "i:" => FilterValue::Int(val.parse().map_err(|_| "invalid cursor".to_string())?),
        "r:" => FilterValue::Real(val.parse().map_err(|_| "invalid cursor".to_string())?),
        "b:" => FilterValue::Bool(val == "true"),
        _ => return Err("invalid cursor".to_string()),
    };
    Ok((value, id.to_string()))
}

/// Builds the `SELECT ... FROM <table> WHERE ... ORDER BY ... LIMIT ?` query
/// for a keyset-paginated entity (tracks). `id_column` is the stable
/// tiebreaker used alongside the sort key.
fn build_keyset_query(
    table: &'static str,
    select_columns: &'static str,
    fields: &[FieldSpec],
    id_column: &'static str,
    spec: &QuerySpec,
) -> Result<(String, Vec<FilterValue>, u32), String> {
    let mut params = Vec::new();
    let filter_clause = match &spec.filter {
        Some(node) => build_where(node, fields, &mut params)?,
        None => "1=1".to_string(),
    };

    let sort = spec.sort.clone().unwrap_or(SortKey { field: "dateAdded".to_string(), descending: true });
    let sort_field = find_field(fields, &sort.field)?;
    if !sort_field.sortable {
        return Err(format!("field '{}' is not sortable", sort.field));
    }
    let dir = if sort.descending { "DESC" } else { "ASC" };
    let cmp = if sort.descending { "<" } else { ">" };

    let mut keyset_clause = String::new();
    if let Some(cursor) = &spec.page.cursor {
        let (sort_value, id) = decode_cursor(cursor)?;
        keyset_clause = format!(
            " AND ({col} {cmp} ? OR ({col} = ? AND {id_col} {cmp} ?))",
            col = sort_field.column,
            id_col = id_column,
        );
        params.push(sort_value.clone());
        params.push(sort_value);
        params.push(FilterValue::Text(id));
    }

    let limit = spec.page.limit.clamp(1, 1000);
    let sql = format!(
        "SELECT {select_columns} FROM {table} WHERE ({filter_clause}){keyset_clause} \
         ORDER BY {col} {dir}, {id_col} {dir} LIMIT ?",
        col = sort_field.column,
        id_col = id_column,
    );
    params.push(FilterValue::Int((limit + 1) as i64));

    Ok((sql, params, limit))
}

/// Reads the sort column's value straight off a row, typed per `FieldKind`.
/// Used to build the next page's cursor without guessing at a mapped
/// struct's field by name.
fn extract_sort_value(row: &rusqlite::Row, sort_field: &FieldSpec) -> rusqlite::Result<FilterValue> {
    Ok(match sort_field.kind {
        FieldKind::Text => FilterValue::Text(row.get::<_, Option<String>>(sort_field.column)?.unwrap_or_default()),
        FieldKind::Int => FilterValue::Int(row.get::<_, Option<i64>>(sort_field.column)?.unwrap_or(0)),
        FieldKind::Real => FilterValue::Real(row.get::<_, Option<f64>>(sort_field.column)?.unwrap_or(0.0)),
        FieldKind::Bool => FilterValue::Bool(row.get::<_, i64>(sort_field.column)? != 0),
        // Never reached: JsonArray/TagJoin fields have `sortable: false`, so
        // `build_keyset_query` rejects them as a sort key before we get here.
        FieldKind::JsonArray | FieldKind::TagJoin => FilterValue::Int(0),
    })
}

pub fn query_tracks(conn: &Connection, spec: &QuerySpec) -> Result<Page<Song>, String> {
    let (sql, params, limit) =
        build_keyset_query("tracks", TRACK_COLUMNS, TRACK_FIELDS, "id", spec)?;

    let sort = spec.sort.clone().unwrap_or(SortKey { field: "dateAdded".to_string(), descending: true });
    let sort_field = find_field(TRACK_FIELDS, &sort.field)?;

    let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| p as &dyn ToSql).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut rows: Vec<(Song, FilterValue)> = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok((row_to_track(row)?, extract_sort_value(row, sort_field)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let has_more = rows.len() as u32 > limit;
    if has_more {
        rows.truncate(limit as usize);
    }

    let next_cursor = if has_more {
        rows.last().map(|(last, sort_value)| encode_cursor(sort_value, &last.id))
    } else {
        None
    };

    let items = rows.into_iter().map(|(song, _)| song).collect();

    Ok(Page { items, next_cursor, has_more })
}

pub fn query_albums(conn: &Connection, spec: &QuerySpec) -> Result<Page<Album>, String> {
    let mut params = Vec::new();
    let filter_clause = match &spec.filter {
        Some(node) => build_where(node, ALBUM_FIELDS, &mut params)?,
        None => "1=1".to_string(),
    };
    let sort = spec.sort.clone().unwrap_or(SortKey { field: "title".to_string(), descending: false });
    let sort_field = find_field(ALBUM_FIELDS, &sort.field)?;
    let dir = if sort.descending { "DESC" } else { "ASC" };
    let limit = spec.page.limit.clamp(1, 1000);
    let offset = spec.page.offset.unwrap_or(0);

    let sql = format!(
        "SELECT {ALBUM_COLUMNS} FROM albums WHERE ({filter_clause}) ORDER BY {col} {dir}, id {dir} LIMIT ? OFFSET ?",
        col = sort_field.column,
    );
    params.push(FilterValue::Int((limit + 1) as i64));
    params.push(FilterValue::Int(offset as i64));

    let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| p as &dyn ToSql).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut items: Vec<Album> = stmt
        .query_map(param_refs.as_slice(), row_to_album)
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let has_more = items.len() as u32 > limit;
    if has_more {
        items.truncate(limit as usize);
    }

    Ok(Page { items, next_cursor: None, has_more })
}

pub fn query_playlists(conn: &Connection, spec: &QuerySpec) -> Result<Page<Playlist>, String> {
    let mut params = Vec::new();
    let filter_clause = match &spec.filter {
        Some(node) => build_where(node, PLAYLIST_FIELDS, &mut params)?,
        None => "1=1".to_string(),
    };
    let sort = spec.sort.clone().unwrap_or(SortKey { field: "title".to_string(), descending: false });
    let sort_field = find_field(PLAYLIST_FIELDS, &sort.field)?;
    let dir = if sort.descending { "DESC" } else { "ASC" };
    let limit = spec.page.limit.clamp(1, 1000);
    let offset = spec.page.offset.unwrap_or(0);

    let sql = format!(
        "SELECT id, title, file_path, is_diverged FROM playlists WHERE ({filter_clause}) \
         ORDER BY {col} {dir}, id {dir} LIMIT ? OFFSET ?",
        col = sort_field.column,
    );
    params.push(FilterValue::Int((limit + 1) as i64));
    params.push(FilterValue::Int(offset as i64));

    let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| p as &dyn ToSql).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut items: Vec<Playlist> = stmt
        .query_map(param_refs.as_slice(), row_to_playlist_header)
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let has_more = items.len() as u32 > limit;
    if has_more {
        items.truncate(limit as usize);
    }

    for playlist in &mut items {
        let mut stmt = conn
            .prepare("SELECT track_id FROM playlist_tracks WHERE playlist_id = ?1 ORDER BY position")
            .map_err(|e| e.to_string())?;
        playlist.tracks = stmt
            .query_map([&playlist.id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
    }

    Ok(Page { items, next_cursor: None, has_more })
}
