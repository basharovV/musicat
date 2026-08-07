use log::info;
use rusqlite::Connection;

/// Ordered, versioned migrations. Each entry is applied at most once, in order,
/// inside its own transaction, and recorded in `schema_migrations` — see
/// `openspec/changes/sqlite-storage-foundation/specs/library-storage/spec.md`
/// ("Versioned schema migrations").
pub(crate) const MIGRATIONS: &[(i64, &str)] = &[(1, MIGRATION_0001), (2, MIGRATION_0002)];

const MIGRATION_0001: &str = r#"
CREATE TABLE tracks (
    id              TEXT PRIMARY KEY,
    path            TEXT NOT NULL UNIQUE,
    file            TEXT NOT NULL,
    title           TEXT NOT NULL,
    artist          TEXT NOT NULL,
    album           TEXT NOT NULL,
    album_id        TEXT REFERENCES albums(id) ON DELETE SET NULL,
    album_artist    TEXT,
    year            INTEGER NOT NULL DEFAULT 0,
    compilation     INTEGER NOT NULL DEFAULT 0,
    genre           TEXT NOT NULL DEFAULT '[]',   -- JSON array
    composer        TEXT NOT NULL DEFAULT '[]',   -- JSON array
    track_number    INTEGER NOT NULL DEFAULT 0,
    track_total     INTEGER NOT NULL DEFAULT 0,
    disc_number     INTEGER NOT NULL DEFAULT 0,
    disc_total      INTEGER NOT NULL DEFAULT 0,
    duration_secs   REAL,
    duration_display TEXT,
    overall_bitrate INTEGER,
    audio_bitrate   INTEGER,
    sample_rate     INTEGER,
    bit_depth       INTEGER,
    channels        INTEGER,
    lossless        INTEGER NOT NULL DEFAULT 0,
    tag_type        TEXT,
    codec           TEXT,
    play_count      INTEGER NOT NULL DEFAULT 0,
    is_favourite    INTEGER NOT NULL DEFAULT 0,
    origin_country  TEXT,
    song_project_id INTEGER,
    markers         TEXT NOT NULL DEFAULT '[]',   -- JSON array
    stems           TEXT NOT NULL DEFAULT '[]',   -- JSON array
    date_added      INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_tracks_artist_year_album_tracknum ON tracks(artist, year, album, track_number);
CREATE INDEX idx_tracks_albumartist_album_tracknum ON tracks(album_artist, album, track_number);
CREATE INDEX idx_tracks_album_tracknum ON tracks(album, track_number);
CREATE INDEX idx_tracks_albumartist_album ON tracks(album_artist, album);
CREATE INDEX idx_tracks_album_id ON tracks(album_id);
CREATE INDEX idx_tracks_title ON tracks(title);
CREATE INDEX idx_tracks_artist ON tracks(artist);
CREATE INDEX idx_tracks_year ON tracks(year);
CREATE INDEX idx_tracks_duration ON tracks(duration_secs);
CREATE INDEX idx_tracks_is_favourite ON tracks(is_favourite);
CREATE INDEX idx_tracks_origin_country ON tracks(origin_country);
CREATE INDEX idx_tracks_date_added ON tracks(date_added);

CREATE TABLE albums (
    id              TEXT PRIMARY KEY,
    title           TEXT NOT NULL,          -- lower-cased, for case-insensitive sort/search
    display_title   TEXT NOT NULL,
    artist          TEXT NOT NULL,
    year            INTEGER NOT NULL DEFAULT 0,
    compilation     INTEGER NOT NULL DEFAULT 0,
    genre           TEXT NOT NULL DEFAULT '[]', -- JSON array
    path            TEXT,
    lossless        INTEGER NOT NULL DEFAULT 0,
    -- Artwork bytes live on disk (see artwork::cache_artwork); this is a
    -- src/format pointer to that cache, not the image data itself.
    artwork_format  TEXT,
    artwork_src     TEXT
) STRICT;

CREATE INDEX idx_albums_title ON albums(title);
CREATE INDEX idx_albums_artist ON albums(artist);
CREATE INDEX idx_albums_year ON albums(year);

CREATE TABLE tags (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    name    TEXT NOT NULL UNIQUE
) STRICT;

CREATE TABLE track_tags (
    track_id    TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    tag_id      INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (track_id, tag_id)
) STRICT;

CREATE INDEX idx_track_tags_tag ON track_tags(tag_id);

CREATE TABLE playlists (
    id              TEXT PRIMARY KEY,
    title           TEXT NOT NULL,
    file_path       TEXT,
    content_hash    TEXT,
    last_synced_at  INTEGER,
    is_diverged     INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE TABLE playlist_tracks (
    playlist_id TEXT NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    track_id    TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, position)
) STRICT;

CREATE INDEX idx_playlist_tracks_playlist ON playlist_tracks(playlist_id, position);
CREATE INDEX idx_playlist_tracks_track ON playlist_tracks(track_id);

CREATE TABLE smart_queries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    query_json  TEXT NOT NULL
) STRICT;

