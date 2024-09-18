#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use futures_util::StreamExt;
use log::info;
use metadata::FileInfo;
use player::AudioStreamer;
use reqwest;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Mutex;
use std::{env, fs};
use std::{io::Write, path::Path};
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::{Emitter, Listener};
use tauri::{Manager, State};
use tempfile::Builder;
use tokio_util::sync::CancellationToken;
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

mod dsp;
mod metadata;
mod output;
mod player;
mod resampler;
mod scrape;
mod store;

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
    match lyrics {
        Ok(l) => return GetLyricsResponse { lyrics: Some(l) },
        Err(err) => {
            info!("Lyrics not found {:?}", err);
            return GetLyricsResponse { lyrics: None };
        }
    }
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
    // info!("{:?}", body);

    // Parse the HTML content using the scraper crate
    let document = Html::parse_document(&body);

    // Use a CSS selector to find the lyrics
    let lyrics_selector = Selector::parse("[data-lyrics-container=\"true\"]").unwrap();

    let lyrics_element = document.select(&lyrics_selector).next();
    // info!("{:?}", lyrics_element);

    // Extract and return the lyrics
    match lyrics_element {
        Some(element) => Ok(element.text().fold(String::new(), |s, l| s + l + "\n")),
        None => Err("Lyrics not found".into()),
    }
}

// fn build_menu(app: &AppHandle) -> Result<(), tauri::Error> {

// #[cfg(dev)]
// return newMenu;

// return menu;
// }

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
    output_device: Option<String>,
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
    info!("Stream file {:?}", event);
    let _ = state
        .player_control_sender
        .send(player::PlayerControlEvent::StreamFile(event));
    state.resume();
}

#[tauri::command]
fn queue_next(
    event: StreamFileRequest,
    state: State<AudioStreamer>,
    _app_handle: tauri::AppHandle,
) {
    info!("Queue next file {:?}", event);
    // If we receive a null path - the queue will be cleared
    let _ = state.next_track_sender.send(event);
}

