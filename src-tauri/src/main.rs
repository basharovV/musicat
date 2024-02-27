#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use chksum_md5::MD5;
use lofty::id3::v2::{upgrade_v2, upgrade_v3};
use lofty::{
    read_from_path, Accessor, AudioFile, FileType, ItemKey, ItemValue, Picture, Probe, TagItem,
    TagType, TaggedFileExt,
};
use rayon::prelude::*;
use reqwest;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fs::{self, File};
use std::future::IntoFuture;
use std::io::BufReader;
use std::ops::{Deref, Mul};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{fmt, thread, time};
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, State};
use tokio::join;
use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;
use webrtc_streamer::web_rtcstreamer::{self, AudioStreamer};
use window_vibrancy::{apply_blur, apply_vibrancy, NSVisualEffectMaterial};
mod decoder;
mod file_chunk_streamer;
mod webrtc_streamer;
#[derive(Serialize, Deserialize, Clone, Debug)]
struct MetadataEntry {
    id: String,
    value: Value,
}

// struct BlockPicture {
//   #[serde(with = "serde_bytes")]
//   data: Vec<u8>,
//   format: String
// }

#[derive(Serialize, Deserialize, Clone, Debug)]
struct WriteMetatadaEvent {
    metadata: Vec<MetadataEntry>,
    tag_type: Option<String>,
    file_path: String,
    artwork_file_to_set: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct WriteMetatadasEvent {
    tracks: Vec<WriteMetatadaEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FixEncodingEvent {
    file_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GetLyricsEvent {
    url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GetLyricsResponse {
    lyrics: Option<String>,
}

#[tauri::command]
async fn get_lyrics(event: GetLyricsEvent, app_handle: tauri::AppHandle) -> GetLyricsResponse {
    let lyrics = fetch_lyrics(event.url.as_str()).await;
    return GetLyricsResponse {
        lyrics: lyrics.map_or(None, |l| Some(l)),
    };
}

async fn fetch_lyrics(genius_url: &str) -> Result<String, Box<dyn Error>> {
    // Make an HTTP GET request to the Genius URL
    let response = reqwest::get(genius_url).await?;

    // Check if the request was successful (status code 200)
    if !response.status().is_success() {
        return Err("Lyrics not found".into());
    }

    // Get the HTML content from the response
    let body = response.text().await?;
    // println!("{:?}", body);

    // Parse the HTML content using the scraper crate
    let document = Html::parse_document(&body);

    // Use a CSS selector to find the lyrics
    let lyrics_selector = Selector::parse("[data-lyrics-container=\"true\"]").unwrap();

    let lyrics_element = document.select(&lyrics_selector).next();
    // println!("{:?}", lyrics_element);

    // Extract and return the lyrics
    match lyrics_element {
        Some(element) => Ok(element.text().fold(String::new(), |s, l| s + l + "\n")),
        None => Err("Lyrics not found".into()),
    }
}

#[tauri::command]
async fn write_metadata(event: WriteMetatadaEvent, app_handle: tauri::AppHandle) -> ToImportEvent {
    println!("{:?}", event);

    // let payload: &str = event.payload().unwrap();
    let songs: Arc<std::sync::Mutex<Vec<Song>>> = Arc::new(Mutex::new(Vec::new()));

    // let v: WriteMetatadaEvent = serde_json::from_str(payload).unwrap();
    let write_result = write_metadata_track(&event);
    match write_result {
        Ok(()) => {
            let song = extract_metadata(Path::new(&event.file_path));
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
        progress: 100,
    };
    return to_import;
}

#[tauri::command]
async fn write_metadatas(
    event: WriteMetatadasEvent,
    _app_handle: tauri::AppHandle,
) -> ToImportEvent {
    println!("{:?}", event);
    // let payload: &str = event.payload().unwrap();
    let songs: Arc<std::sync::Mutex<Vec<Song>>> = Arc::new(Mutex::new(Vec::new()));

    // let v: WriteMetatadaEvent = serde_json::from_str(payload).unwrap();
    for track in event.tracks.iter() {
        let write_result = write_metadata_track(&track.clone());
        match write_result {
            Ok(()) => {
                // Emit result back to client
                let song = extract_metadata(Path::new(&track.file_path));
                if song.is_some() {
                    songs.lock().unwrap().push(song.unwrap());
                }
                println!("Wrote metadata")
            }
            Err(err) => {
                println!("{}", err)
            }
        }
    }
    // Emit result back to client

    let to_import = ToImportEvent {
        songs: songs.lock().unwrap().clone(),
        progress: 100,
    };
    return to_import;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ScanPathsEvent {
    paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct FileInfo {
    duration: Option<u64>, //s
    overall_bitrate: Option<u32>,
    audio_bitrate: Option<u32>,
    sample_rate: Option<u32>,
    bit_depth: Option<u8>,
    channels: Option<u8>,
    lossless: bool,
    tagType: Option<String>,
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
            tagType: None,
            codec: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Artwork {
    data: Vec<u8>,
    format: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Song {
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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ToImportEvent {
    songs: Vec<Song>,
    progress: u8,
}

#[tauri::command]
async fn scan_paths(event: ScanPathsEvent, app_handle: tauri::AppHandle) -> ToImportEvent {
    let songs: Arc<std::sync::Mutex<Vec<Song>>> = Arc::new(Mutex::new(Vec::new()));

    event.paths.par_iter().for_each(|p| {
        let path = Path::new(p.as_str());

        if path.is_file() {
            // println!("{:?}", entry.path());
            if let Some(song) = extract_metadata(&path) {
                songs.lock().unwrap().push(song);
            }
        } else if path.is_dir() {
            if let Some(sub_songs) = process_directory(Path::new(path), &songs) {
                songs.lock().unwrap().extend(sub_songs);
            }
        }
    });

    // Print the collected songs for demonstration purposes
    // for song in songs.lock().unwrap().clone(){
    //     println!("{:?}", song);
    // }

    let length = songs.lock().unwrap().clone().len();
    if length > 1000 {
        let songsClone = songs.lock().unwrap();
        let enumerator = songsClone.chunks(200);
        let chunks = enumerator.len();
        enumerator.into_iter().enumerate().for_each(|(idx, slice)| {
            thread::sleep(time::Duration::from_millis(1200));
            let progress = if (idx == chunks - 1) {
                100
            } else {
                u8::min(
                    ((slice.len() * (idx + 1)) as f64 / length as f64).mul(100.0) as u8,
                    100,
                )
            };
            println!("{:?}", progress);
            app_handle.emit_all(
                "import_chunk",
                ToImportEvent {
                    songs: slice.to_vec(),
                    progress: progress,
                },
            );
        });
        ToImportEvent {
            songs: Vec::new(),
            progress: 100,
        }
    } else {
        ToImportEvent {
            songs: songs.lock().unwrap().clone(),
            progress: 100,
        }
    }
}

fn process_directory(
    directory_path: &Path,
    songs: &Arc<std::sync::Mutex<Vec<Song>>>,
) -> Option<Vec<Song>> {
    let subsongs: Arc<std::sync::Mutex<Vec<Song>>> = Arc::new(Mutex::new(Vec::new()));

    fs::read_dir(directory_path)
        .par_iter_mut()
        .for_each(|entries| {
            entries.by_ref().par_bridge().for_each(|entry| {
                if let Ok(entry) = entry {
                    let path = entry.path();

                    // println!("{:?}", entry.path());
                    if path.is_file() {
                        if let Some(song) = extract_metadata(&path) {
                            subsongs.lock().unwrap().push(song);
                        }
                    } else if path.is_dir() {
                        if let Some(sub_songs) = process_directory(&path, songs) {
                            songs.lock().unwrap().extend(sub_songs);
                        }
                    }
                }
            });
        });

    return Some(subsongs.lock().unwrap().to_vec());
}

fn seconds_to_hms(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    if (hours == 0) {
        format!("{:02}:{:02}", minutes, seconds)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

fn extract_metadata(file_path: &Path) -> Option<Song> {
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

                    if (tagged_file.tags().is_empty()) {
                        title = file.to_string();
                    }
                    file_info = FileInfo {
                        duration: Some(tagged_file.properties().duration().as_secs()),
                        channels: tagged_file.properties().channels(),
                        bit_depth: tagged_file.properties().bit_depth().or(Some(16)),
                        sample_rate: tagged_file.properties().sample_rate(),
                        audio_bitrate: tagged_file.properties().audio_bitrate(),
                        overall_bitrate: tagged_file.properties().overall_bitrate(),
                        lossless: vec![FileType::Flac, FileType::Wav]
                            .iter()
                            .any(|f| f.eq(&tagged_file.file_type())),
                        tagType: if let Some(tag) = tagged_file.primary_tag() {
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

                    if (duration.is_empty()) {
                        duration = seconds_to_hms(tagged_file.properties().duration().as_secs());
                    }

                    // println!("Tag properties {:?}", file_info);
                    tagged_file.tags().iter().for_each(|tag| {
                        // println!("Tag type {:?}", tag.tag_type());
                        // println!("Tag items {:?}", tag.items());
                        if (title.is_empty()) {
                            title = tag
                                .title()
                                .filter(|x| !x.is_empty())
                                .unwrap_or(std::borrow::Cow::Borrowed(&file))
                                .to_string();
                        }
                        if (artist.is_empty()) {
                            artist = tag.artist().unwrap_or_default().to_string();
                        }
                        if (album.is_empty()) {
                            album = tag.album().unwrap_or_default().to_string();
                        }
                        if (genre.is_empty()) {
                            genre = tag.genre().map_or_else(Vec::new, |g| {
                                g.split('/').map(String::from).collect()
                            });
                        }
                        if (year == 0) {
                            year = tag.year().unwrap_or(0) as i32;
                        }
                        if (composer.is_empty()) {
                            composer = tag
                                .get_items(&ItemKey::Composer)
                                .map(|c| c.value().to_owned().into_string().unwrap_or_default())
                                .clone()
                                .collect()
                        }
                        if (track_number == -1) {
                            track_number = tag.track().unwrap_or(0) as i32;
                        }
                    });

                    if (tagged_file.primary_tag().is_some()) {
                        if let Some(pic) = tagged_file.primary_tag().unwrap().pictures().first() {
                            artwork = Some(Artwork {
                                data: pic.data().to_vec(),
                                format: pic.mime_type().to_string(),
                            })
                        }
                    }

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
                    });
                }
            }
        }
    }
    None
}

fn build_menu() -> Menu {
    let menu = Menu::new()
        .add_submenu(Submenu::new(
            "Musicat",
            Menu::new()
                .add_item(CustomMenuItem::new("about", "About Musicat"))
                .add_item(
                    CustomMenuItem::new("settings", "Settings").accelerator("CommandOrControl+,"),
                )
                .add_native_item(MenuItem::Hide)
                .add_native_item(MenuItem::Quit),
        ))
        .add_submenu(Submenu::new(
            "File",
            Menu::new().add_item(CustomMenuItem::new("import", "Import Library")),
        ))
        .add_submenu(Submenu::new(
            "Library",
            Menu::new()
                .add_item(CustomMenuItem::new("find", "Find").accelerator("CommandOrControl+F")),
        ))
        .add_submenu(Submenu::new(
            "Edit",
            Menu::new()
                .add_native_item(MenuItem::Copy)
                .add_native_item(MenuItem::Cut)
                .add_native_item(MenuItem::Paste)
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Undo)
                .add_native_item(MenuItem::Redo)
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::SelectAll),
        ));
    #[cfg(dev)]
    let newMenu = menu.add_submenu(Submenu::new(
        "DevTools",
        Menu::new().add_item(CustomMenuItem::new("clear-db", "Clear DB")),
    ));
    #[cfg(dev)]
    return newMenu;

    return menu;
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

fn write_metadata_track(v: &WriteMetatadaEvent) -> Result<(), Box<dyn Error>> {
    // println!("got event-name with payload {:?}", event.payload());

    // Parse JSON
    // println!("v {:?}", v);
    if v.tag_type.is_some() {
        // We know which tag type this is, continue with writing...

        if !v.metadata.is_empty() {
            // println!("{:?}", v.metadata);
            let mut tag_type: Option<TagType> = None;
            let mut tag_type_evt = v.tag_type.as_deref().unwrap();
            match tag_type_evt {
                "vorbis" => tag_type = Some(TagType::VorbisComments),
                "ID3v1" => tag_type = Some(TagType::Id3v2),
                "ID3v2.2" => tag_type = Some(TagType::Id3v2),
                "ID3v2.3" => tag_type = Some(TagType::Id3v2),
                "ID3v2.4" => tag_type = Some(TagType::Id3v2),
                _ => println!("Unhandled tag type: {:?}", v.tag_type),
            }
            let mut tag_type_value = tag_type.unwrap();
            let probe = Probe::open(&v.file_path).unwrap().guess_file_type()?;
            // &probe.guess_file_type();
            let file_type = &probe.file_type();
            println!("fileType: {:?}", &file_type);
            let mut tag = read_from_path(&v.file_path).unwrap();
            let tag_file_type = tag.file_type();
            let mut to_write = lofty::Tag::new(tag_type.unwrap());

            tag.primary_tag()
                .unwrap()
                .pictures()
                .iter()
                .enumerate()
                .for_each(|(idx, pic)| to_write.set_picture(idx, pic.clone()));

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
                            if (item_keyv4.is_some()) {
                                tag_key = item_keyv4.unwrap().to_string();
                                println!("Upgraded ID3v1 tag to ID3v2.4: {}", tag_key);
                            }
                        } else if tag_type_evt == "ID3v2.2" {
                            println!("Upgrading v2.2 to v2.3 tag: {}", tag_key);
                            let item_keyv4 = upgrade_v2(tag_key.as_str());
                            println!("Result v4: {:?}", item_keyv4);
                            if (item_keyv4.is_some()) {
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
                        let item_key = ItemKey::from_key(tag_type_value, tag_key.deref());

                        if (item.value.is_null()) {
                            let mut exists = false;

                            for tag_item in tag.tags() {
                                for tg in tag_item.items() {
                                    if tg.key().eq(&item_key) {
                                        exists = true;
                                    }
                                }
                            }
                            if (exists) {
                                to_write.remove_key(&item_key);
                            }
                        } else {
                            let item_value: ItemValue =
                                ItemValue::Text(String::from(item.value.as_str().unwrap()));
                            to_write.insert_unchecked(TagItem::new(item_key, item_value));
                        }
                    }
                }
            }

            // Set image if provided
            if !v.artwork_file_to_set.is_empty() {
                let picture_file = File::options()
                    .read(true)
                    .write(true)
                    .open(&Path::new(&v.artwork_file_to_set))
                    .unwrap();

                let mut reader = BufReader::new(picture_file);
                let picture = Picture::from_reader(reader.get_mut());
                to_write.set_picture(0, picture.unwrap());
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
            tag.save_to(&mut file)?;
            println!("File saved succesfully!");
        }
    } else {
        println!("tagType is missing");
    }
    Ok(())
    // println("title:")
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GetFileSizeRequest {
    path: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct GetFileSizeResponse {
    file_size: Option<u64>,
}

#[tauri::command]
fn get_file_size(event: GetFileSizeRequest) -> GetFileSizeResponse {
    if let Ok(metadata) = fs::metadata(event.path.unwrap()) {
        return GetFileSizeResponse {
            file_size: Some(metadata.len()),
        };
    }
    return GetFileSizeResponse { file_size: None };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StreamFileRequest {
    path: Option<String>,
    seek: Option<f64>
}

#[tauri::command]
fn stream_file(event: StreamFileRequest, state: State<AudioStreamer>) {
    println!("Stream file {:?}", event);
    state.start_data_channel(event.path.unwrap(), event.seek);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct StreamStatus {
    is_open: bool,
}

#[tauri::command]
async fn init_streamer(
    event: Option<String>,
    state: State<'_, AudioStreamer<'_>>,
    _app_handle: tauri::AppHandle,
) -> Result<StreamStatus, ()> {
    println!("Get stream status {:?}", event);

    // Close existing connection
    state.clone().reset().await;
    // Create new peer connection and datachannel
    state.clone().init(_app_handle).await;

    return Ok(StreamStatus { is_open: false });
}

#[tokio::main]
async fn main() {
    let streamer = web_rtcstreamer::AudioStreamer::create().await.unwrap();

    tauri::Builder::default()
        .menu(build_menu())
        .manage(streamer)
        .setup(|app| {
            let app_ = app.handle();
            let app2_ = app.handle();
            let app3_ = app.handle();
            let state: State<web_rtcstreamer::AudioStreamer<'static>> = app.state();

            // state.data_channel.on_close(Box::new(move || {
            //     Box::pin(async move {
            //         *is_open2.unwrap() = false;
            //     })
            // }));
            // state.data_channel.on_close(Box::new(move || {
            //     Box::pin(async move {
            //         let mut is_open = state.is_open.lock().await;
            //         *is_open = false;
            //     })
            // }));
            let strm1 = state.inner().to_owned();
            let strm2 = strm1.clone();
            let strm3 = strm1.clone();

            let window = app.get_window("main").unwrap();
            let window_reference = window.clone();
            window.on_menu_event(move |event| {
                window_reference.emit("menu", event.menu_item_id()).unwrap();
            });

            #[cfg(target_os = "macos")]
            apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
                .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

            #[cfg(target_os = "windows")]
            apply_blur(&window, Some((18, 18, 18, 125)))
                .expect("Unsupported platform! 'apply_blur' is only supported on Windows");

            // Listen for metadata write event
            // listen to the `event-name` (emitted on any window)

            let window_clone = Box::new(window.clone());

            let _id2 = app.listen_global("show-toolbar", move |_| {
                window.set_decorations(true).unwrap();
            });

            let _id2 = app.listen_global("hide-toolbar", move |_| {
                window_clone.set_decorations(false).unwrap();
            });

            let _id3 = app.listen_global("webrtc-signal", move |event| {
                println!("webrtc-signal {:?}", event);
                let eventClone = event.clone();
                let current = Handle::current();
                let app_clone = app_.clone();

                let handleClone = strm1.clone();
                tokio::spawn(async move {
                    // Handle client's offer
                    let answer = handleClone.handle_signal(eventClone.payload()).await;
                    if let Some(ans) = answer {
                        app_clone.emit_all("webrtc-answer", ans.clone());
                    }
                });
            });

            let _id3 = app.listen_global("webrtc-icecandidate-server", move |event| {
                println!("webrtc-signal {:?}", event);
                let eventClone = event.clone();
                let current = Handle::current();
                let app_clone = app2_.clone();

                let handleClone = strm2.clone();
                tokio::spawn(async move {
                    // Handle client's offer
                    let answer = handleClone
                        .clone()
                        .handle_ice_candidate(eventClone.payload())
                        .await;
                });
            });

            // tokio::spawn(async move {
            //     let handleClone = strm3.clone();
            //     // Listen for ICE candidates
            //     handleClone
            //         .peer_connection
            //         .on_ice_candidate(Box::new(move |c| {
            //             println!("on_ice_candidate {:?}", c);
            //             if let Some(cand) = c {
            //                 // let candidate = serde_json::to_string(&cand.to_json().unwrap());

            //                 app3_.emit_all("webrtc-icecandidate-client", &cand.to_json().unwrap());
            //             }
            //             Box::pin(async {})
            //         }));
            // });
            // #[cfg(target_os = "macos")]
            // apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow)
            //   .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

            // #[cfg(target_os = "windows")]
            // apply_blur(&window, Some((18, 18, 18, 125)))
            //   .expect("Unsupported platform! 'apply_blur' is only supported on Windows");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            write_metadata,
            write_metadatas,
            scan_paths,
            get_lyrics,
            get_file_size,
            stream_file,
            init_streamer
        ])
        .register_uri_scheme_protocol("stream", move |_app, request| {
            let boundary_id = Arc::new(Mutex::new(0));

            let result =
                file_chunk_streamer::file_chunk_handler::get_stream_response(request, &boundary_id);
            // println!("File chunk handler: response: {:?}", result);

            if let Some(e) = &result.as_ref().err() {
                println!("File chunk handler: Error: {:?}", e);
            }
            result
        })
        .plugin(tauri_plugin_fs_watch::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
