#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use futures_util::StreamExt;
use metadata::FileInfo;
use player::file_streamer::AudioStreamer;
use reqwest;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::{io::Write, path::Path};
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, State};
use tempfile::Builder;
use tokio_util::sync::CancellationToken;
use window_vibrancy::{apply_blur, apply_vibrancy, NSVisualEffectMaterial};

mod metadata;
mod output;
mod player;
mod resampler;

#[cfg(test)]
mod tests;

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
async fn get_lyrics(event: GetLyricsEvent) -> GetLyricsResponse {
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
            Menu::new().add_item(
                CustomMenuItem::new("import", "Import folder").accelerator("CommandOrControl+O"),
            ),
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
                .add_native_item(MenuItem::SelectAll)
                .add_item(CustomMenuItem::new("find", "Find").accelerator("CommandOrControl+F")),
        ))
        .add_submenu(Submenu::new(
            "View",
            Menu::new()
                .add_item(CustomMenuItem::new("albums", "Go to Albums").accelerator("A"))
                .add_item(CustomMenuItem::new("library", "Go to Library").accelerator("L"))
                .add_item(CustomMenuItem::new("queue", "Toggle Queue").accelerator("Q"))
                .add_item(
                    CustomMenuItem::new("lyrics", "Toggle Lyrics")
                        .accelerator("CommandOrControl+L"),
                ),
        ));
    #[cfg(dev)]
    let newMenu = menu.add_submenu(Submenu::new(
        "DevTools",
        Menu::new()
            .add_item(CustomMenuItem::new("clear-db", "Clear DB"))
            .add_item(CustomMenuItem::new("open-cache", "Open cache directory"))
            .add_item(CustomMenuItem::new("open-config", "Open config directory")),
    ));
    #[cfg(dev)]
    return newMenu;

    return menu;
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
pub struct StreamFileRequest {
    path: Option<String>,
    seek: Option<f64>,
    file_info: Option<FileInfo>,
    volume: Option<f64>, // 0 to 1
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoopRegionRequest {
    enabled: Option<bool>,
    start_pos: Option<f64>,
    end_pos: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetWaveformRequest {
    path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct GetWaveformResponse {
    data: Option<Vec<f32>>,
}

#[tauri::command]
fn stream_file(
    event: StreamFileRequest,
    state: State<AudioStreamer>,
    _app_handle: tauri::AppHandle,
) {
    println!("Stream file {:?}", event);
    let _ = state
        .player_control_sender
        .send(player::file_streamer::PlayerControlEvent::StreamFile(event));
    state.resume();
}

#[tauri::command]
fn queue_next(
    event: StreamFileRequest,
    state: State<AudioStreamer>,
    _app_handle: tauri::AppHandle,
) {
    println!("Queue next file {:?}", event);
    // If we receive a null path - the queue will be cleared
    let _ = state.next_track_sender.send(event);
}

#[tauri::command]
fn loop_region(
    event: LoopRegionRequest,
    state: State<AudioStreamer>,
    _app_handle: tauri::AppHandle,
) {
    println!("Loop region{:?}", event);
    let _ = state
        .player_control_sender
        .send(player::file_streamer::PlayerControlEvent::LoopRegion(event));
}

#[tauri::command]
fn get_waveform(
    event: GetWaveformRequest,
    state: State<AudioStreamer>,
    _app_handle: tauri::AppHandle,
) -> () {
    println!("Get waveform {:?}", event);

    let evt = event.clone();
    let token = CancellationToken::new();
    let token_clone = token.clone();
    if let Ok(mut tokens) = state.cancel_tokens.try_lock() {
        // Cancel all existing waveform threads
        tokens
            .iter()
            .filter(|t| t.0 != &event.clone().path.unwrap())
            .for_each(|t| {
                t.1.cancel();
            });
        tokens.insert(event.path.unwrap(), token);
    }

    std::thread::spawn(move || {
        // Handle client's offer
        let result = player::file_streamer::get_peaks(evt, &_app_handle, token_clone);
        // println!("Waveform: {:?}", result);
    });
}

#[derive(Clone, Debug)]
pub struct SampleOffsetEvent {
    pub sample_offset: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VolumeControlEvent {
    volume: Option<f64>, // 0 to 1
}

#[tauri::command]
fn volume_control(event: VolumeControlEvent, state: State<AudioStreamer>) {
    println!("Received volume_control event");
    match state.volume_control_sender.send(event) {
        Ok(_) => {
            // println!("Sent control flow info");
        }
        Err(_err) => {
            println!("Error sending volume control info (channel inactive");
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlowControlEvent {
    client_bitrate: Option<f64>,
    decoding_active: Option<bool>,
}

#[tauri::command]
fn decode_control(event: FlowControlEvent, state: State<AudioStreamer>) {
    println!("Received decode control event: {:?}", event);
    match event.decoding_active {
        Some(true) => {
            state.resume();
        }
        Some(false) => {
            state.pause();
        }
        None => {}
    }
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

    let _ = state.init_webrtc(_app_handle).await;

    return Ok(StreamStatus { is_open: false });
}

#[tauri::command]
async fn download_file(
    url: String,
    path: String,
    _app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let client = Client::new();

    // Start the request
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let total_size = response
        .content_length()
        .ok_or("Failed to get content length")?;

    // Create a temporary file
    let mut temp_file = Builder::new()
        .prefix("download_")
        .tempfile()
        .map_err(|e| e.to_string())?;

    let mut downloaded = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        temp_file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        // Emit progress to the frontend
        let progress = (downloaded as f64 / total_size as f64) * 100.0;
        _app_handle
            .emit_all("download-progress", progress)
            .map_err(|e| e.to_string())?;
    }

    // Flush and sync the file
    temp_file.flush().map_err(|e| e.to_string())?;
    temp_file
        .as_file_mut()
        .sync_all()
        .map_err(|e| e.to_string())?;

    // Move the temporary file to the final destination
    let temp_path = temp_file.into_temp_path();
    let final_path = Path::new(&path);
    temp_path.persist(&final_path).map_err(|e| e.to_string())?;

    Ok(())
}

#[tokio::main]
async fn main() {
    // std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // #[cfg(dev)]
    // let devtools = devtools::init(); // initialize the plugin as early as possible

    let streamer = player::file_streamer::AudioStreamer::create().unwrap();

    // let mut builder = tauri::Builder::default();

    // #[cfg(dev)]
    // {
    //     builder = builder.plugin(devtools);
    // }

    tauri::Builder::default()
        .menu(build_menu())
        .manage(streamer)
        .setup(|app| {
            let app_ = app.handle();
            let app2_ = app.handle();
            let state: State<player::file_streamer::AudioStreamer<'static>> = app.state();
            state.init(app_);
            let strm1 = state.inner().to_owned();
            let strm2 = strm1.clone();

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
                let event_clone = event.clone();
                let app_clone = app2_.clone();

                let handle_clone = strm1.clone();
                tokio::spawn(async move {
                    // Handle client's offer
                    let answer = handle_clone.handle_signal(event_clone.payload()).await;
                    if let Some(ans) = answer {
                        let _ = app_clone.emit_all("webrtc-answer", ans.clone());
                    }
                });
            });

            let _id3 = app.listen_global("webrtc-icecandidate-server", move |event| {
                println!("webrtc-signal {:?}", event);
                let event_clone = event.clone();
                let handle_clone = strm2.clone();
                tokio::spawn(async move {
                    // Handle client's offer
                    let _answer = handle_clone
                        .clone()
                        .handle_ice_candidate(event_clone.payload())
                        .await;
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            metadata::write_metadata,
            metadata::write_metadatas,
            metadata::scan_paths,
            metadata::get_song_metadata,
            get_lyrics,
            get_file_size,
            stream_file,
            queue_next,
            init_streamer,
            decode_control,
            volume_control,
            get_waveform,
            loop_region,
            download_file
        ])
        .plugin(tauri_plugin_fs_watch::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
