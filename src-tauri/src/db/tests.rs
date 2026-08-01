use rusqlite::Connection;

use super::migration::{self, DexieExport};
use super::playlists;
use super::query::{self, CmpOp, FilterNode, FilterValue, PageRequest, QuerySpec, SortKey};
use super::queue;
use super::schema;
use super::settings::{self, Settings};
use super::writer::WriterHandle;

fn open_test_db() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    schema::run_migrations(&mut conn).unwrap();
    conn
}

fn insert_track(conn: &Connection, id: &str, title: &str, year: i64, is_favourite: bool) {
    conn.execute(
        "INSERT INTO tracks (id, path, file, title, artist, album, year, is_favourite, date_added)
         VALUES (?1, ?2, ?2, ?3, 'Artist', 'Album', ?4, ?5, ?6)",
        rusqlite::params![id, format!("/music/{id}.mp3"), title, year, is_favourite as i64, year],
    )
    .unwrap();
}

#[test]
fn migrations_are_idempotent() {
    let mut conn = Connection::open_in_memory().unwrap();
    schema::run_migrations(&mut conn).unwrap();
    // Running again must not error or re-apply migration 1.
    schema::run_migrations(&mut conn).unwrap();
    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 1);
}

#[test]
fn query_tracks_filters_and_sorts() {
    let conn = open_test_db();
    insert_track(&conn, "a", "Beta", 2001, false);
    insert_track(&conn, "b", "Alpha", 2002, true);
    insert_track(&conn, "c", "Gamma", 2003, true);

    let spec = QuerySpec {
        filter: Some(FilterNode::Cmp {
            field: "isFavourite".into(),
            op: CmpOp::Eq,
            value: FilterValue::Bool(true),
        }),
        sort: Some(SortKey { field: "title".into(), descending: false }),
        page: PageRequest::default(),
    };

    let page = query::query_tracks(&conn, &spec).unwrap();
    let titles: Vec<_> = page.items.iter().map(|s| s.title.clone()).collect();
    assert_eq!(titles, vec!["Alpha", "Gamma"]);
    assert!(!page.has_more);
}

#[test]
fn query_tracks_rejects_unknown_field() {
    let conn = open_test_db();
    let spec = QuerySpec {
        filter: Some(FilterNode::Cmp {
            field: "path".into(), // not whitelisted
            op: CmpOp::Eq,
            value: FilterValue::Text("/etc/passwd".into()),
        }),
        sort: None,
        page: PageRequest::default(),
    };
    let err = query::query_tracks(&conn, &spec).unwrap_err();
    assert!(err.contains("path"), "error should name the rejected field: {err}");
}

#[test]
fn keyset_pagination_covers_every_row_without_duplicates() {
    let conn = open_test_db();
    for i in 0..5 {
        insert_track(&conn, &format!("t{i}"), &format!("Track {i}"), 2000 + i, false);
    }

    let mut seen = Vec::new();
    let mut cursor = None;
    loop {
        let spec = QuerySpec {
            filter: None,
            sort: Some(SortKey { field: "year".into(), descending: false }),
            page: PageRequest { cursor: cursor.clone(), offset: None, limit: 2 },
        };
        let page = query::query_tracks(&conn, &spec).unwrap();
        seen.extend(page.items.iter().map(|s| s.id.clone()));
        if !page.has_more {
            break;
        }
        cursor = page.next_cursor;
    }

    seen.sort();
    assert_eq!(seen, vec!["t0", "t1", "t2", "t3", "t4"]);
}

#[test]
fn tag_contains_filter_uses_the_join_not_a_substring_scan() {
    let conn = open_test_db();
    insert_track(&conn, "a", "Song A", 2000, false);
    insert_track(&conn, "b", "Song B", 2000, false);
    conn.execute("INSERT INTO tags (id, name) VALUES (1, 'chill')", []).unwrap();
    conn.execute("INSERT INTO track_tags (track_id, tag_id) VALUES ('a', 1)", []).unwrap();

    let spec = QuerySpec {
        filter: Some(FilterNode::Contains { field: "tags".into(), value: "chill".into() }),
        sort: None,
        page: PageRequest::default(),
    };
    let page = query::query_tracks(&conn, &spec).unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id, "a");
    assert_eq!(page.items[0].tags, vec!["chill".to_string()]);
}

