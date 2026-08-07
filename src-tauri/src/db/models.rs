use rusqlite::Row;
use serde::{Deserialize, Serialize};

use crate::metadata::{Album, AlbumArtwork, FileInfo, Marker, Song};

/// Column list for `tracks`, in the exact order `row_to_track` reads them.
/// The final column is a correlated subquery pulling this track's tags out of
/// the normalized `track_tags`/`tags` join (see `db/schema.rs`) as a single
/// `\u{1}`-joined string, so listing tracks stays one query instead of N+1.
pub const TRACK_COLUMNS: &str = "
    id, path, file, title, artist, album, album_id, album_artist, year, compilation,
    genre, composer, track_number, track_total, disc_number, disc_total,
    duration_secs, duration_display, overall_bitrate, audio_bitrate, sample_rate,
    bit_depth, channels, lossless, tag_type, codec, play_count, is_favourite,
    origin_country, song_project_id, markers, stems, date_added,
    (SELECT group_concat(t.name, char(1))
       FROM track_tags tt JOIN tags t ON t.id = tt.tag_id
      WHERE tt.track_id = tracks.id) AS tags_concat
";

fn json_vec(s: Option<String>) -> Vec<String> {
    s.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn json_or_default<T: Default + serde::de::DeserializeOwned>(s: Option<String>) -> T {
    s.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

pub fn row_to_track(row: &Row) -> rusqlite::Result<Song> {
    let duration_secs: Option<f64> = row.get("duration_secs")?;
    let tags_concat: Option<String> = row.get("tags_concat")?;
    let tags = tags_concat
        .map(|s| s.split('\u{1}').map(str::to_string).collect())
        .unwrap_or_default();

    Ok(Song {
        id: row.get("id")?,
        path: row.get("path")?,
        file: row.get("file")?,
        file_info: FileInfo {
            duration: duration_secs,
            duration_display: row.get("duration_display")?,
            overall_bitrate: row.get("overall_bitrate")?,
            audio_bitrate: row.get("audio_bitrate")?,
            sample_rate: row.get("sample_rate")?,
            bit_depth: row.get("bit_depth")?,
            channels: row.get("channels")?,
            lossless: row.get::<_, i64>("lossless")? != 0,
            tag_type: row.get("tag_type")?,
            codec: row.get("codec")?,
        },
        metadata: Default::default(),
        title: row.get("title")?,
        artist: row.get("artist")?,
        album: row.get("album")?,
        album_id: row.get("album_id")?,
        album_artist: row.get("album_artist")?,
        compilation: row.get("compilation")?,
        year: row.get("year")?,
        genre: json_vec(row.get("genre")?),
        composer: json_vec(row.get("composer")?),
        track_number: row.get("track_number")?,
        track_total: row.get("track_total")?,
        disc_number: row.get("disc_number")?,
        disc_total: row.get("disc_total")?,
        duration: row.get::<_, Option<String>>("duration_display")?.unwrap_or_default(),
        artwork: None,
        artwork_origin: None,
        origin_country: row.get("origin_country")?,
        origin_country_name: None,
        date_added: row.get::<_, i64>("date_added").ok().map(|v| v as u128),
        play_count: row.get("play_count")?,
        is_favourite: row.get::<_, i64>("is_favourite")? != 0,
        tags,
        markers: json_or_default::<Vec<Marker>>(row.get("markers")?),
        stems: json_or_default(row.get("stems")?),
        song_project_id: row.get("song_project_id")?,
    })
}

/// `tracks.album_id` is the source of truth for the album/track relationship
/// now (see design.md's schema-shape decision) — `tracks_ids` here is a
/// read-time convenience for callers that still want it (track-count checks,
/// bulk lookups), populated via a correlated subquery the same way tags are
/// for tracks, not a stored/denormalized column.
pub const ALBUM_COLUMNS: &str = "
    id, title, display_title, artist, year, compilation, genre, path, lossless,
    artwork_format, artwork_src,
    (SELECT group_concat(id, char(1)) FROM tracks
      WHERE tracks.album_id = albums.id ORDER BY track_number) AS track_ids_concat
";

pub fn row_to_album(row: &Row) -> rusqlite::Result<Album> {
    let artwork_format: Option<String> = row.get("artwork_format")?;
    let artwork_src: Option<String> = row.get("artwork_src")?;

    let artwork = artwork_format.map(|format| AlbumArtwork {
        src: artwork_src.unwrap_or_default(),
        format,
    });

    let track_ids_concat: Option<String> = row.get("track_ids_concat")?;
    let tracks_ids = track_ids_concat.map(|s| s.split('\u{1}').map(str::to_string).collect()).unwrap_or_default();

    Ok(Album {
        id: row.get("id")?,
        title: row.get("title")?,
        display_title: row.get("display_title")?,
        artist: row.get("artist")?,
        compilation: row.get("compilation")?,
        year: row.get("year")?,
        genre: json_vec(row.get("genre")?),
        tracks_ids,
        path: row.get::<_, Option<String>>("path")?.unwrap_or_default(),
        artwork,
        lossless: row.get::<_, i64>("lossless")? != 0,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: String,
    pub title: String,
    pub file_path: Option<String>,
    pub is_diverged: bool,
    pub tracks: Vec<String>,
}

pub fn row_to_playlist_header(row: &Row) -> rusqlite::Result<Playlist> {
    Ok(Playlist {
        id: row.get("id")?,
        title: row.get("title")?,
        file_path: row.get("file_path")?,
        is_diverged: row.get::<_, i64>("is_diverged")? != 0,
        tracks: Vec::new(),
    })
}

// ---------------------------------------------------------------------------
// Artist's Toolkit (`song_projects`/`artist_projects`/`scrapbook`) — see
// `openspec/changes/artists-toolkit-storage/design.md`.
// ---------------------------------------------------------------------------

pub const ARTIST_PROJECT_COLUMNS: &str = "id, name, members";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistProject {
    pub id: i64,
    pub name: String,
    pub members: Vec<String>,
}

pub fn row_to_artist_project(row: &Row) -> rusqlite::Result<ArtistProject> {
    let members: String = row.get("members")?;
    Ok(ArtistProject {
        id: row.get("id")?,
        name: row.get("name")?,
        members: serde_json::from_str(&members).unwrap_or_default(),
    })
}

pub const SONG_PROJECT_COLUMNS: &str = "
    id, artist_id, title, song_filepath, album, key, bpm, music_composed_by,
    lyrics_written_by, artwork_file_path, lyrics, recordings, other_content_items, song_id
";

/// `recordings`/`otherContentItems` stay opaque JSON here rather than typed Rust structs — they
/// hold arbitrary `SongProjectRecording`/`ArtistContentItem` shapes owned by the frontend
/// (`src/App.d.ts`), not normalized into their own tables (see design.md's Non-Goals: a recording
/// can point at a file that isn't an imported library track, so there's no `tracks` row to
/// reference via a join table the way `playlist_tracks` does).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongProjectRow {
    pub id: i64,
    pub artist_id: i64,
    pub title: String,
    pub song_filepath: Option<String>,
    pub album: Option<String>,
    pub key: Option<String>,
    pub bpm: Option<f64>,
    pub music_composed_by: Vec<String>,
    pub lyrics_written_by: Vec<String>,
    pub artwork_file_path: Option<String>,
    pub lyrics: Option<String>,
    pub recordings: serde_json::Value,
    pub other_content_items: serde_json::Value,
    pub song_id: Option<String>,
}

pub fn row_to_song_project(row: &Row) -> rusqlite::Result<SongProjectRow> {
    let music_composed_by: String = row.get("music_composed_by")?;
    let lyrics_written_by: String = row.get("lyrics_written_by")?;
    let recordings: String = row.get("recordings")?;
    let other_content_items: String = row.get("other_content_items")?;
    Ok(SongProjectRow {
        id: row.get("id")?,
        artist_id: row.get("artist_id")?,
        title: row.get("title")?,
        song_filepath: row.get("song_filepath")?,
        album: row.get("album")?,
        key: row.get("key")?,
        bpm: row.get("bpm")?,
        music_composed_by: serde_json::from_str(&music_composed_by).unwrap_or_default(),
        lyrics_written_by: serde_json::from_str(&lyrics_written_by).unwrap_or_default(),
        artwork_file_path: row.get("artwork_file_path")?,
        lyrics: row.get("lyrics")?,
        recordings: serde_json::from_str(&recordings).unwrap_or(serde_json::Value::Array(vec![])),
        other_content_items: serde_json::from_str(&other_content_items)
            .unwrap_or(serde_json::Value::Array(vec![])),
        song_id: row.get("song_id")?,
    })
}

pub const SCRAPBOOK_COLUMNS: &str =
    "id, type, name, tags, song_id, url, image_url, file_type_type, file_type_extension, path";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapbookFileType {
    #[serde(rename = "type")]
    pub kind: String,
    pub extension: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapbookItem {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub song_id: Option<String>,
    pub url: Option<String>,
    pub image_url: Option<String>,
    pub file_type: Option<ScrapbookFileType>,
    pub path: Option<String>,
}

pub fn row_to_scrapbook_item(row: &Row) -> rusqlite::Result<ScrapbookItem> {
    let tags: String = row.get("tags")?;
    let file_type_type: Option<String> = row.get("file_type_type")?;
    let file_type_extension: Option<String> = row.get("file_type_extension")?;
    let file_type = file_type_type
        .zip(file_type_extension)
        .map(|(kind, extension)| ScrapbookFileType { kind, extension });
    Ok(ScrapbookItem {
        id: row.get("id")?,
        kind: row.get("type")?,
        name: row.get("name")?,
        tags: serde_json::from_str(&tags).unwrap_or_default(),
        song_id: row.get("song_id")?,
        url: row.get("url")?,
        image_url: row.get("image_url")?,
        file_type,
        path: row.get("path")?,
    })
}