#[tauri::command]
fn get_waveform(
    event: GetWaveformRequest,
    state: State<AudioStreamer>,
    _app_handle: tauri::AppHandle,
) -> () {
    info!("Get waveform {:?}", event);

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
        let _ = player::get_peaks(evt, &_app_handle, token_clone);
        // info!("Waveform: {:?}", result);
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
    info!("Received volume_control event");
    match state.volume_control_sender.send(event) {
        Ok(_) => {
            // info!("Sent control flow info");
        }
        Err(_err) => {
            info!("Error sending volume control info (channel inactive");
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
    info!("Received decode control event: {:?}", event);
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
    info!("Get stream status {:?}", event);
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
            .emit("download-progress", progress)
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

struct OpenedUrls(Mutex<Option<Vec<url::Url>>>);

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

#[tokio::main]
async fn main() {
    info!("Starting Musicat");
    // std::env::set_var("RUST_LOG", "debug");
    // env_logger::init();

    // #[cfg(dev)]
    // let devtools = devtools::init(); // initialize the plugin as early as possible

    let streamer = player::AudioStreamer::create().unwrap();

    // let mut builder = tauri::Builder::default().plugin(tauri_plugin_single_instance::init()).plugin(tauri_plugin_window::init());

    // #[cfg(dev)]
    // {
    //     builder = builder.plugin(devtools);
    // }

    tauri::Builder::default()
        .manage(streamer)
        .manage(OpenedUrls(Default::default()))
        .setup(|app| {
            let app_ = app.handle();
            let app2_ = app_.clone();
            let state: State<player::AudioStreamer<'static>> = app.state();

            let resource_path = app
                .path()
                .resolve("resources/log4rs.yml", tauri::path::BaseDirectory::Resource)
                .expect("failed to resolve resource");
            env::set_var("MUSICAT_LOG_DIR", app.path().app_log_dir().unwrap());
            log4rs::init_file(resource_path, Default::default()).unwrap();

            info!("Goes to stderr and file");

            // File associations

            let opened_urls: State<OpenedUrls> = app.state();
            let file_urls = opened_urls.inner().to_owned();

            state.init(app_.clone());
            let strm1 = state.inner().to_owned();
            let strm2 = strm1.clone();

            #[cfg(any(windows, target_os = "linux"))]
            {
                // NOTICE: `args` may include URL protocol (`your-app-protocol://`) or arguments (`--`) if app supports them.
                let mut urls = Vec::new();
                for arg in env::args().skip(1) {
                    if let Ok(url) = url::Url::parse(&arg) {
                        urls.push(url);
                    }
                }

                if !urls.is_empty() {
                    file_urls.0.lock().unwrap().replace(urls);
                }
            }

            let opened_urls = if let Some(urls) = &*file_urls.0.lock().unwrap() {
                urls.iter()
                    .map(|u| {
                        urlencoding::decode(u.as_str())
                            .unwrap()
                            .replace("\\", "\\\\")
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                "".into()
            };

            info!("Initial opened urls: {:?}", opened_urls);

            let mut window_builder =
                tauri::WebviewWindowBuilder::new(app, "main", Default::default())
                    .initialization_script(&format!("window.openedUrls = `{opened_urls}`"))
                    .initialization_script(&format!("console.log(`{opened_urls}`)"))
                    .theme(Some(tauri::Theme::Dark))
                    .fullscreen(false)
                    .inner_size(1200f64, 780f64)
                    .min_inner_size(210f64, 210f64)
                    .accept_first_mouse(true)
                    .visible(true)
                    .decorations(true)
                    .resizable(true)
                    .title("Musicat");

            #[cfg(target_os = "macos")]
            {
                window_builder = window_builder
                    .title_bar_style(tauri::TitleBarStyle::Overlay)
                    .hidden_title(true)
                    .transparent(true);
            }

            #[cfg(target_os = "windows")]
            {
                window_builder = window_builder.transparent(true);
            }

            #[cfg(target_os = "linux")]
            {
                window_builder = window_builder.transparent(false);
            }

            let window = window_builder.build().unwrap();

            app.on_menu_event(move |app, event| {
                app.emit("menu", event.id.0).unwrap();
            });

            #[cfg(target_os = "macos")]
            apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
                .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

            // Listen for metadata write event
            // listen to the `event-name` (emitted on any window)

            let window_clone = Box::new(window.clone());

            let _id2 = app.listen_any("show-toolbar", move |_| {
                window.set_decorations(true).unwrap();
            });

            let _id2 = app.listen_any("hide-toolbar", move |_| {
                window_clone.set_decorations(false).unwrap();
            });

            let _id3 = app.listen_any("webrtc-signal", move |event| {
                info!("webrtc-signal {:?}", event);
                let event_clone = event.clone();
                let app_clone = app2_.clone();

                let handle_clone = strm1.clone();
                tokio::spawn(async move {
                    // Handle client's offer
                    let answer = handle_clone.handle_signal(event_clone.payload()).await;
                    if let Some(ans) = answer {
                        let _ = app_clone.emit("webrtc-answer", ans.clone());
                    }
                });
            });

            let _id3 = app.listen_any("webrtc-icecandidate-server", move |event| {
                info!("webrtc-signal {:?}", event);
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
        .menu(|app| {
            let app_submenu = SubmenuBuilder::new(app, "Musicat")
                .items(&[
                    &MenuItemBuilder::with_id("about", "About Musicat").build(app)?,
                    &MenuItemBuilder::with_id("settings", "Settings")
                        .accelerator("CommandOrControl+,")
                        .build(app)?,
                    &PredefinedMenuItem::hide(app, Some("Hide"))?,
                    &PredefinedMenuItem::quit(app, Some("Quit"))?,
                ])
                .build()?;
            let file_submenu = SubmenuBuilder::new(app, "File")
                .items(&[&MenuItemBuilder::with_id("import", "Import folder")
                    .accelerator("CommandOrControl+O")
                    .build(app)?])
                .build()?;

            let edit_submenu = SubmenuBuilder::new(app, "Edit")
                .items(&[
                    &PredefinedMenuItem::copy(app, Some("Copy"))?,
                    &PredefinedMenuItem::cut(app, Some("Cut"))?,
                    &PredefinedMenuItem::paste(app, Some("Paste"))?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::undo(app, Some("Undo"))?,
                    &PredefinedMenuItem::redo(app, Some("Redo"))?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::select_all(app, Some("Select All"))?,
                    &MenuItemBuilder::with_id("find", "Find")
                        .accelerator("CommandOrControl+F")
                        .build(app)?,
                ])
                .build()?;

            let view_submenu = SubmenuBuilder::new(app, "View")
                .items(&[
                    &MenuItemBuilder::with_id("albums", "Albums")
                        .accelerator("Option+A")
                        .build(app)?,
                    &MenuItemBuilder::with_id("library", "Library")
                        .accelerator("Option+L")
                        .build(app)?,
                    &MenuItemBuilder::with_id("queue", "Queue")
                        .accelerator("Option+Q")
                        .build(app)?,
                    &MenuItemBuilder::with_id("lyrics", "Lyrics")
                        .accelerator("Option+L")
                        .build(app)?,
                ])
                .build()?;

            let mut builder = MenuBuilder::new(app)
                .item(&app_submenu)
                .item(&file_submenu)
                .item(&edit_submenu)
                .item(&view_submenu);

            if cfg!(dev) {
                let devtools_submenu = SubmenuBuilder::new(app, "DevTools")
                    .items(&[
                        &MenuItemBuilder::with_id("clear-db", "Clear DB").build(app)?,
                        &MenuItemBuilder::with_id("open-cache", "Open cache directory")
                            .build(app)?,
                        &MenuItemBuilder::with_id("open-config", "Open config directory")
                            .build(app)?,
                    ])
                    .build()?;
                builder = builder.item(&devtools_submenu);
            }

            let menu = builder.build()?;

            Ok(menu.to_owned())
        })
        .invoke_handler(tauri::generate_handler![
            metadata::write_metadatas,
            metadata::scan_paths,
            metadata::get_song_metadata,
            player::get_devices,
            get_lyrics,
            get_file_size,
            stream_file,
            queue_next,
            init_streamer,
            decode_control,
            volume_control,
            get_waveform,
            player::loop_region,
            player::change_audio_device,
            download_file,
            scrape::get_wikipedia
        ])
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            info!("{}, {argv:?}, {cwd}", app.package_info().name);
            app.emit("single-instance", Payload { args: argv, cwd })
                .unwrap();
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .build(tauri::generate_context!())
        .unwrap()
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = event {
                info!("Opened urls: {:?}", urls);
                if let Some(w) = app.get_webview_window("main") {
                    let urls = urls
                        .iter()
                        .map(|u| urlencoding::decode(u.as_str()).unwrap())
                        .collect::<Vec<_>>()
                        .join(",");
                    let _ = w.eval(&format!("window.onFileOpen(`{urls}`)"));
                }

                let opened_urls = app.try_state::<OpenedUrls>();
                if let Some(u) = opened_urls {
                    u.0.lock().unwrap().replace(urls);
                }
            }
        });
}