#[test]
fn settings_round_trip_and_defaults() {
    let conn = open_test_db();
    let loaded = settings::get_settings(&conn).unwrap();
    assert_eq!(loaded.theme, "auto"); // default, no row written yet

    let mut updated = Settings::default();
    updated.theme = "dark".to_string();
    updated.folders_to_watch.push("/music".to_string());
    settings::update_settings(&conn, &updated).unwrap();

    let reloaded = settings::get_settings(&conn).unwrap();
    assert_eq!(reloaded.theme, "dark");
    assert_eq!(reloaded.folders_to_watch, vec!["/music".to_string()]);
}

#[test]
fn queue_round_trip_preserves_order() {
    let mut conn = open_test_db();
    insert_track(&conn, "a", "A", 2000, false);
    insert_track(&conn, "b", "B", 2000, false);
    queue::set_queue(&mut conn, &["b".to_string(), "a".to_string()]).unwrap();
    assert_eq!(queue::get_queue(&conn).unwrap(), vec!["b".to_string(), "a".to_string()]);
}

#[test]
fn playlist_divergence_detects_external_edits_but_not_our_own() {
    let mut conn = open_test_db();
    playlists::create_playlist(&mut conn, "p1", "My Playlist", Some("/playlists/p1.m3u")).unwrap();

    // App writes the file itself: recording the write must not count as divergence.
    playlists::record_own_write(&conn, "p1", b"#EXTM3U\n/music/a.mp3\n").unwrap();
    let diverged = playlists::check_for_divergence(&conn, "p1", b"#EXTM3U\n/music/a.mp3\n").unwrap();
    assert!(!diverged);

    // Someone hand-edits the file: the next check must catch it and flag the row.
    let diverged = playlists::check_for_divergence(&conn, "p1", b"#EXTM3U\n/music/a.mp3\n/music/b.mp3\n").unwrap();
    assert!(diverged);
    let is_diverged: i64 = conn
        .query_row("SELECT is_diverged FROM playlists WHERE id = 'p1'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(is_diverged, 1);
}

#[test]
fn writer_actor_serializes_writes_via_the_channel() {
    let conn = open_test_db();
    let writer = WriterHandle::spawn(conn);

    let result: Result<i64, String> = writer.execute(|conn| {
        conn.execute(
            "INSERT INTO tracks (id, path, file, title, artist, album, date_added) \
             VALUES ('w1', '/music/w1.mp3', 'w1.mp3', 'Written', 'Artist', 'Album', 0)",
            [],
        )?;
        conn.query_row("SELECT changes()", [], |r| r.get(0))
    });

    assert_eq!(result.unwrap(), 1);
}

/// A hand-built `dexie-export-import` envelope covering every table this
/// migration reads, with one row of realistic, hand-editable-by-a-user data
/// per table (favourites, play counts, tags) — the fields `verify()` cares
/// most about surviving byte-for-byte.
fn sample_dexie_export(favourite_song_play_count: i64) -> String {
    format!(
        r#"{{
        "formatName": "dexie",
        "formatVersion": 1,
        "data": {{
            "databaseName": "musicatdb",
            "databaseVersion": 21,
            "tables": [],
            "data": [
                {{"tableName": "songs", "rows": [
                    {{
                        "id": "song-1", "path": "/music/song-1.mp3", "file": "song-1.mp3",
                        "title": "Chill Track", "artist": "Artist A", "album": "Album A",
                        "albumId": "album-1", "year": 2020, "genre": ["chillwave"],
                        "composer": [], "trackNumber": 1, "trackTotal": 10,
                        "discNumber": 1, "discTotal": 1, "duration": "3:45",
                        "playCount": {favourite_song_play_count}, "isFavourite": true,
                        "tags": ["chill", "favourites"], "dateAdded": 1700000000000,
                        "fileInfo": {{"duration": 225.0, "lossless": false, "codec": "mp3"}}
                    }},
                    {{
                        "id": "song-2", "path": "/music/song-2.mp3", "file": "song-2.mp3",
                        "title": "Another Track", "artist": "Artist A", "album": "Album A",
                        "albumId": "album-1", "year": 2020, "genre": [],
                        "trackNumber": 2, "duration": "2:30", "playCount": 0,
                        "isFavourite": false, "tags": [], "dateAdded": 1700000001000
                    }}
                ]}},
                {{"tableName": "albums", "rows": [
                    {{
                        "id": "album-1", "title": "album a", "displayTitle": "Album A",
                        "artist": "Artist A", "year": 2020, "genre": ["chillwave"],
                        "tracksIds": ["song-1", "song-2"], "path": "/music", "lossless": false
                    }}
                ]}},
                {{"tableName": "playlists", "rows": [
                    {{"id": 1, "title": "My Mix", "path": "/playlists/my-mix.m3u",
                      "tracks": ["song-2", "song-1"]}}
                ]}},
                {{"tableName": "internalPlaylists", "rows": [
                    {{"id": "todelete", "title": "To delete", "tracks": []}}
                ]}},
                {{"tableName": "smartQueries", "rows": [
                    {{"id": 1, "name": "Chill favourites", "parts": [{{"fieldKey": "isFavourite"}}]}}
                ]}},
                {{"tableName": "songProjects", "rows": [
                    {{"title": "WIP Song", "artist": "Artist A", "musicComposedBy": ["Artist A"],
                      "lyricsWrittenBy": [], "recordings": [], "otherContentItems": []}}
                ]}},
                {{"tableName": "artistProjects", "rows": [
                    {{"name": "Artist A", "members": ["Solo"]}}
                ]}},
                {{"tableName": "scrapbook", "rows": [
                    {{"id": "scrap-1", "type": "link", "name": "Interview", "tags": [],
                      "url": "https://example.com/interview"}}
                ]}}
            ]
        }}
    }}"#
    )
}

