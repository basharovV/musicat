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
