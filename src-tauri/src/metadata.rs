use artwork_cacher::look_for_art;
use chksum_md5::MD5;
use color_print::cprintln;
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, FileType, TaggedFileExt};
use lofty::id3::v2::{upgrade_v2, upgrade_v3, Frame, FrameId, Id3v2Tag, TextInformationFrame};
use lofty::picture::Picture;
use lofty::probe::Probe;
use lofty::read_from_path;
use lofty::tag::{Accessor, ItemKey, ItemValue, TagItem, TagType};
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, ErrorKind, Write};
use std::ops::{Deref, Mul};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::{fmt, thread, time};
use tauri::{Config, Manager};

mod artwork_cacher;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MetadataEntry {
    id: String,
    value: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WriteMetatadaEvent {
    metadata: Vec<MetadataEntry>,
    tag_type: Option<String>,
    file_path: String,
    artwork_file_to_set: String,
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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    duration: Option<f64>, //s
    overall_bitrate: Option<u32>,
    audio_bitrate: Option<u32>,
    sample_rate: Option<u32>,
    bit_depth: Option<u8>,
    pub channels: Option<u8>,
    lossless: bool,
    tag_type: Option<String>,
    codec: Option<String>,
}
impl FileInfo {
    fn new() -> FileInfo {
        FileInfo {
            duration: None,
            overall_bitrate: None,
            audio_bitrate: None,
            sample_rate: None,
            bit_depth: None,
            channels: None,
            lossless: false,
            tag_type: None,
            codec: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Artwork {
    data: Vec<u8>,
    format: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AlbumArtwork {
    src: String,
    format: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    id: String,
    path: String,
    file: String,
    title: String,
    artist: String,
    album: String,
    year: i32,
    genre: Vec<String>,
    composer: Vec<String>,
    track_number: i32,
    duration: String,
    file_info: FileInfo,
    artwork: Option<Artwork>,
    origin_country: Option<String>,
    date_added: Option<u128>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    id: String, // Hash of artist + album name
    title: String,
    artist: String,
    year: i32,
    genre: Vec<String>,
    tracks_ids: Vec<String>,
    path: String,
    artwork: Option<AlbumArtwork>,
    lossless: bool,
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
}

#[tauri::command]
pub async fn write_metadata(
    event: WriteMetatadaEvent,
    app_handle: tauri::AppHandle,
) -> ToImportEvent {
    println!("{:?}", event);

    // let payload: &str = event.payload().unwrap();
    let songs: Arc<std::sync::Mutex<Vec<Song>>> = Arc::new(Mutex::new(Vec::new()));

    // let v: WriteMetatadaEvent = serde_json::from_str(payload).unwrap();
    let write_result = write_metadata_track(&event);
    match write_result {
        Ok(()) => {
            let song = crate::metadata::extract_metadata(Path::new(&event.file_path), false);
            if song.is_some() {
                songs.lock().unwrap().push(song.unwrap());
            }
            // Emit result back to client
            app_handle.emit_all("write-success", {}).unwrap();
        }
        Err(err) => {
            println!("{}", err)
        }
    }

    let to_import = ToImportEvent {
        songs: songs.lock().unwrap().clone(),
        albums: vec![],
        progress: 100,
        done: true,
        error: None,
    };
    return to_import;
}

#[tauri::command]
pub async fn write_metadatas(
    event: WriteMetatadasEvent,
    _app_handle: tauri::AppHandle,
) -> ToImportEvent {
    println!("{:?}", event);
    // let payload: &str = event.payload().unwrap();
    let songs: Arc<std::sync::Mutex<Vec<Song>>> = Arc::new(Mutex::new(Vec::new()));

    let mut error: Option<String> = None;

    // let v: WriteMetatadaEvent = serde_json::from_str(payload).unwrap();
    for track in event.tracks.iter() {
        let write_result = write_metadata_track(&track.clone());
        match write_result {
            Ok(()) => {
                // Emit result back to client
                let song = crate::metadata::extract_metadata(Path::new(&track.file_path), false);
                if song.is_some() {
                    songs.lock().unwrap().push(song.unwrap());
                }
                println!("Wrote metadata")
            }
            Err(err) => {
                match err.downcast_ref::<std::io::Error>() {
                    Some(io_err) => match (io_err.kind()) {
                        ErrorKind::PermissionDenied => {
                            error.replace(String::from(
                                "Permission denied. Check your file permissions and try again",
                            ));
                            break;
                        }
                        _ => {
                            panic!("{}", io_err);
                        }
                    },
                    None => {}
                }
                println!("Error writing metadata: {}", err);
            }
        }
    }
    // Emit result back to client

    let to_import = ToImportEvent {
        songs: songs.lock().unwrap().clone(),
        albums: vec![],
        progress: 100,
        done: true,
        error: error,
    };
    return to_import;
}

#[tauri::command]
pub async fn get_song_metadata(event: GetSongMetadataEvent) -> Option<Song> {
    let path = Path::new(event.path.as_str());

    if path.is_file() {
        // println!("{:?}", entry.path());
        if let Some(song) = crate::metadata::extract_metadata(&path, event.is_import) {
            return Some(song);
        }
    }
    None
}

#[tauri::command]
pub async fn scan_paths(event: ScanPathsEvent, app_handle: tauri::AppHandle) -> ToImportEvent {
    // println!("scan_paths", event);
    let start = Instant::now();
    let songs: Arc<std::sync::Mutex<Vec<Song>>> = Arc::new(Mutex::new(Vec::new()));
    let albums: Arc<std::sync::Mutex<HashMap<String, Album>>> =
        Arc::new(Mutex::new(HashMap::new()));

    event.paths.par_iter().for_each(|p| {
        let path = Path::new(p.as_str());
        // println!("{:?}", path);

        if path.is_file() {
            if let Some(song) = crate::metadata::extract_metadata(&path, true) {
                if let Some(album) = process_new_album(&song, app_handle.config().as_ref()) {
                    println!("Album: {:?}", album);
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
        } else if path.is_dir() {
            if let Some(sub_results) = process_directory(
                Path::new(path),
                &songs,
                &albums,
                event.recursive,
                app_handle.config().as_ref(),
            ) {
                if (!sub_results.songs.is_empty()) {
                    songs.lock().unwrap().extend(sub_results.songs);
                }
                if (!sub_results.albums.is_empty()) {
                    albums.lock().unwrap().extend(sub_results.albums);
                }
            }
        }
    });

    // Print the collected songs for demonstration purposes
    // for song in songs.lock().unwrap().clone(){
    //     println!("{:?}", song);
    // }

    // First send all songs in chunks
    let length = songs.lock().unwrap().clone().len();
    if length > 500 {
        let songs_clone = songs.lock().unwrap();
        let enumerator = songs_clone.chunks(200);
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
            println!("{:?}", progress);
            let _ = app_handle.emit_all(
                "import_chunk",
                ToImportEvent {
                    songs: slice.to_vec(),
                    albums: vec![],
                    progress: progress,
                    done: progress == 100 && albums.lock().unwrap().clone().len() == 0,
                    error: None,
                },
            );
        });
    } else {
        // Send album artworks first - client needs to process the songs, which in turn needs to process the albums
        // Once this is done the client can update all existing albums with the artworks
        // The done flag is true - this determines when the client is done importing
        let _ = app_handle.emit_all(
            "import_albums",
            ToImportEvent {
                songs: vec![],
                albums: albums.lock().unwrap().values().cloned().collect(),
                progress: 100,
                done: true,
                error: None,
            },
        );
        let _ = app_handle.emit_all(
            "import_chunk",
            ToImportEvent {
                songs: songs.lock().unwrap().clone(),
                albums: vec![],
                progress: 100,
                done: albums.lock().unwrap().clone().len() == 0,
                error: None,
            },
        );
    }

    // Send all albums

    // If more than 100 albums, also send them in chunks

    if (albums.lock().unwrap().clone().len() > 100) {
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
            println!("{:?}", progress);
            let _ = app_handle.emit_all(
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
    } else {
        let _ = app_handle.emit_all(
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
    cprintln!(
        "<bold><green>Imported {} songs and {} albums in {:.2} seconds</green></bold>",
        songs.lock().unwrap().len(),
        albums.lock().unwrap().len(),
        (Instant::now() - start).as_secs_f32()
    );

    ToImportEvent {
        songs: vec![],
        albums: vec![],
        progress: 100,
        done: true,
        error: None,
    }
}

fn process_new_album(song: &Song, app_config: &Config) -> Option<Album> {
    let mut artwork_src = String::new();
    let mut artwork_format = String::new();
    // Strip song from path
    let album_path = Path::new(&song.path).parent().unwrap().to_str().unwrap();
    let album_id = MD5::hash(
        format!("{} - {}", album_path, song.album.as_str())
            .to_lowercase()
            .as_bytes(),
    )
    .to_hex_lowercase();
    // println!("album: {} , {}", song.album, album_id);
    let result = look_for_art(&song.path, &song.file, &app_config);
    if let Ok(res) = result {
        if let Some(art) = res.clone() {
            // println!("Found existing artwork in folder for: {}", song.album);
            artwork_src = art.artwork_src;
            artwork_format = art.artwork_format;
        }
    } else if let Err(e) = result {
        println!("Error looking for artwork: {}", e);
    }

    if (artwork_src.is_empty()) {
        if let Some(art) = &song.artwork {
            // println!("Caching artwork for: {}", song.album);
            // Cache artwork using artwork_cacher
            let cached_art_path =
                artwork_cacher::cache_artwork(&art.data, &album_id, &art.format, app_config);
            if let Ok(p) = cached_art_path {
                artwork_src = p.to_str().unwrap().to_string();
                artwork_format = art.format.clone();
            } else {
                println!("Error caching artwork: {}", cached_art_path.unwrap_err());
            }
        }
    }

    // Convert src to Tauri's asset protocol and encode it as a URI
    if !artwork_src.is_empty() {
        artwork_src = convert_file_src(artwork_src);
    }

    return Some(Album {
        id: album_id,
        title: song.album.clone(),
        artist: song.artist.clone(),
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

pub fn convert_file_src(artwork_src: String) -> String {
    // If windows or android, use http://{protocol}.localhost/{path}
    if cfg!(target_os = "windows") || cfg!(target_os = "android") {
        format!("http://asset.localhost/{}", artwork_src)
    } else {
        format!("asset://localhost/{}", artwork_src)
    }
}

struct ProcessDirectoryResult {
    songs: Vec<Song>,
    albums: HashMap<String, Album>,
}

fn process_directory(
    directory_path: &Path,
    songs: &Arc<std::sync::Mutex<Vec<Song>>>,
    albums: &Arc<std::sync::Mutex<HashMap<String, Album>>>,
    recursive: bool,
    config: &Config,
) -> Option<ProcessDirectoryResult> {
    let subsongs: Arc<std::sync::Mutex<Vec<Song>>> = Arc::new(Mutex::new(Vec::new()));
    let subalbums: Arc<std::sync::Mutex<HashMap<String, Album>>> =
        Arc::new(Mutex::new(HashMap::new()));
    fs::read_dir(directory_path)
        .par_iter_mut()
        .for_each(|entries| {
            entries.by_ref().par_bridge().for_each(|entry| {
                if let Ok(entry) = entry {
                    let path = entry.path();

                    // println!("{:?}", entry.path());
                    if path.is_file() {
                        if let Some(song) = crate::metadata::extract_metadata(&path, true) {
                            if let Some(album) = process_new_album(&song, config) {
                                // println!("Album: {:?}", album);
                                let existing_album =
                                    albums.lock().unwrap().get_mut(&album.id).cloned();
                                let existing_album_subalbums =
                                    subalbums.lock().unwrap().get_mut(&album.id).cloned();
                                if let Some(existing_album) = existing_album {
                                    // Merge with existing album
                                    let mut merged_album = existing_album.clone();
                                    merged_album.tracks_ids.push(song.id.clone());
                                    subalbums
                                        .lock()
                                        .unwrap()
                                        .insert(album.id.clone(), merged_album.to_owned());
                                } else if let Some(existing_album_subalbums) =
                                    existing_album_subalbums
                                {
                                    // Merge with existing album
                                    let mut merged_album = existing_album_subalbums.clone();
                                    merged_album.tracks_ids.push(song.id.clone());
                                    subalbums
                                        .lock()
                                        .unwrap()
                                        .insert(album.id.clone(), merged_album.to_owned());
                                } else {
                                    subalbums.lock().unwrap().insert(album.id.clone(), album);
                                }
                            }
                            subsongs.lock().unwrap().push(song);
                        }
                    } else if path.is_dir() && recursive {
                        if let Some(sub_results) =
                            process_directory(&path, songs, albums, true, config)
                        {
                            if (!sub_results.songs.is_empty()) {
                                songs.lock().unwrap().extend(sub_results.songs);
                            }
                            if (!sub_results.albums.is_empty()) {
                                albums.lock().unwrap().extend(sub_results.albums);
                            }
                        }
                    }
                }
            });
        });

    return Some(ProcessDirectoryResult {
        songs: subsongs.lock().unwrap().to_vec(),
        albums: subalbums.lock().unwrap().to_owned(),
    });
}

pub fn extract_metadata(file_path: &Path, is_import: bool) -> Option<Song> {
    if let Some(extension) = file_path.extension() {
        if let Some(ext_str) = extension.to_str() {
            if ext_str.eq_ignore_ascii_case("mp3")
                || ext_str.eq_ignore_ascii_case("flac")
                || ext_str.eq_ignore_ascii_case("wav")
                || ext_str.eq_ignore_ascii_case("aiff")
                || ext_str.eq_ignore_ascii_case("ape")
                || ext_str.eq_ignore_ascii_case("ogg")
            {
                if let Ok(tagged_file) = read_from_path(&file_path) {
                    let id = MD5::hash(file_path.to_str().unwrap().as_bytes()).to_hex_lowercase();
                    let path = file_path.to_string_lossy().into_owned();
                    let file = file_path.file_name()?.to_string_lossy().into_owned();

                    let mut title = String::new();
                    let mut artist = String::new();
                    let mut album = String::new();
                    let mut year = 0;
                    let mut genre = Vec::new();
                    let mut composer = Vec::new();
                    let mut track_number = -1;
                    let mut duration = String::new();
                    let mut file_info = FileInfo::new();
                    let mut artwork = None;

                    if tagged_file.tags().is_empty() {
                        title = file.to_string();
                    }
                    // println!("bit depth {:?}", tagged_file.properties().bit_depth());
                    file_info = FileInfo {
                        duration: Some(tagged_file.properties().duration().as_secs_f64()),
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
                                TagType::Ape
                                | TagType::Mp4Ilst
                                | TagType::RiffInfo
                                | TagType::AiffText => None,
                                _ => todo!(),
                            }
                        } else {
                            match tagged_file.file_type() {
                                FileType::Flac | FileType::Wav | FileType::Vorbis => {
                                    Some("vorbis".to_string())
                                }
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
                            _ => None,
                        },
                    };

                    if duration.is_empty() {
                        duration = seconds_to_hms(tagged_file.properties().duration().as_secs());
                    }

                    // println!("Tag properties {:?}", file_info);
                    tagged_file.tags().iter().for_each(|tag| {
                        // println!("Tag type {:?}", tag.tag_type());
                        // println!("Tag items {:?}", tag.items());
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
                                .map(|c| c.value().to_owned().into_string().unwrap_or_default())
                                .clone()
                                .collect()
                        }
                        if track_number == -1 {
                            track_number = tag.track().unwrap_or(0) as i32;
                        }
                    });

                    if tagged_file.primary_tag().is_some() && !is_import {
                        if let Some(pic) = tagged_file.primary_tag().unwrap().pictures().first() {
                            artwork = Some(Artwork {
                                data: pic.data().to_vec(),
                                format: pic.mime_type().unwrap().to_string(),
                            })
                        }
                    }

                    let start = SystemTime::now();
                    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap().as_millis();

                    return Some(Song {
                        id,
                        path,
                        file,
                        title,
                        artist,
                        album,
                        year,
                        genre,
                        composer,
                        track_number,
                        duration,
                        file_info,
                        artwork,
                        // We default the origin country to "" to allow Dexie to return results when using orderBy,
                        // even if there are zero songs with a non-empty country
                        origin_country: Some(String::from("")),
                        date_added: if is_import {
                            Some(since_the_epoch)
                        } else {
                            None
                        },
                    });
                }
            }
        }
    }
    None
}

#[derive(Debug)]
pub enum MyCustomError {
    TagWritingError,
}

impl std::error::Error for MyCustomError {}

impl fmt::Display for MyCustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyCustomError::TagWritingError => write!(f, "Error writing tag to file!"),
        }
    }
}

fn map_id3v1_to_id3v2_4(key: &str) -> Option<&'static str> {
    match key {
        "title" => Some("TIT2"),
        "artist" => Some("TPE1"),
        "album" => Some("TALB"),
        "year" => Some("TDRC"),
        "comment" => Some("COMM"),
        "track" => Some("TRCK"),
        "genre" => Some("TCON"),
        _ => None,
    }
}

fn write_metadata_track(v: &WriteMetatadaEvent) -> Result<(), anyhow::Error> {
    // println!("got event-name with payload {:?}", event.payload());

    // Parse JSON
    // println!("v {:?}", v);
    if v.tag_type.is_some() {
        // We know which tag type this is, continue with writing...

        if !v.metadata.is_empty() {
            // println!("{:?}", v.metadata);
            let mut tag_type: Option<TagType> = None;
            let tag_type_evt = v.tag_type.as_deref().unwrap();
            match tag_type_evt {
                "vorbis" => tag_type = Some(TagType::VorbisComments),
                "ID3v1" => tag_type = Some(TagType::Id3v2),
                "ID3v2.2" => tag_type = Some(TagType::Id3v2),
                "ID3v2.3" => tag_type = Some(TagType::Id3v2),
                "ID3v2.4" => tag_type = Some(TagType::Id3v2),
                _ => println!("Unhandled tag type: {:?}", v.tag_type),
            }
            let tag_type_value = tag_type.unwrap();
            let probe = Probe::open(&v.file_path).unwrap().guess_file_type()?;
            // &probe.guess_file_type();
            let file_type = &probe.file_type();
            println!("fileType: {:?}", &file_type);
            let mut tag = read_from_path(&v.file_path).unwrap();
            let tag_file_type = tag.file_type();
            let mut to_write = lofty::tag::Tag::new(tag_type.unwrap());

            if (tag.primary_tag().is_some()) {
                tag.primary_tag()
                    .unwrap()
                    .pictures()
                    .iter()
                    .enumerate()
                    .for_each(|(idx, pic)| to_write.set_picture(idx, pic.clone()));
            }
            println!("tag fileType: {:?}", &tag_file_type);

            // let primary_tag = tag.primary_tag_mut().unwrap();
            for item in v.metadata.iter() {
                if tag_type.is_some() {
                    if item.id == "METADATA_BLOCK_PICTURE" {
                        // Ignore picture, set by artwork_file_to_set
                    } else {
                        let mut tag_key: String = item.id.clone();

                        if tag_type_evt == "ID3v1" {
                            println!("Upgrading v1 to v2.4 tag: {}", tag_key);
                            let item_keyv4 = map_id3v1_to_id3v2_4(&tag_key);
                            println!("Result v4: {:?}", item_keyv4);
                            if item_keyv4.is_some() {
                                tag_key = item_keyv4.unwrap().to_string();
                                println!("Upgraded ID3v1 tag to ID3v2.4: {}", tag_key);
                            }
                        } else if tag_type_evt == "ID3v2.2" {
                            println!("Upgrading v2.2 to v2.3 tag: {}", tag_key);
                            let item_keyv4 = upgrade_v2(tag_key.as_str());
                            println!("Result v4: {:?}", item_keyv4);
                            if item_keyv4.is_some() {
                                tag_key = item_keyv4.unwrap().to_string();
                                println!("Upgraded ID3v2.2 tag to ID3v2.4: {}", tag_key);
                            }
                        } else if tag_type_evt == "ID3v2.3" {
                            let item_keyv4 = upgrade_v3(tag_key.as_str());
                            println!("Result v4: {:?}", item_keyv4);
                            if item_keyv4.is_some() {
                                tag_key = item_keyv4.unwrap().to_string();
                                println!("Upgraded ID3v2.3 tag to ID3v2.4: {}", tag_key);
                            }
                        }
                        let mut item_key = ItemKey::from_key(tag_type_value, tag_key.deref());

                        if item.value.is_null() {
                            let mut exists = false;

                            for tag_item in tag.tags() {
                                for tg in tag_item.items() {
                                    if tg.key().eq(&item_key) {
                                        exists = true;
                                    }
                                }
                            }
                            if exists {
                                to_write.remove_key(&item_key);
                            }
                        } else {
                            let item_value: ItemValue =
                                ItemValue::Text(String::from(item.value.as_str().unwrap()));
                            if (tag_key.eq_ignore_ascii_case("TRCK")) {
                                item_key = ItemKey::TrackNumber;
                            }
                            to_write.insert(TagItem::new(item_key, item_value));
                        }
                    }
                }
            }

            // Set image if provided
            if !v.artwork_file_to_set.is_empty() {
                let picture_file = File::options()
                    .read(true)
                    .write(true)
                    .open(&Path::new(&v.artwork_file_to_set))?;

                let mut reader = BufReader::new(picture_file);
                let pic = Picture::from_reader(reader.get_mut());
                to_write.set_picture(0, pic.unwrap());
            }

            for tag_item in tag.tags() {
                for tag in tag_item.items() {
                    println!("{:?}", tag);
                }
            }
            let mut file = File::options().read(true).write(true).open(&v.file_path)?;
            println!("{:?}", file);
            println!("FILETYPE: {:?}", file_type);

            // Keep picture, overwrite everything else
            let pictures = to_write.pictures();

            tag.clear();
            tag.insert_tag(to_write);
            tag.save_to(&mut file, WriteOptions::new())?;
            println!("File saved succesfully!");
        }
    } else {
        println!("tagType is missing");
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
