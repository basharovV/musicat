use artwork_cacher::look_for_art;
use chksum_md5::MD5;
use lofty::config::{ParseOptions, WriteOptions};
use lofty::file::{AudioFile, FileType, TaggedFileExt};
use lofty::picture::{MimeType, Picture};
use lofty::probe::Probe;
use lofty::read_from_path;
use lofty::tag::{Accessor, ItemKey, ItemValue, TagItem, TagType};
use log::info;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use serde_m3u::Playlist;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{BufReader, ErrorKind};
use std::ops::Mul;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::{thread, time};
use tauri::{AppHandle, Emitter};

use crate::store::{load_settings, UserSettings};

pub mod artwork_cacher;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MetadataEntry {
    id: String, // format-specific tag key eg. for ID3v2, "TIT2"
    #[serde(default, deserialize_with = "deserialize_some")]
    pub value: Option<Option<String>>, // value eg. "Canon in D"
}

impl Default for MetadataEntry {
    fn default() -> Self {
        Self {
            id: String::new(),
            value: None,
        }
    }
}

// Any value that is present is considered Some value, including null.
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WriteMetatadaEvent {
    metadata: Vec<MetadataEntry>,
    tag_type: Option<String>,
    file_path: String,
    artwork_file: String,
    artwork_data: Vec<u8>,
    artwork_data_mime_type: Option<String>,
    delete_artwork: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WriteMetatadasEvent {
    tracks: Vec<WriteMetatadaEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanPathsEvent {
    paths: Vec<String>,
    recursive: bool,
    process_albums: bool,
    process_m3u: bool,
    is_async: bool,
    is_cover_fullcheck: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanPlaylistEvent {
    playlist: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub duration: Option<f64>, //s
    pub duration_display: Option<String>,
    pub overall_bitrate: Option<u32>,
    pub audio_bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub channels: Option<u8>,
    pub lossless: bool,
    pub tag_type: Option<String>,
    pub codec: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Artwork {
    pub data: Vec<u8>,
    pub src: Option<String>,
    format: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ArtworkOrigin {
    Broken,
    File,
    Metadata,
    NotFound,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AlbumArtwork {
    pub src: String,
    pub format: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: String,
    pub path: String,
    pub file: String,
    pub file_info: FileInfo,

    /// The metadata from the file, only needed for the tagger
    /// Not stored in database, retrieved on request
    pub metadata: HashMap<String, MetadataEntry>,

    // Derived metadata, stored for performance
    // because it's used in the library UI
    /// file name or title from metadata
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_id: Option<String>, // Might be a singleton
    pub album_artist: Option<String>,
    pub compilation: i32,
    pub year: i32,
    pub genre: Vec<String>,
    pub composer: Vec<String>,
    pub track_number: i32,
    pub track_total: i32,
    pub disc_number: i32,
    pub disc_total: i32,
    pub duration: String,

    /// Artwork on request
    pub artwork: Option<Artwork>,

    pub artwork_origin: Option<ArtworkOrigin>,
    pub origin_country: Option<String>,
    pub origin_country_name: Option<String>,
    pub date_added: Option<u128>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: String,            // Hash of artist + album name
    pub title: String, // We store the title in lower case for indexed case insensitive searches
    pub display_title: String, // The display title with actual case
    pub artist: String,
    pub compilation: i32,
    pub year: i32,
    pub genre: Vec<String>,
    pub tracks_ids: Vec<String>,
    pub path: String,
    pub artwork: Option<AlbumArtwork>,
    pub lossless: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToImportEvent {
    songs: Vec<Song>,
    albums: Vec<Album>,
    progress: u8,
    done: bool,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSongMetadataEvent {
    path: String,
    is_import: bool,
    include_folder_artwork: bool,
    include_raw_tags: bool,
}

#[tauri::command]
pub async fn write_metadatas(
    event: WriteMetatadasEvent,
    app_handle: tauri::AppHandle,
) -> ToImportEvent {
    info!("{:?}", event);
    // let payload: &str = event.payload().unwrap();
    let songs: Arc<std::sync::Mutex<Vec<Song>>> = Arc::new(Mutex::new(Vec::new()));
    let albums: Arc<std::sync::Mutex<HashMap<String, Album>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let settings = load_settings(&app_handle).ok();

    let mut error: Option<String> = None;

    // let v: WriteMetatadaEvent = serde_json::from_str(payload).unwrap();
    for track in event.tracks.iter() {
        let write_result = write_metadata_track(&track.clone());
        match write_result {
            Ok(()) => {
                if let Some(mut song) = crate::metadata::extract_metadata(
                    Path::new(&track.file_path),
                    true,
                    false,
                    false,
                    true,
                    &app_handle,
                ) {
                    if let Some(album) = process_new_album(
                        &mut song,
                        Some(&albums),
                        None,
                        None,
                        &settings,
                        &app_handle,
                    ) {
                        info!("Album: {:?}", album);
                        let existing_album = albums.lock().unwrap().get_mut(&album.id).cloned();
                        if let Some(existing_album) = existing_album {
                            // Merge with existing album
                            let mut merged_album = existing_album.clone();
                            merged_album.tracks_ids.push(song.id.clone());
                            albums
                                .lock()
                                .unwrap()
                                .insert(album.id.clone(), merged_album);
                        } else {
                            albums.lock().unwrap().insert(album.id.clone(), album);
                        }
                    }
                    songs.lock().unwrap().push(song);
                }
                info!("Wrote metadata")
            }
            Err(err) => {
                match err.downcast_ref::<std::io::Error>() {
                    Some(io_err) => match io_err.kind() {
                        ErrorKind::PermissionDenied => {
                            error.replace(String::from(
                                "Permission denied. Check your file permissions and try again",
                            ));
                            break;
                        }
                        _ => {
                            error.replace(String::from(io_err.to_string()));
                            // panic!("{}", io_err);
                        }
                    },
                    None => {}
                }
                error.replace(String::from(err.to_string()));
                info!("Error writing metadata: {}", err);
            }
        }
    }
    // Emit result back to client

    let to_import = ToImportEvent {
        songs: songs.lock().unwrap().clone(),
        albums: albums.lock().unwrap().values().cloned().collect(),
        progress: 100,
        done: true,
        error: error,
    };
    return to_import;
}

#[tauri::command]
pub async fn get_song_metadata(event: GetSongMetadataEvent, app: AppHandle) -> Option<Song> {
    let path = Path::new(event.path.as_str());

    if path.is_file() {
        // info!("{:?}", entry.path());
        if let Some(song) = crate::metadata::extract_metadata(
            &path,
            event.is_import,
            false,
            event.include_folder_artwork,
            event.include_raw_tags,
            &app,
        ) {
            return Some(song);
        }
    }
    None
}

#[tauri::command]
pub async fn scan_paths(
    event: ScanPathsEvent,
    app_handle: tauri::AppHandle,
) -> Option<ToImportEvent> {
    // info!("scan_paths {:?}", event);
    let start = Instant::now();
    let songs: Arc<std::sync::Mutex<HashMap<String, Song>>> = Arc::new(Mutex::new(HashMap::new()));
    let albums: Arc<std::sync::Mutex<HashMap<String, Album>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let artwork_origins: Arc<std::sync::Mutex<HashMap<String, ArtworkOrigin>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let settings = load_settings(&app_handle).ok();

    event.paths.par_iter().for_each(|p| {
        let path = Path::new(p.as_str());
        // info!("{:?}", path);

        if path.is_file() {
            if event.process_m3u && path.extension() == Some(OsStr::new("m3u")) {
                if let Some(sub_results) = process_playlist(Path::new(path), &app_handle) {
                    if !sub_results.songs.is_empty() {
                        songs.lock().unwrap().extend(sub_results.songs);
                    }
                }
            } else {
                // info!("path is file");
                if let Some(mut song) = crate::metadata::extract_metadata(
                    &path,
                    true,
                    event.is_cover_fullcheck,
                    false,
                    false,
                    &app_handle,
                ) {
                    if event.process_albums {
                        if let Some(album) = process_new_album(
                            &mut song,
                            Some(&albums),
                            None,
                            Some(&artwork_origins),
                            &settings,
                            &app_handle,
                        ) {
                            info!("Album: {:?}", album);
                            albums
                                .lock()
                                .unwrap()
                                .entry(album.id.clone())
                                .and_modify(|a| {
                                    a.tracks_ids.push(song.id.clone());
                                })
                                .or_insert(album);
                        }
                    }
                    song.artwork = None;
                    songs.lock().unwrap().insert(song.id.clone(), song);
                }
            }
        } else if path.is_dir() {
            if let Some(sub_results) = process_directory(
                Path::new(path),
                &songs,
                &albums,
                &artwork_origins,
                event.recursive,
                event.process_albums,
                event.is_cover_fullcheck,
                &app_handle,
            ) {
                if !sub_results.songs.is_empty() {
                    songs.lock().unwrap().extend(sub_results.songs);
                }
                if !sub_results.albums.is_empty() {
                    albums.lock().unwrap().extend(sub_results.albums);
                }
            }
        }
    });

    // Print the collected songs for demonstration purposes
    // for song in songs.lock().unwrap().clone() {
    //     info!("{:?}", song);
    // }

    for album in albums.lock().unwrap().values_mut() {
        album
            .tracks_ids
            .sort_by_key(|id| songs.lock().unwrap().get(id).unwrap().track_number);
    }

    // First send all songs in chunks
    let length = songs.lock().unwrap().clone().len();
    if event.is_async && length > 500 {
        let songs_locked = songs.lock().unwrap();
        let enumerated_songs: Vec<_> = songs_locked.values().enumerate().collect();
        let enumerator = enumerated_songs.chunks(200);
        let chunks = enumerator.len();

        // Send songs to client in chunks
        enumerator.into_iter().enumerate().for_each(|(idx, slice)| {
            thread::sleep(time::Duration::from_millis(1000));
            let progress = if idx == chunks - 1 {
                100
            } else {
                u8::min(
                    ((slice.len() * (idx + 1)) as f64 / length as f64).mul(100.0) as u8,
                    100,
                )
            };
            info!("{:?}", progress);
            let _ = app_handle.emit(
                "import_chunk",
                ToImportEvent {
                    songs: slice.iter().map(|(_, song)| (*song).clone()).collect(),
                    albums: vec![],
                    progress: progress,
                    done: progress == 100 && albums.lock().unwrap().clone().len() == 0,
                    error: None,
                },
            );
        });
    } else if event.is_async {
        // Send album artworks first - client needs to process the songs, which in turn needs to process the albums
        // Once this is done the client can update all existing albums with the artworks
        // The done flag is true - this determines when the client is done importing
        let _ = app_handle.emit(
            "import_albums",
            ToImportEvent {
                songs: vec![],
                albums: albums.lock().unwrap().values().cloned().collect(),
                progress: 100,
                done: true,
                error: None,
            },
        );
        let _ = app_handle.emit(
            "import_chunk",
            ToImportEvent {
                songs: songs.lock().unwrap().values().cloned().collect(),
                albums: vec![],
                progress: 100,
                done: albums.lock().unwrap().clone().len() == 0,
                error: None,
            },
        );
    }

    // Send all albums

    // If more than 100 albums, also send them in chunks

    if event.is_async && albums.lock().unwrap().clone().len() > 100 {
        let albums_clone: Vec<Album> = albums.lock().unwrap().values().cloned().collect();
        let enumerator = albums_clone.chunks(100);
        let chunks = enumerator.len();
        enumerator.into_iter().enumerate().for_each(|(idx, slice)| {
            thread::sleep(time::Duration::from_millis(1000));
            let progress = if idx == chunks - 1 {
                100
            } else {
                u8::min(
                    ((slice.len() * (idx + 1)) as f64 / length as f64).mul(100.0) as u8,
                    100,
                )
            };
            info!("{:?}", progress);
            let _ = app_handle.emit(
                "import_albums",
                ToImportEvent {
                    songs: vec![],
                    albums: slice.to_vec(),
                    progress: progress,
                    done: progress == 100,
                    error: None,
                },
            );
        });
    } else if event.is_async {
        let _ = app_handle.emit(
            "import_albums",
            ToImportEvent {
                songs: vec![],
                albums: albums.lock().unwrap().values().cloned().collect(),
                progress: 100,
                done: true,
                error: None,
            },
        );
    }

    // Print how many songs and albums were imported, and how long the import took
    info!(
        "<bold><green>Imported {} songs and {} albums in {:.2} seconds</green></bold>",
        songs.lock().unwrap().len(),
        albums.lock().unwrap().len(),
        (Instant::now() - start).as_secs_f32()
    );

    Some(ToImportEvent {
        songs: if event.is_async {
            vec![]
        } else {
            songs.lock().unwrap().values().cloned().collect()
        },
        albums: if event.is_async {
            vec![]
        } else {
            albums.lock().unwrap().values().cloned().collect()
        },
        progress: 100,
        done: true,
        error: None,
    })
}

#[tauri::command]
pub async fn scan_playlist(
    event: ScanPlaylistEvent,
    app_handle: tauri::AppHandle,
) -> Option<ToImportEvent> {
    let mut songs: Vec<Song> = vec![];

    let read_result = std::fs::read_to_string(event.playlist);

    if let Ok(content) = read_result {
        let playlist: Playlist = Playlist::from(content.as_str());

        for entry in playlist.list {
            if let Some(mut song) = crate::metadata::extract_metadata(
                Path::new(&entry.url),
                true,
                false,
                false,
                false,
                &app_handle,
            ) {
                song.artwork = None;

                songs.push(song);
            }
        }
    }

    Some(ToImportEvent {
        songs,
        albums: vec![],
        progress: 100,
        done: true,
        error: None,
    })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetArtworkEvent {
    path: String,
}

#[tauri::command]
pub async fn get_artwork_file(event: GetArtworkEvent, app: AppHandle) -> Option<Artwork> {
    let file_path = Path::new(event.path.as_str());

    if file_path.is_file() {
        let path = file_path.to_string_lossy().into_owned();
        let file = file_path.file_name()?.to_string_lossy().into_owned();
        let settings = load_settings(&app).ok()?;

        match look_for_art(&path, &file, &settings, &app) {
            Ok(res) => {
                if let Some(art) = res.clone() {
                    return Some(Artwork {
                        data: vec![],
                        src: Some(art.artwork_src),
                        format: art.artwork_format,
                    });
                }
            }
            Err(e) => {
                info!("Error looking for artwork: {}", e);
            }
        }
    }

    None
}

#[tauri::command]
pub async fn get_artwork_metadata(event: GetArtworkEvent, _app: AppHandle) -> Option<Artwork> {
    let file_path = Path::new(event.path.as_str());

    if file_path.is_file() {
        match read_from_path(&file_path) {
            Ok(tagged_file) => {
                if tagged_file.primary_tag().is_some() {
                    if let Some(pic) = tagged_file.primary_tag().unwrap().pictures().first() {
                        let mut format = String::new();

                        if let Some(mime) = pic.mime_type() {
                            format = mime.to_string();
                        } else {
                            let mime = mimetype::detect(pic.data());

                            format = mime.mime.to_string();
                        }

                        return Some(Artwork {
                            data: pic.data().to_vec(),
                            src: None,
                            format,
                        });
                    }
                }
            }
            Err(e) => {
                info!("Error reading file: {}", e);
                return None;
            }
        }
    }

    None
}

fn process_new_album(
    song: &mut Song,
    new_albums: Option<&Arc<std::sync::Mutex<HashMap<String, Album>>>>,
    newest_albums: Option<&Arc<std::sync::Mutex<HashMap<String, Album>>>>,
    artwork_origins: Option<&Arc<std::sync::Mutex<HashMap<String, ArtworkOrigin>>>>,
    settings: &Option<UserSettings>,
    app: &tauri::AppHandle,
) -> Option<Album> {
    let mut artwork_src = String::new();
    let mut artwork_format = String::new();
    // Strip song from path
    let song_path = song.path.clone();
    let album_path = Path::new(&song_path).parent().unwrap().to_str().unwrap();
    let album_id = MD5::hash(
        format!("{} - {}", album_path, song.album.as_str())
            .to_lowercase()
            .as_bytes(),
    )
    .to_hex_lowercase();
    // info!("album: {} , {}", song.album, album_id);

    if let Some(album) = get_new_album(new_albums, album_id.clone(), song, artwork_origins) {
        return Some(album);
    }

    if let Some(album) = get_new_album(newest_albums, album_id.clone(), song, artwork_origins) {
        return Some(album);
    }

    if let Some(settings) = settings {
        let result = look_for_art(&song.path, &song.file, settings, app);
        if let Ok(res) = result {
            if let Some(art) = res.clone() {
                // info!("Found existing artwork in folder for: {}", song.album);
                artwork_src = art.artwork_src;
                artwork_format = art.artwork_format;

                if song.artwork_origin.is_none() {
                    song.artwork_origin = Some(ArtworkOrigin::File);
                }

                if artwork_origins.is_some() {
                    artwork_origins
                        .unwrap()
                        .lock()
                        .unwrap()
                        .insert(album_id.clone(), ArtworkOrigin::File);
                }
            }
        } else if let Err(e) = result {
            info!("Error looking for artwork: {}", e);
        }
    }
    info!("artwork found: {}", artwork_src);
    if artwork_src.is_empty() {
        // info!("Song artwork: {:?}", &song.artwork);
        if let Some(art) = &song.artwork {
            // info!("Caching artwork for: {}", song.album);
            // Cache artwork using artwork_cacher
            let cached_art_path =
                artwork_cacher::cache_artwork(&art.data, &album_id, &art.format, app);
            if let Ok(p) = cached_art_path {
                artwork_src = p.to_str().unwrap().to_string();
                artwork_format = art.format.clone();
            } else {
                info!("Error caching artwork: {}", cached_art_path.unwrap_err());
            }
        } else {
            if song.artwork_origin.is_none() {
                song.artwork_origin = Some(ArtworkOrigin::NotFound);
            }

            if artwork_origins.is_some() {
                artwork_origins
                    .unwrap()
                    .lock()
                    .unwrap()
                    .insert(album_id.clone(), ArtworkOrigin::NotFound);
            }
        }
    }

    // Convert src to Tauri's asset protocol and encode it as a URI
    if !artwork_src.is_empty() {
        artwork_src = convert_file_src(artwork_src);
    }

    return Some(Album {
        id: album_id,
        title: song.album.clone().to_lowercase(),
        display_title: song.album.clone(),
        artist: if song.album_artist.is_some() {
            song.album_artist.clone().unwrap()
        } else if song.compilation == 1 {
            String::from("Compilation")
        } else {
            song.artist.clone()
        },
        compilation: song.compilation,
        tracks_ids: vec![song.id.clone()],
        lossless: song.file_info.lossless,
        path: album_path.to_string(),
        year: song.year,
        genre: song.genre.clone(),
        artwork: if artwork_src.is_empty() {
            None
        } else {
            Some(AlbumArtwork {
                src: artwork_src.to_string(),
                format: artwork_format.to_string(),
            })
        },
    });
}

fn get_new_album(
    albums: Option<&Arc<std::sync::Mutex<HashMap<String, Album>>>>,
    album_id: String,
    song: &mut Song,
    artwork_origins: Option<&Arc<std::sync::Mutex<HashMap<String, ArtworkOrigin>>>>,
) -> Option<Album> {
    if let Some(albums) = albums {
        let album = albums.lock().unwrap().get_mut(&album_id).cloned();

        if album.is_some() {
            if song.artwork_origin.is_none() {
                if artwork_origins.is_some() {
                    let artwork_origin = artwork_origins
                        .unwrap()
                        .lock()
                        .unwrap()
                        .get_mut(&album_id)
                        .cloned();

                    if artwork_origin.is_some() {
                        song.artwork_origin = artwork_origin;
                    }
                } else {
                    song.artwork_origin = Some(ArtworkOrigin::NotFound);
                }
            }

            return album;
        }
    }

    None
}

pub fn convert_file_src(artwork_src: String) -> String {
    // If windows or android, use http://{protocol}.localhost/{path}
    if cfg!(target_os = "windows") || cfg!(target_os = "android") {
        format!("http://asset.localhost/{}", artwork_src)
    } else {
        format!("asset://localhost/{}", artwork_src)
    }
}

struct ProcessDirectoryResult {
    songs: HashMap<String, Song>,
    albums: HashMap<String, Album>,
}

fn process_directory(
    directory_path: &Path,
    songs: &Arc<std::sync::Mutex<HashMap<String, Song>>>,
    albums: &Arc<std::sync::Mutex<HashMap<String, Album>>>,
    artwork_origins: &Arc<std::sync::Mutex<HashMap<String, ArtworkOrigin>>>,
    recursive: bool,
    process_albums: bool,
    is_cover_fullcheck: bool,
    app: &AppHandle,
) -> Option<ProcessDirectoryResult> {
    let subsongs: Arc<std::sync::Mutex<HashMap<String, Song>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let subalbums: Arc<std::sync::Mutex<HashMap<String, Album>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let settings = load_settings(app).ok();

    fs::read_dir(directory_path)
        .par_iter_mut()
        .for_each(|entries| {
            entries.by_ref().par_bridge().for_each(|entry| {
                if let Ok(entry) = entry {
                    let path = entry.path();

                    // info!("{:?}", entry.path());
                    if path.is_file() {
                        if let Some(mut song) = crate::metadata::extract_metadata(
                            &path,
                            true,
                            is_cover_fullcheck,
                            false,
                            false,
                            &app,
                        ) {
                            if process_albums {
                                if let Some(album) = process_new_album(
                                    &mut song,
                                    Some(albums),
                                    Some(&subalbums),
                                    Some(artwork_origins),
                                    &settings,
                                    app,
                                ) {
                                    let album_id = album.id.clone(); // clone once, on purpose

                                    // info!("Album: {:?}", album);
                                    let existing_album =
                                        albums.lock().unwrap().get_mut(&album_id).cloned();
                                    let existing_album_subalbums =
                                        subalbums.lock().unwrap().get_mut(&album_id).cloned();
                                    if let Some(_existing_album) = existing_album {
                                        // Merge with existing album
                                        albums.lock().unwrap().entry(album_id.clone()).and_modify(
                                            |a| {
                                                a.tracks_ids.push(song.id.clone());
                                            },
                                        );
                                    } else if let Some(_existing_album_subalbums) =
                                        existing_album_subalbums
                                    {
                                        // Merge with existing album
                                        subalbums
                                            .lock()
                                            .unwrap()
                                            .entry(album_id.clone())
                                            .and_modify(|a| {
                                                a.tracks_ids.push(song.id.clone());
                                            });
                                    } else {
                                        subalbums.lock().unwrap().insert(album_id.clone(), album);
                                    }
                                    song.album_id = Some(album_id);
                                }
                            }
                            song.artwork = None;
                            subsongs.lock().unwrap().insert(song.id.clone(), song);
                        }
                    } else if path.is_dir() && recursive {
                        if let Some(sub_results) = process_directory(
                            &path,
                            songs,
                            albums,
                            artwork_origins,
                            true,
                            process_albums,
                            is_cover_fullcheck,
                            app,
                        ) {
                            if !sub_results.songs.is_empty() {
                                songs.lock().unwrap().extend(sub_results.songs);
                            }
                            if !sub_results.albums.is_empty() {
                                albums.lock().unwrap().extend(sub_results.albums);
                            }
                        }
                    }
                }
            });
        });

    return Some(ProcessDirectoryResult {
        songs: subsongs.lock().unwrap().to_owned(),
        albums: subalbums.lock().unwrap().to_owned(),
    });
}

struct ProcessPlaylistResult {
    songs: HashMap<String, Song>,
}

fn process_playlist(playlist_path: &Path, app: &AppHandle) -> Option<ProcessPlaylistResult> {
    let subsongs: Arc<std::sync::Mutex<HashMap<String, Song>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let read_result = std::fs::read_to_string(playlist_path);

    if let Ok(content) = read_result {
        let playlist: Playlist = Playlist::from(content.as_str());

        for entry in playlist.list {
            if let Some(mut song) = crate::metadata::extract_metadata(
                Path::new(&entry.url),
                true,
                false,
                false,
                false,
                &app,
            ) {
                song.artwork = None;
                subsongs.lock().unwrap().insert(song.id.clone(), song);
            }
        }
    }

    return Some(ProcessPlaylistResult {
        songs: subsongs.lock().unwrap().to_owned(),
    });
}

trait AsStr {
    fn as_str(&self) -> &'static str;
    fn from_str(s: &str) -> Self
    where
        Self: Sized;
}

impl AsStr for ItemKey {
    fn as_str(&self) -> &'static str {
        match self {
            // Titles
            Self::AlbumTitle => "AlbumTitle",
            Self::SetSubtitle => "SetSubtitle",
            Self::ShowName => "ShowName",
            Self::ContentGroup => "ContentGroup",
            Self::TrackTitle => "TrackTitle",
            Self::TrackSubtitle => "TrackSubtitle",
            Self::OriginalAlbumTitle => "OriginalAlbumTitle",
            Self::OriginalArtist => "OriginalArtist",
            Self::OriginalLyricist => "OriginalLyricist",
            Self::AlbumTitleSortOrder => "AlbumTitleSortOrder",
            Self::AlbumArtistSortOrder => "AlbumArtistSortOrder",
            Self::TrackTitleSortOrder => "TrackTitleSortOrder",
            Self::TrackArtistSortOrder => "TrackArtistSortOrder",
            Self::ShowNameSortOrder => "ShowNameSortOrder",
            Self::ComposerSortOrder => "ComposerSortOrder",
            Self::AlbumArtist => "AlbumArtist",
            Self::TrackArtist => "TrackArtist",
            Self::TrackArtists => "TrackArtists",
            Self::Arranger => "Arranger",
            Self::Writer => "Writer",
            Self::Composer => "Composer",
            Self::Conductor => "Conductor",
            Self::Director => "Director",
            Self::Engineer => "Engineer",
            Self::Lyricist => "Lyricist",
            Self::MixDj => "MixDj",
            Self::MixEngineer => "MixEngineer",
            Self::MusicianCredits => "MusicianCredits",
            Self::Performer => "Performer",
            Self::Producer => "Producer",
            Self::Publisher => "Publisher",
            Self::Label => "Label",
            Self::InternetRadioStationName => "InternetRadioStationName",
            Self::InternetRadioStationOwner => "InternetRadioStationOwner",
            Self::Remixer => "Remixer",
            Self::DiscNumber => "DiscNumber",
            Self::DiscTotal => "DiscTotal",
            Self::TrackNumber => "TrackNumber",
            Self::TrackTotal => "TrackTotal",
            Self::Popularimeter => "Popularimeter",
            Self::ParentalAdvisory => "ParentalAdvisory",
            Self::RecordingDate => "RecordingDate",
            Self::Year => "Year",
            Self::ReleaseDate => "ReleaseDate",
            Self::OriginalReleaseDate => "OriginalReleaseDate",
            Self::Isrc => "Isrc",
            Self::Barcode => "Barcode",
            Self::CatalogNumber => "CatalogNumber",
            Self::Work => "Work",
            Self::Movement => "Movement",
            Self::MovementNumber => "MovementNumber",
            Self::MovementTotal => "MovementTotal",
            Self::MusicBrainzRecordingId => "MusicBrainzRecordingId",
            Self::MusicBrainzTrackId => "MusicBrainzTrackId",
            Self::MusicBrainzReleaseId => "MusicBrainzReleaseId",
            Self::MusicBrainzReleaseGroupId => "MusicBrainzReleaseGroupId",
            Self::MusicBrainzArtistId => "MusicBrainzArtistId",
            Self::MusicBrainzReleaseArtistId => "MusicBrainzReleaseArtistId",
            Self::MusicBrainzWorkId => "MusicBrainzWorkId",
            Self::FlagCompilation => "FlagCompilation",
            Self::FlagPodcast => "FlagPodcast",
            Self::FileType => "FileType",
            Self::FileOwner => "FileOwner",
            Self::TaggingTime => "TaggingTime",
            Self::Length => "Length",
            Self::OriginalFileName => "OriginalFileName",
            Self::OriginalMediaType => "OriginalMediaType",
            Self::EncodedBy => "EncodedBy",
            Self::EncoderSoftware => "EncoderSoftware",
            Self::EncoderSettings => "EncoderSettings",
            Self::EncodingTime => "EncodingTime",
            Self::ReplayGainAlbumGain => "ReplayGainAlbumGain",
            Self::ReplayGainAlbumPeak => "ReplayGainAlbumPeak",
            Self::ReplayGainTrackGain => "ReplayGainTrackGain",
            Self::ReplayGainTrackPeak => "ReplayGainTrackPeak",
            Self::AudioFileUrl => "AudioFileUrl",
            Self::AudioSourceUrl => "AudioSourceUrl",
            Self::CommercialInformationUrl => "CommercialInformationUrl",
            Self::CopyrightUrl => "CopyrightUrl",
            Self::TrackArtistUrl => "TrackArtistUrl",
            Self::RadioStationUrl => "RadioStationUrl",
            Self::PaymentUrl => "PaymentUrl",
            Self::PublisherUrl => "PublisherUrl",
            Self::Genre => "Genre",
            Self::InitialKey => "InitialKey",
            Self::Color => "Color",
            Self::Mood => "Mood",
            Self::Bpm => "Bpm",
            Self::IntegerBpm => "IntegerBpm",
            Self::CopyrightMessage => "CopyrightMessage",
            Self::License => "License",
            Self::PodcastDescription => "PodcastDescription",
            Self::PodcastSeriesCategory => "PodcastSeriesCategory",
            Self::PodcastUrl => "PodcastUrl",
            Self::PodcastGlobalUniqueId => "PodcastGlobalUniqueId",
            Self::PodcastKeywords => "PodcastKeywords",
            Self::Comment => "Comment",
            Self::Description => "Description",
            Self::Language => "Language",
            Self::Script => "Script",
            Self::Lyrics => "Lyrics",
            Self::AppleXid => "AppleXid",
            Self::AppleId3v2ContentGroup => "AppleId3v2ContentGroup",
            &_ => "Unknown",
        }
    }

    fn from_str(s: &str) -> Self
    where
        Self: Sized,
    {
        match s {
            "AlbumTitle" => Self::AlbumTitle,
            "SetSubtitle" => Self::SetSubtitle,
            "ShowName" => Self::ShowName,
            "ContentGroup" => Self::ContentGroup,
            "TrackTitle" => Self::TrackTitle,
            "TrackSubtitle" => Self::TrackSubtitle,
            "OriginalAlbumTitle" => Self::OriginalAlbumTitle,
            "OriginalArtist" => Self::OriginalArtist,
            "OriginalLyricist" => Self::OriginalLyricist,
            "AlbumTitleSortOrder" => Self::AlbumTitleSortOrder,
            "AlbumArtistSortOrder" => Self::AlbumArtistSortOrder,
            "TrackTitleSortOrder" => Self::TrackTitleSortOrder,
            "TrackArtistSortOrder" => Self::TrackArtistSortOrder,
            "ShowNameSortOrder" => Self::ShowNameSortOrder,
            "ComposerSortOrder" => Self::ComposerSortOrder,
            "AlbumArtist" => Self::AlbumArtist,
            "TrackArtist" => Self::TrackArtist,
            "TrackArtists" => Self::TrackArtists,
            "Arranger" => Self::Arranger,
            "Writer" => Self::Writer,
            "Composer" => Self::Composer,
            "Conductor" => Self::Conductor,
            "Director" => Self::Director,
            "Engineer" => Self::Engineer,
            "Lyricist" => Self::Lyricist,
            "MixDj" => Self::MixDj,
            "MixEngineer" => Self::MixEngineer,
            "MusicianCredits" => Self::MusicianCredits,
            "Performer" => Self::Performer,
            "Producer" => Self::Producer,
            "Publisher" => Self::Publisher,
            "Label" => Self::Label,
            "InternetRadioStationName" => Self::InternetRadioStationName,
            "InternetRadioStationOwner" => Self::InternetRadioStationOwner,
            "Remixer" => Self::Remixer,
            "DiscNumber" => Self::DiscNumber,
            "DiscTotal" => Self::DiscTotal,
            "TrackNumber" => Self::TrackNumber,
            "TrackTotal" => Self::TrackTotal,
            "Popularimeter" => Self::Popularimeter,
            "ParentalAdvisory" => Self::ParentalAdvisory,
            "RecordingDate" => Self::RecordingDate,
            "Year" => Self::Year,
            "ReleaseDate" => Self::ReleaseDate,
            "OriginalReleaseDate" => Self::OriginalReleaseDate,
            "Isrc" => Self::Isrc,
            "Barcode" => Self::Barcode,
            "CatalogNumber" => Self::CatalogNumber,
            "Work" => Self::Work,
            "Movement" => Self::Movement,
            "MovementNumber" => Self::MovementNumber,
            "MovementTotal" => Self::MovementTotal,
            "MusicBrainzRecordingId" => Self::MusicBrainzRecordingId,
            "MusicBrainzTrackId" => Self::MusicBrainzTrackId,
            "MusicBrainzReleaseId" => Self::MusicBrainzReleaseId,
            "MusicBrainzReleaseGroupId" => Self::MusicBrainzReleaseGroupId,
            "MusicBrainzArtistId" => Self::MusicBrainzArtistId,
            "MusicBrainzReleaseArtistId" => Self::MusicBrainzReleaseArtistId,
            "MusicBrainzWorkId" => Self::MusicBrainzWorkId,
            "FlagCompilation" => Self::FlagCompilation,
            "FlagPodcast" => Self::FlagPodcast,
            "FileType" => Self::FileType,
            "FileOwner" => Self::FileOwner,
            "TaggingTime" => Self::TaggingTime,
            "Length" => Self::Length,
            "OriginalFileName" => Self::OriginalFileName,
            "OriginalMediaType" => Self::OriginalMediaType,
            "EncodedBy" => Self::EncodedBy,
            "EncoderSoftware" => Self::EncoderSoftware,
            "EncoderSettings" => Self::EncoderSettings,
            "EncodingTime" => Self::EncodingTime,
            "ReplayGainAlbumGain" => Self::ReplayGainAlbumGain,
            "ReplayGainAlbumPeak" => Self::ReplayGainAlbumPeak,
            "ReplayGainTrackGain" => Self::ReplayGainTrackGain,
            "ReplayGainTrackPeak" => Self::ReplayGainTrackPeak,
            "AudioFileUrl" => Self::AudioFileUrl,
            "AudioSourceUrl" => Self::AudioSourceUrl,
            "CommercialInformationUrl" => Self::CommercialInformationUrl,
            "CopyrightUrl" => Self::CopyrightUrl,
            "TrackArtistUrl" => Self::TrackArtistUrl,
            "RadioStationUrl" => Self::RadioStationUrl,
            "PaymentUrl" => Self::PaymentUrl,
            "PublisherUrl" => Self::PublisherUrl,
            "Genre" => Self::Genre,
            "InitialKey" => Self::InitialKey,
            "Color" => Self::Color,
            "Mood" => Self::Mood,
            "Bpm" => Self::Bpm,
            "IntegerBpm" => Self::IntegerBpm,
            "CopyrightMessage" => Self::CopyrightMessage,
            "License" => Self::License,
            "PodcastDescription" => Self::PodcastDescription,
            "PodcastSeriesCategory" => Self::PodcastSeriesCategory,
            "PodcastUrl" => Self::PodcastUrl,
            "PodcastGlobalUniqueId" => Self::PodcastGlobalUniqueId,
            "PodcastKeywords" => Self::PodcastKeywords,
            "Comment" => Self::Comment,
            "Description" => Self::Description,
            "Language" => Self::Language,
            "Script" => Self::Script,
            "Lyrics" => Self::Lyrics,
            "AppleXid" => Self::AppleXid,
            "AppleId3v2ContentGroup" => Self::AppleId3v2ContentGroup,
            &_ => Self::Unknown(s.to_string()),
        }
    }
}

pub fn extract_metadata(
    file_path: &Path,
    is_import: bool,
    is_cover_fullcheck: bool,
    include_folder_artwork: bool,
    include_raw_tags: bool,
    app: &AppHandle,
) -> Option<Song> {
    info!("extract_metadata: {}", file_path.display());
    if let Some(extension) = file_path.extension() {
        info!("extension: {:?}", extension);
        if let Some(ext_str) = extension.to_str() {
            info!("ext: {}", ext_str);
            if ext_str.eq_ignore_ascii_case("mp3")
                || ext_str.eq_ignore_ascii_case("flac")
                || ext_str.eq_ignore_ascii_case("wav")
                || ext_str.eq_ignore_ascii_case("aiff")
                || ext_str.eq_ignore_ascii_case("ape")
                || ext_str.eq_ignore_ascii_case("ogg")
                || ext_str.eq_ignore_ascii_case("m4a")
            {
                match read_from_path(&file_path) {
                    Ok(tagged_file) => {
                        let id =
                            MD5::hash(file_path.to_str().unwrap().as_bytes()).to_hex_lowercase();
                        let path = file_path.to_string_lossy().into_owned();
                        let file = file_path.file_name()?.to_string_lossy().into_owned();

                        let mut title = String::new();
                        let mut artist = String::new();
                        let mut album = String::new();
                        let mut album_artist = None;
                        let mut compilation = 0;
                        let mut year = 0;
                        let mut genre = Vec::new();
                        let mut composer = Vec::new();
                        let mut track_number = -1;
                        let mut track_total = -1;
                        let mut disc_number = -1;
                        let mut disc_total = -1;
                        let mut duration = String::new();
                        let file_info;
                        let mut artwork = None;
                        let mut artwork_origin = None;
                        let mut metadata: HashMap<String, MetadataEntry> = HashMap::new();

                        // info!("bit depth {:?}", tagged_file.properties().bit_depth());
                        file_info = FileInfo {
                            duration: Some(tagged_file.properties().duration().as_secs_f64()),
                            duration_display: Some(seconds_to_hms(
                                tagged_file.properties().duration().as_secs(),
                            )),
                            channels: tagged_file.properties().channels(),
                            bit_depth: tagged_file.properties().bit_depth().or(Some(16)),
                            sample_rate: tagged_file.properties().sample_rate(),
                            audio_bitrate: tagged_file.properties().audio_bitrate(),
                            overall_bitrate: tagged_file.properties().overall_bitrate(),
                            lossless: vec![FileType::Flac, FileType::Wav]
                                .iter()
                                .any(|f| f.eq(&tagged_file.file_type())),
                            tag_type: if let Some(tag) = tagged_file.primary_tag() {
                                match tag.tag_type() {
                                    TagType::VorbisComments => Some("vorbis".to_string()),
                                    TagType::Id3v1 => Some("ID3v1".to_string()),
                                    TagType::Id3v2 => Some("ID3v2".to_string()),
                                    TagType::Mp4Ilst => Some("MP4".to_string()),
                                    TagType::Ape | TagType::RiffInfo | TagType::AiffText => None,
                                    _ => todo!(),
                                }
                            } else {
                                match tagged_file.file_type() {
                                    FileType::Flac | FileType::Vorbis => Some("vorbis".to_string()),
                                    FileType::Wav => Some("ID3v2".to_string()),
                                    FileType::Mpeg => Some("ID3v2".to_string()),
                                    FileType::Ape | FileType::Opus | FileType::Speex => None,
                                    _ => None,
                                }
                            },
                            codec: match tagged_file.file_type() {
                                FileType::Flac => Some("FLAC".to_string()),
                                FileType::Mpeg => Some("MPEG".to_string()),
                                FileType::Aiff => Some("AIFF".to_string()),
                                FileType::Wav => Some("WAV".to_string()),
                                FileType::Ape => Some("APE".to_string()),
                                FileType::Opus => Some("Opus".to_string()),
                                FileType::Speex => Some("Speex".to_string()),
                                FileType::Vorbis => Some("Vorbis".to_string()),
                                FileType::Mp4 => Some("MP4".to_string()),
                                _ => None,
                            },
                        };

                        duration = file_info.duration_display.clone().unwrap_or_default();

                        if tagged_file.tags().is_empty() {
                            title = file.to_string();
                        }

                        if let Some(tag) = tagged_file.primary_tag().or(tagged_file.first_tag()) {
                            for item in tag.items() {
                                // info!("Tag type {:?}", tag.tag_type());
                                // info!("Tag items {:?}", tag.items());
                                if title.is_empty() {
                                    title = tag
                                        .title()
                                        .filter(|x| !x.is_empty())
                                        .unwrap_or(std::borrow::Cow::Borrowed(&file))
                                        .to_string();
                                }
                                if artist.is_empty() {
                                    artist = tag.artist().unwrap_or_default().to_string();
                                }
                                if album.is_empty() {
                                    album = tag.album().unwrap_or_default().to_string();
                                }
                                if album_artist.is_none() {
                                    // album_artist = Some(tag.get_string(&ItemKey::AlbumArtist).unwrap_or_default().to_string());
                                    album_artist = tag
                                        .get_string(&ItemKey::AlbumArtist)
                                        .map(|s| s.to_string());
                                }
                                if compilation == 0 {
                                    compilation = tag
                                        .get_string(&ItemKey::FlagCompilation)
                                        .unwrap_or_default()
                                        .parse::<u32>()
                                        .ok()
                                        .unwrap_or(0)
                                        as i32;
                                }
                                if genre.is_empty() {
                                    genre = tag.genre().map_or_else(Vec::new, |g| {
                                        g.split('/').map(String::from).collect()
                                    });
                                }
                                if year == 0 {
                                    year = tag.year().unwrap_or(0) as i32;
                                }
                                if composer.is_empty() {
                                    composer = tag
                                        .get_items(&ItemKey::Composer)
                                        .map(|c| {
                                            c.value().to_owned().into_string().unwrap_or_default()
                                        })
                                        .clone()
                                        .collect()
                                }
                                if track_number == -1 {
                                    track_number = tag.track().unwrap_or(0) as i32;
                                }
                                if track_total == -1 {
                                    track_total = tag.track_total().unwrap_or(0) as i32;
                                }
                                if disc_number == -1 {
                                    disc_number = tag.disk().unwrap_or(0) as i32;
                                }
                                if disc_total == -1 {
                                    disc_total = tag.disk_total().unwrap_or(0) as i32;
                                }

                                if include_raw_tags {
                                    // Lofty's ItemKey representation (we don't expose internal tag keys to the UI)
                                    let key = item.key().as_str().to_string();
                                    let value =
                                        tag.get_string(item.key()).unwrap_or_default().to_string();

                                    metadata.insert(
                                        key.clone(),
                                        MetadataEntry {
                                            id: key,
                                            value: Some(Some(value)),
                                        },
                                    );
                                }
                            }
                        }

                        if tagged_file.primary_tag().is_some() {
                            if let Some(pic) = tagged_file.primary_tag().unwrap().pictures().first()
                            {
                                let mut decoded = false;
                                let mut format = String::new();

                                if let Some(mime) = pic.mime_type() {
                                    format = mime.to_string();
                                } else {
                                    let mime = mimetype::detect(pic.data());

                                    format = mime.mime.to_string();
                                }

                                match format.as_str() {
                                    "image/jpeg" => {
                                        let mut decoder = zune_jpeg::JpegDecoder::new(pic.data());

                                        decoded = if is_cover_fullcheck {
                                            decoder.decode().is_ok()
                                        } else {
                                            decoder.decode_headers().is_ok()
                                        };
                                    }
                                    "image/png" => {
                                        let mut decoder = zune_png::PngDecoder::new(pic.data());

                                        decoded = if is_cover_fullcheck {
                                            decoder.decode().is_ok()
                                        } else {
                                            decoder.decode_headers().is_ok()
                                        };
                                    }
                                    _ => {}
                                };

                                if decoded {
                                    artwork_origin = Some(ArtworkOrigin::Metadata);
                                    artwork = Some(Artwork {
                                        data: pic.data().to_vec(),
                                        src: None,
                                        format,
                                    })
                                } else {
                                    artwork_origin = Some(ArtworkOrigin::Broken);
                                }
                            }
                        }

                        if include_folder_artwork && artwork.is_none() {
                            if let Some(settings) = load_settings(app).ok() {
                                let result = look_for_art(&path, &file, &settings, app);
                                if let Ok(res) = result {
                                    if let Some(art) = res.clone() {
                                        artwork_origin = Some(ArtworkOrigin::File);
                                        artwork = Some(Artwork {
                                            data: vec![],
                                            src: Some(art.artwork_src),
                                            format: art.artwork_format,
                                        })
                                    } else {
                                        artwork_origin = Some(ArtworkOrigin::NotFound);
                                    }
                                } else if let Err(e) = result {
                                    info!("Error looking for artwork: {}", e);

                                    artwork_origin = Some(ArtworkOrigin::NotFound);
                                }
                            }
                        }

                        info!("artwork_origin: {:?}", artwork_origin);

                        let start = SystemTime::now();
                        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap().as_millis();

                        return Some(Song {
                            id,
                            path,
                            file,
                            title,
                            artist,
                            album_id: None,
                            album,
                            album_artist,
                            compilation,
                            year,
                            genre,
                            composer,
                            track_number,
                            track_total,
                            disc_number,
                            disc_total,
                            duration,
                            file_info,
                            metadata,
                            artwork,
                            artwork_origin,
                            // We default the origin country to "" to allow Dexie to return results when using orderBy,
                            // even if there are zero songs with a non-empty country
                            origin_country: Some(String::from("")),
                            origin_country_name: Some(String::from("")),
                            date_added: if is_import {
                                Some(since_the_epoch)
                            } else {
                                None
                            },
                        });
                    }
                    Err(e) => {
                        info!("Error reading file: {}", e);
                        return None;
                    }
                }
            }
        }
    }
    None
}

fn write_metadata_track(v: &WriteMetatadaEvent) -> Result<(), anyhow::Error> {
    // info!("got event-name with payload {:?}", event.payload());

    // Parse JSON
    // info!("v {:?}", v);
    if v.tag_type.is_some() {
        // We know which tag type this is, continue with writing...

        // info!("{:?}", v.metadata);
        let tag_type_evt = v.tag_type.as_deref().unwrap();
        let tag_type = match tag_type_evt.to_lowercase().as_str() {
            "vorbis" => TagType::VorbisComments,
            "id3v1" => TagType::Id3v1,
            "id3v2" => TagType::Id3v2,
            "id3v2.2" => TagType::Id3v2,
            "id3v2.3" => TagType::Id3v2,
            "id3v2.4" => TagType::Id3v2,
            "mp4ilst" => TagType::Mp4Ilst,
            _ => panic!("Unhandled tag type: {:?}", v.tag_type),
        };
        info!("tag fileType: {:?}", &tag_type);
        let options = ParseOptions::new().max_junk_bytes(2048);

        let mut file = File::options().read(true).write(true).open(&v.file_path)?;

        let reader = BufReader::new(&file);
        let probe = Probe::new(reader).options(options);
        // Try to guess the file type
        let probe = probe.guess_file_type();
        // Handle error
        let probe = match probe {
            Ok(probe) => probe,
            Err(e) => {
                info!("Error probing file: {}", e);
                return Err(anyhow::anyhow!(e));
            }
        };

        let file_type = &probe.file_type();
        info!("fileType: {:?}", &file_type);
        // let mut tagged_file = read_from_path(&v.file_path)?;
        let mut tagged_file = probe.read().expect("ERROR: Failed to read file!");

        let tag = if let Some(primary_tag) = tagged_file.primary_tag_mut() {
            primary_tag
        } else if let Some(first_tag) = tagged_file.first_tag_mut() {
            first_tag
        } else {
            tagged_file.insert_tag(lofty::tag::Tag::new(tag_type));

            tagged_file.first_tag_mut().unwrap()
        };

        if tag.tag_type() == TagType::Id3v1 || tag.tag_type() == TagType::Id3v2 {
            // upgrade to ID3v2.4
            info!("Upgrading to ID3v2.4");
            tag.re_map(TagType::Id3v2);
        }

        tag.remove_empty();

        for item in v.metadata.iter() {
            println!("To write item: {:?}\n", item);
            if item.id == "METADATA_BLOCK_PICTURE" {
                // Ignore picture, set by artwork_file_to_set
            } else if item.id == "year" {
                match &item.value {
                    None => {}
                    Some(None) => tag.remove_year(),
                    Some(Some(year)) => {
                        tag.set_year(year.parse().unwrap());
                    }
                }
            } else {
                let item_key = ItemKey::from_str(&item.id.as_str());

                match &item.value {
                    None => {}
                    Some(None) => tag.remove_key(&item_key),
                    Some(Some(value)) => {
                        let item_value: ItemValue = ItemValue::Text(value.clone());
                        println!("WRITING: {:?} {:?}", item_key, item_value);

                        tag.insert(TagItem::new(item_key, item_value));
                    }
                }
            }
        }

        // Delete artwork if requested
        if v.delete_artwork && tag.pictures().len() > 0 {
            while tag.pictures().len() > 0 {
                tag.remove_picture(0);
            }
        }
        // Set artwork if provided
        else if !v.artwork_file.is_empty() {
            let picture_file = File::options()
                .read(true)
                .write(true)
                .open(&Path::new(&v.artwork_file))?;

            let mut reader = BufReader::new(picture_file);
            let pic = Picture::from_reader(reader.get_mut());
            tag.set_picture(0, pic.unwrap());
        } else if !v.artwork_data.is_empty() {
            let mime_type = if let Some(mime) = &v.artwork_data_mime_type {
                match mime.as_str() {
                    "image/jpeg" => MimeType::Jpeg,
                    "image/png" => MimeType::Png,
                    "image/tiff" => MimeType::Tiff,
                    _ => MimeType::Unknown(mime.to_string()),
                }
            } else {
                MimeType::Png
            };
            let pic = Picture::new_unchecked(
                lofty::picture::PictureType::CoverFront,
                Some(mime_type),
                None,
                v.artwork_data.clone(),
            );
            tag.set_picture(0, pic);
        }

        // Keep picture, overwrite everything else
        let _pictures = tag.pictures();

        // Writing file

        let mut file = File::options().read(true).write(true).open(&v.file_path)?;

        let reader = BufReader::new(&file);
        let probe = Probe::new(reader).options(options);
        // Try to guess the file type
        let probe = probe.guess_file_type()?;
        let file_type = &probe.file_type();
        info!("fileType just before save: {:?}", &file_type);
        tagged_file.save_to(&mut file, WriteOptions::new())?;
        info!("File saved succesfully!");
    } else {
        info!("tagType is missing");
    }
    Ok(())
    // println("title:")
}

fn seconds_to_hms(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    if hours == 0 {
        format!("{:02}:{:02}", minutes, seconds)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

pub fn country_name(code: &str) -> Option<&'static str> {
    match code {
        "AF" => Some("Afghanistan"),
        "AX" => Some("Aland Islands"),
        "AL" => Some("Albania"),
        "DZ" => Some("Algeria"),
        "AS" => Some("American Samoa"),
        "AD" => Some("Andorra"),
        "AO" => Some("Angola"),
        "AI" => Some("Anguilla"),
        "AQ" => Some("Antarctica"),
        "AG" => Some("Antigua And Barbuda"),
        "AR" => Some("Argentina"),
        "AM" => Some("Armenia"),
        "AW" => Some("Aruba"),
        "AU" => Some("Australia"),
        "AT" => Some("Austria"),
        "AZ" => Some("Azerbaijan"),
        "BS" => Some("Bahamas"),
        "BH" => Some("Bahrain"),
        "BD" => Some("Bangladesh"),
        "BB" => Some("Barbados"),
        "BY" => Some("Belarus"),
        "BE" => Some("Belgium"),
        "BZ" => Some("Belize"),
        "BJ" => Some("Benin"),
        "BM" => Some("Bermuda"),
        "BT" => Some("Bhutan"),
        "BO" => Some("Bolivia"),
        "BA" => Some("Bosnia And Herzegovina"),
        "BW" => Some("Botswana"),
        "BV" => Some("Bouvet Island"),
        "BR" => Some("Brazil"),
        "IO" => Some("British Indian Ocean Territory"),
        "BN" => Some("Brunei Darussalam"),
        "BG" => Some("Bulgaria"),
        "BF" => Some("Burkina Faso"),
        "BI" => Some("Burundi"),
        "KH" => Some("Cambodia"),
        "CM" => Some("Cameroon"),
        "CA" => Some("Canada"),
        "CV" => Some("Cape Verde"),
        "KY" => Some("Cayman Islands"),
        "CF" => Some("Central African Republic"),
        "TD" => Some("Chad"),
        "CL" => Some("Chile"),
        "CN" => Some("China"),
        "CX" => Some("Christmas Island"),
        "CC" => Some("Cocos (Keeling) Islands"),
        "CO" => Some("Colombia"),
        "KM" => Some("Comoros"),
        "CG" => Some("Congo"),
        "CD" => Some("Congo, Democratic Republic"),
        "CK" => Some("Cook Islands"),
        "CR" => Some("Costa Rica"),
        "CI" => Some("Cote D'Ivoire"),
        "HR" => Some("Croatia"),
        "CU" => Some("Cuba"),
        "CY" => Some("Cyprus"),
        "CZ" => Some("Czech Republic"),
        "DK" => Some("Denmark"),
        "DJ" => Some("Djibouti"),
        "DM" => Some("Dominica"),
        "DO" => Some("Dominican Republic"),
        "EC" => Some("Ecuador"),
        "EG" => Some("Egypt"),
        "SV" => Some("El Salvador"),
        "GQ" => Some("Equatorial Guinea"),
        "ER" => Some("Eritrea"),
        "EE" => Some("Estonia"),
        "ET" => Some("Ethiopia"),
        "FK" => Some("Falkland Islands (Malvinas)"),
        "FO" => Some("Faroe Islands"),
        "FJ" => Some("Fiji"),
        "FI" => Some("Finland"),
        "FR" => Some("France"),
        "GF" => Some("French Guiana"),
        "PF" => Some("French Polynesia"),
        "TF" => Some("French Southern Territories"),
        "GA" => Some("Gabon"),
        "GM" => Some("Gambia"),
        "GE" => Some("Georgia"),
        "DE" => Some("Germany"),
        "GH" => Some("Ghana"),
        "GI" => Some("Gibraltar"),
        "GR" => Some("Greece"),
        "GL" => Some("Greenland"),
        "GD" => Some("Grenada"),
        "GP" => Some("Guadeloupe"),
        "GU" => Some("Guam"),
        "GT" => Some("Guatemala"),
        "GG" => Some("Guernsey"),
        "GN" => Some("Guinea"),
        "GW" => Some("Guinea-Bissau"),
        "GY" => Some("Guyana"),
        "HT" => Some("Haiti"),
        "HM" => Some("Heard Island & Mcdonald Islands"),
        "VA" => Some("Holy See (Vatican City State)"),
        "HN" => Some("Honduras"),
        "HK" => Some("Hong Kong"),
        "HU" => Some("Hungary"),
        "IS" => Some("Iceland"),
        "IN" => Some("India"),
        "ID" => Some("Indonesia"),
        "IR" => Some("Iran, Islamic Republic Of"),
        "IQ" => Some("Iraq"),
        "IE" => Some("Ireland"),
        "IM" => Some("Isle Of Man"),
        "IL" => Some("Israel"),
        "IT" => Some("Italy"),
        "JM" => Some("Jamaica"),
        "JP" => Some("Japan"),
        "JE" => Some("Jersey"),
        "JO" => Some("Jordan"),
        "KZ" => Some("Kazakhstan"),
        "KE" => Some("Kenya"),
        "KI" => Some("Kiribati"),
        "KR" => Some("Korea"),
        "KP" => Some("North Korea"),
        "KW" => Some("Kuwait"),
        "KG" => Some("Kyrgyzstan"),
        "LA" => Some("Lao People's Democratic Republic"),
        "LV" => Some("Latvia"),
        "LB" => Some("Lebanon"),
        "LS" => Some("Lesotho"),
        "LR" => Some("Liberia"),
        "LY" => Some("Libyan Arab Jamahiriya"),
        "LI" => Some("Liechtenstein"),
        "LT" => Some("Lithuania"),
        "LU" => Some("Luxembourg"),
        "MO" => Some("Macao"),
        "MK" => Some("Macedonia"),
        "MG" => Some("Madagascar"),
        "MW" => Some("Malawi"),
        "MY" => Some("Malaysia"),
        "MV" => Some("Maldives"),
        "ML" => Some("Mali"),
        "MT" => Some("Malta"),
        "MH" => Some("Marshall Islands"),
        "MQ" => Some("Martinique"),
        "MR" => Some("Mauritania"),
        "MU" => Some("Mauritius"),
        "YT" => Some("Mayotte"),
        "MX" => Some("Mexico"),
        "FM" => Some("Micronesia, Federated States Of"),
        "MD" => Some("Moldova"),
        "MC" => Some("Monaco"),
        "MN" => Some("Mongolia"),
        "ME" => Some("Montenegro"),
        "MS" => Some("Montserrat"),
        "MA" => Some("Morocco"),
        "MZ" => Some("Mozambique"),
        "MM" => Some("Myanmar"),
        "NA" => Some("Namibia"),
        "NR" => Some("Nauru"),
        "NP" => Some("Nepal"),
        "NL" => Some("Netherlands"),
        "AN" => Some("Netherlands Antilles"),
        "NC" => Some("New Caledonia"),
        "NZ" => Some("New Zealand"),
        "NI" => Some("Nicaragua"),
        "NE" => Some("Niger"),
        "NG" => Some("Nigeria"),
        "NU" => Some("Niue"),
        "NF" => Some("Norfolk Island"),
        "MP" => Some("Northern Mariana Islands"),
        "NO" => Some("Norway"),
        "OM" => Some("Oman"),
        "PK" => Some("Pakistan"),
        "PW" => Some("Palau"),
        "PS" => Some("Palestinian Territory, Occupied"),
        "PA" => Some("Panama"),
        "PG" => Some("Papua New Guinea"),
        "PY" => Some("Paraguay"),
        "PE" => Some("Peru"),
        "PH" => Some("Philippines"),
        "PN" => Some("Pitcairn"),
        "PL" => Some("Poland"),
        "PT" => Some("Portugal"),
        "PR" => Some("Puerto Rico"),
        "QA" => Some("Qatar"),
        "RE" => Some("Reunion"),
        "RO" => Some("Romania"),
        "RU" => Some("Russia"),
        "RW" => Some("Rwanda"),
        "BL" => Some("Saint Barthelemy"),
        "SH" => Some("Saint Helena"),
        "KN" => Some("Saint Kitts And Nevis"),
        "LC" => Some("Saint Lucia"),
        "MF" => Some("Saint Martin"),
        "PM" => Some("Saint Pierre And Miquelon"),
        "VC" => Some("Saint Vincent And Grenadines"),
        "WS" => Some("Samoa"),
        "SM" => Some("San Marino"),
        "ST" => Some("Sao Tome And Principe"),
        "SA" => Some("Saudi Arabia"),
        "SN" => Some("Senegal"),
        "RS" => Some("Serbia"),
        "SC" => Some("Seychelles"),
        "SL" => Some("Sierra Leone"),
        "SG" => Some("Singapore"),
        "SK" => Some("Slovakia"),
        "SI" => Some("Slovenia"),
        "SB" => Some("Solomon Islands"),
        "SO" => Some("Somalia"),
        "ZA" => Some("South Africa"),
        "GS" => Some("South Georgia And Sandwich Isl."),
        "ES" => Some("Spain"),
        "LK" => Some("Sri Lanka"),
        "SD" => Some("Sudan"),
        "SR" => Some("Suriname"),
        "SJ" => Some("Svalbard And Jan Mayen"),
        "SZ" => Some("Swaziland"),
        "SE" => Some("Sweden"),
        "CH" => Some("Switzerland"),
        "SY" => Some("Syrian Arab Republic"),
        "TW" => Some("Taiwan"),
        "TJ" => Some("Tajikistan"),
        "TZ" => Some("Tanzania"),
        "TH" => Some("Thailand"),
        "TL" => Some("Timor-Leste"),
        "TG" => Some("Togo"),
        "TK" => Some("Tokelau"),
        "TO" => Some("Tonga"),
        "TT" => Some("Trinidad And Tobago"),
        "TN" => Some("Tunisia"),
        "TR" => Some("Turkey"),
        "TM" => Some("Turkmenistan"),
        "TC" => Some("Turks And Caicos Islands"),
        "TV" => Some("Tuvalu"),
        "UG" => Some("Uganda"),
        "UA" => Some("Ukraine"),
        "AE" => Some("United Arab Emirates"),
        "GB" => Some("United Kingdom"),
        "US" => Some("United States of America"),
        "UM" => Some("United States Outlying Islands"),
        "UY" => Some("Uruguay"),
        "UZ" => Some("Uzbekistan"),
        "VU" => Some("Vanuatu"),
        "VE" => Some("Venezuela"),
        "VN" => Some("Vietnam"),
        "VG" => Some("Virgin Islands, British"),
        "VI" => Some("Virgin Islands, U.S."),
        "WF" => Some("Wallis And Futuna"),
        "EH" => Some("Western Sahara"),
        "YE" => Some("Yemen"),
        "ZM" => Some("Zambia"),
        "ZW" => Some("Zimbabwe"),
        _ => None,
    }
}