#[test]
fn migrate_populates_every_table_and_resolves_tags() {
    let mut conn = open_test_db();
    let export: DexieExport = serde_json::from_str(&sample_dexie_export(42)).unwrap();

    let report = migration::migrate(&mut conn, &export).unwrap();
    assert_eq!(report.tracks, 2);
    assert_eq!(report.albums, 1);
    assert_eq!(report.playlists, 2); // "playlists" + "internalPlaylists"
    assert_eq!(report.smart_queries, 1);
    assert_eq!(report.song_projects, 1);
    assert_eq!(report.artist_projects, 1);
    assert_eq!(report.scrapbook_items, 1);
    assert!(report.warnings.is_empty());

    let (play_count, is_favourite): (i64, i64) = conn
        .query_row("SELECT play_count, is_favourite FROM tracks WHERE id = 'song-1'", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!(play_count, 42);
    assert_eq!(is_favourite, 1);

    let tag_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM track_tags tt JOIN tags t ON t.id = tt.tag_id WHERE tt.track_id = 'song-1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(tag_count, 2);

    // Playlist order must survive (source listed song-2 before song-1).
    let ordered: Vec<String> = conn
        .prepare("SELECT track_id FROM playlist_tracks WHERE playlist_id = '1' ORDER BY position")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(ordered, vec!["song-2".to_string(), "song-1".to_string()]);
}

#[test]
fn verify_passes_on_a_clean_migration_and_fails_on_drift() {
    let mut conn = open_test_db();
    let export: DexieExport = serde_json::from_str(&sample_dexie_export(42)).unwrap();
    migration::migrate(&mut conn, &export).unwrap();

    let report = migration::verify(&export, &conn).unwrap();
    assert!(report.ok, "expected a clean verification, got: {:?}", report.mismatches);

    // A source that disagrees with what's actually in the shadow db (as if
    // something got dropped or altered mid-migration) must be caught.
    let drifted_export: DexieExport = serde_json::from_str(&sample_dexie_export(999)).unwrap();
    let report = migration::verify(&drifted_export, &conn).unwrap();
    assert!(!report.ok);
    assert!(report.mismatches.iter().any(|m| m.contains("playCount")));
}

#[test]
fn cutover_copies_shadow_data_into_the_live_database() {
    // Real files, matching production: both the shadow and live databases
    // are on-disk SQLite files, and `cutover` attaches one to the other.
    let dir = tempfile::tempdir().unwrap();
    let shadow_path = dir.path().join("shadow.sqlite3");
    {
        let mut shadow_conn = Connection::open(&shadow_path).unwrap();
        schema::run_migrations(&mut shadow_conn).unwrap();
        let export: DexieExport = serde_json::from_str(&sample_dexie_export(42)).unwrap();
        migration::migrate(&mut shadow_conn, &export).unwrap();
    } // shadow_conn dropped/closed here, before attaching it elsewhere

    let live_path = dir.path().join("live.sqlite3");
    let mut live_conn = Connection::open(&live_path).unwrap();
    live_conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    schema::run_migrations(&mut live_conn).unwrap();
    migration::cutover(&mut live_conn, &shadow_path).unwrap();

    let track_count: i64 = live_conn.query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get(0)).unwrap();
    assert_eq!(track_count, 2);
    let album_count: i64 = live_conn.query_row("SELECT COUNT(*) FROM albums", [], |r| r.get(0)).unwrap();
    assert_eq!(album_count, 1);
    let playlist_track_count: i64 =
        live_conn.query_row("SELECT COUNT(*) FROM playlist_tracks", [], |r| r.get(0)).unwrap();
    assert_eq!(playlist_track_count, 2);
}