CREATE TABLE settings (
    id      INTEGER PRIMARY KEY CHECK (id = 1),
    data    TEXT NOT NULL
) STRICT;

CREATE TABLE queue (
    position    INTEGER PRIMARY KEY,
    track_id    TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE song_projects (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    song_filepath           TEXT,
    title                   TEXT NOT NULL,
    artist                  TEXT NOT NULL,
    album                   TEXT,
    key                     TEXT,
    bpm                     REAL,
    music_composed_by       TEXT NOT NULL DEFAULT '[]', -- JSON array
    lyrics_written_by       TEXT NOT NULL DEFAULT '[]', -- JSON array
    artwork_file_path       TEXT,
    lyrics                  TEXT,
    recordings              TEXT NOT NULL DEFAULT '[]', -- JSON array
    other_content_items     TEXT NOT NULL DEFAULT '[]', -- JSON array
    song_id                 TEXT REFERENCES tracks(id) ON DELETE SET NULL
) STRICT;

CREATE TABLE artist_projects (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    members         TEXT NOT NULL DEFAULT '[]', -- JSON array
    profile_photo   TEXT
) STRICT;

CREATE TABLE scrapbook (
    id                  TEXT PRIMARY KEY,
    type                TEXT NOT NULL,
    name                TEXT NOT NULL,
    tags                TEXT NOT NULL DEFAULT '[]', -- JSON array
    song_id             TEXT,
    url                 TEXT,
    image_url           TEXT,
    file_type_type      TEXT,
    file_type_extension TEXT,
    path                TEXT
) STRICT;

-- Standalone (non-content-linked) FTS5 index: tracks.id is a text hash, not an
-- integer rowid, so external-content mode can't map rows directly. Kept in
-- sync via triggers below instead.
CREATE VIRTUAL TABLE tracks_fts USING fts5(id UNINDEXED, title, artist, album, composer);

CREATE TRIGGER trg_tracks_fts_insert AFTER INSERT ON tracks BEGIN
    INSERT INTO tracks_fts(id, title, artist, album, composer)
    VALUES (new.id, new.title, new.artist, new.album, new.composer);
END;

CREATE TRIGGER trg_tracks_fts_update AFTER UPDATE ON tracks BEGIN
    DELETE FROM tracks_fts WHERE id = old.id;
    INSERT INTO tracks_fts(id, title, artist, album, composer)
    VALUES (new.id, new.title, new.artist, new.album, new.composer);
END;

CREATE TRIGGER trg_tracks_fts_delete AFTER DELETE ON tracks BEGIN
    DELETE FROM tracks_fts WHERE id = old.id;
END;
"#;

/// Reshapes `song_projects`/`artist_projects` for the identity/reconciliation model in
/// `openspec/changes/artists-toolkit-storage/design.md` ("Schema" section — this is that SQL
/// verbatim). Migration 0001 carried these two tables over from Dexie as a straight lift, with no
/// working identity reconciliation (`artist_projects` had no uniqueness on `name`; `song_projects`
/// stored a free-text `artist` copy instead of a foreign key), which is exactly what let the
/// pre-this-change dead-write bug accumulate duplicate rows for the same logical song. The dedup
/// steps below are for installs where `sqlite-storage-foundation`'s Dexie migration already copied
/// that duplication into SQLite before this change existed.
const MIGRATION_0002: &str = r#"
DELETE FROM artist_projects WHERE id NOT IN (SELECT MAX(id) FROM artist_projects GROUP BY name);

CREATE TABLE artist_projects_new (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    name    TEXT NOT NULL UNIQUE,
    members TEXT NOT NULL DEFAULT '[]' -- JSON array
) STRICT;

INSERT INTO artist_projects_new (id, name, members)
SELECT id, name, members FROM artist_projects;

DROP TABLE artist_projects;
ALTER TABLE artist_projects_new RENAME TO artist_projects;

DELETE FROM song_projects WHERE id NOT IN (SELECT MAX(id) FROM song_projects GROUP BY artist, title);

-- Every song_project's `artist` text must resolve to a tracked artist row before the FK rebuild
-- below — covers the common case where `artist_projects` never had a row at all (its one write path
-- was a no-op before this change; see proposal.md).
INSERT OR IGNORE INTO artist_projects (name, members)
SELECT DISTINCT artist, '[]' FROM song_projects WHERE artist NOT IN (SELECT name FROM artist_projects);

CREATE TABLE song_projects_new (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    artist_id               INTEGER NOT NULL REFERENCES artist_projects(id) ON DELETE CASCADE,
    title                   TEXT NOT NULL,
    song_filepath           TEXT,
    album                   TEXT,
    key                     TEXT,
    bpm                     REAL,
    music_composed_by       TEXT NOT NULL DEFAULT '[]', -- JSON array
    lyrics_written_by       TEXT NOT NULL DEFAULT '[]', -- JSON array
    artwork_file_path       TEXT,
    lyrics                  TEXT,
    recordings              TEXT NOT NULL DEFAULT '[]', -- JSON array of SongProjectRecording
    other_content_items     TEXT NOT NULL DEFAULT '[]', -- JSON array of ArtistContentItem
    song_id                 TEXT REFERENCES tracks(id) ON DELETE SET NULL,
    UNIQUE (artist_id, title)
) STRICT;

INSERT INTO song_projects_new (id, artist_id, title, song_filepath, album, key, bpm,
    music_composed_by, lyrics_written_by, artwork_file_path, lyrics, recordings,
    other_content_items, song_id)
SELECT sp.id, ap.id, sp.title, sp.song_filepath, sp.album, sp.key, sp.bpm,
    sp.music_composed_by, sp.lyrics_written_by, sp.artwork_file_path, sp.lyrics,
    sp.recordings, sp.other_content_items, sp.song_id
FROM song_projects sp JOIN artist_projects ap ON ap.name = sp.artist;

DROP TABLE song_projects;
ALTER TABLE song_projects_new RENAME TO song_projects;

CREATE INDEX idx_song_projects_artist ON song_projects(artist_id);
"#;

/// Applies every migration with a version greater than the highest recorded
/// one, each in its own transaction, and records it in `schema_migrations`.
/// Safe to call on every app start (idempotent when nothing is pending).
pub fn run_migrations(conn: &mut Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version     INTEGER PRIMARY KEY,
            applied_at  INTEGER NOT NULL
        ) STRICT;",
    )?;

    let current_version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    for (version, sql) in MIGRATIONS {
        if *version <= current_version {
            continue;
        }
        info!("[db] Applying migration {version}");
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![
                version,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64
            ],
        )?;
        tx.commit()?;
    }

    Ok(())
}
