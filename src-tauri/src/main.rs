#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use futures_util::StreamExt;
use log::info;
#[cfg(target_os = "macos")]
use mediakeys::RemoteCommandCenter;
use metadata::FileInfo;
use player::AudioPlayer;
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
#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

mod constants;
mod dsp;
mod files;
#[cfg(target_os = "macos")]
mod mediakeys;
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
    boot: Option<bool>,
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
fn stream_file(event: StreamFileRequest, state: State<AudioPlayer>, _app_handle: tauri::AppHandle) {
    info!("Stream file {:?}", event);

    let boot = event.boot.clone();
    let waiting_for_boot = state
        .waiting_for_boot
        .load(std::sync::atomic::Ordering::Relaxed);

    // Developer experience niceness
    // Make sure this is an actual boot / app cold start,
    // not a hot reload / manual reload of the UI (playback continues)
    info!("Waiting for boot: {}", waiting_for_boot);
    if !(boot.is_some() && !waiting_for_boot) {
        info!("Sending stream file event");
        let _ = state
            .player_control_sender
            .send(player::PlayerControlEvent::StreamFile(event));
    }
    match boot {
        Some(true) => {
            if waiting_for_boot {
                #[cfg(target_os = "macos")]
                mediakeys::boot();

                state.pause();
                state
                    .waiting_for_boot
                    .swap(false, std::sync::atomic::Ordering::Relaxed);
            }
        }
        _ => {
            state.resume();
        }
    }
}

#[tauri::command]
fn queue_next(event: StreamFileRequest, state: State<AudioPlayer>, _app_handle: tauri::AppHandle) {
    info!("Queue next file {:?}", event);
    // If we receive a null path - the queue will be cleared
    let _ = state.next_track_sender.send(event);
}

#[tauri::command]
fn get_waveform(
    event: GetWaveformRequest,
    state: State<AudioPlayer>,
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
pub struct FlowControlEvent {
    client_bitrate: Option<f64>,
    decoding_active: Option<bool>,
}

#[tauri::command]
fn decode_control(event: FlowControlEvent, state: State<AudioPlayer>) {
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
    state: State<'_, AudioPlayer<'_>>,
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

    let temp_dev = get_device_id(temp_file.path());
    let temp_path = temp_file.into_temp_path();
    let final_path = Path::new(&path);
    let final_dev = get_device_id(&final_path);

    if temp_dev == final_dev {
        // Move the temporary file to the final destination
        temp_path.persist(&final_path).map_err(|e| e.to_string())?;
    } else {
        // Copy the temporary file to the final destination since files aren't on the same device
        fs::copy(temp_path, final_path).expect("Failed to copy file");
    }

    Ok(())
}

fn get_device_id(path: &Path) -> u64 {
    let parent_path = path.parent().expect("Failed to get parent directory");

    match file_id::get_file_id(parent_path).unwrap() {
        file_id::FileId::Inode { device_id, .. } => device_id,
        file_id::FileId::HighRes {
            volume_serial_number,
            ..
        } => volume_serial_number,
        file_id::FileId::LowRes {
            volume_serial_number,
            ..
        } => volume_serial_number.into(),
    }
}

struct OpenedUrls(Mutex<Option<Vec<url::Url>>>);

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

fn handle_decorations(window: &tauri::WebviewWindow, size: &tauri::PhysicalSize<u32>) {
    let width_scaled = (size.width as f64 / window.scale_factor().unwrap()).round() as u32;
    let height_scaled = (size.height as f64 / window.scale_factor().unwrap()).round() as u32;
    let is_decorated = window.is_decorated().unwrap();
    // Decorations off when width and height are 210px
    if width_scaled == 210 && height_scaled == 210 && is_decorated {
        window.set_decorations(false).unwrap();
        let _ = window.set_visible_on_all_workspaces(true);
        let _ = window.set_always_on_top(true);
    } else if width_scaled != 210 && height_scaled != 210 && !is_decorated {
        let _ = window.set_decorations(true).unwrap();
        let _ = window.set_always_on_top(false);
        let _ = window.set_visible_on_all_workspaces(false);
    }
}

#[tokio::main]
async fn main() {
    info!("Starting Musicat");
    // std::env::set_var("RUST_LOG", "debug");
    // env_logger::init();

    // #[cfg(dev)]
    // let devtools = devtools::init(); // initialize the plugin as early as possible

    let streamer = player::AudioPlayer::create().unwrap();

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
            let state: State<player::AudioPlayer<'static>> = app.state();

            env::set_var("MUSICAT_LOG_DIR", app.path().app_log_dir().unwrap());

            #[cfg(dev)]
            {
                let resource_path = app
                    .path()
                    .resolve(
                        "resources/log4rs.dev.yml",
                        tauri::path::BaseDirectory::Resource,
                    )
                    .expect("failed to resolve resource");
                log4rs::init_file(resource_path, Default::default()).unwrap();
            }

            #[cfg(not(dev))]
            {
                let resource_path = app
                    .path()
                    .resolve(
                        "resources/log4rs.release.yml",
                        tauri::path::BaseDirectory::Resource,
                    )
                    .expect("failed to resolve resource");
                log4rs::init_file(resource_path, Default::default()).unwrap();
            }

            info!("Goes to stderr and file");

            // File associations

            let opened_urls: State<OpenedUrls> = app.state();
            let file_urls = opened_urls.inner().to_owned();

            state.init(app_.clone());
            let strm1 = state.inner().to_owned();
            let strm2 = strm1.clone();
            let strm3 = strm1.clone();

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
                window_builder = window_builder.transparent(false);
            }

            #[cfg(target_os = "linux")]
            {
                window_builder = window_builder.transparent(false);
            }

            let window = window_builder.build().unwrap();

            let window2 = window.clone();
            #[cfg(target_os = "macos")]
            {
                apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
                    .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");
            }

            window.clone().on_window_event(move |event| match event {
                tauri::WindowEvent::Resized(size) => {
                    handle_decorations(&window, size);
                }
                _ => (),
            });

            app.on_menu_event(move |app, event| {
                app.emit("menu", event.id.0).unwrap();
            });

            app.listen_any("opened", move |_| {
                let inner_size = window2.inner_size().unwrap();
                handle_decorations(&window2, &inner_size);
            });

            // Listen for metadata write event
            // listen to the `event-name` (emitted on any window)

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

            let command_clone = strm3.clone();
            let app_next = app_.clone();
            let app_previous = app_.clone();
            let app_toggle = app_.clone();
            // Prepare to set Now Playing info on Mac
            #[cfg(target_os = "macos")]
            {
                let mut command_center = RemoteCommandCenter::new();

                // Define the handlers
                let next_handler = move || {
                    println!("Next command received - custom handling logic here");
                    // Add your custom next logic
                    let _ = app_next.emit("play_next", ());
                };

                let pause_handler = move || {
                    println!("Pause command received - custom handling logic here");
                    // Add your custom pause logic
                    command_clone.pause();
                };

                let play_handler = move || {
                    println!("Play command received - custom handling logic here");
                    // Add your custom play logic
                    strm3.resume();
                };

                let previous_handler = move || {
                    println!("Previous command received - custom handling logic here");
                    // Add your custom previous logic
                    let _ = app_previous.emit("play_previous", ());
                };

                let toggle_handler = move || {
                    println!("Toggle command received - custom handling logic here");
                    // Add your custom toggle logic
                    let _ = app_toggle.emit("toggle_play", ());
                };

                // Set the handlers
                command_center.set_handlers(
                    play_handler,
                    pause_handler,
                    toggle_handler,
                    previous_handler,
                    next_handler,
                );

                // Setup the remote command center
                command_center.setup_remote_command_center();
            }

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
                        .accelerator("CommandOrControl+L")
                        .build(app)?,
                ])
                .build()?;

            let library_submenu = SubmenuBuilder::new(app, "Library")
                .items(&[&MenuItemBuilder::with_id("prune", "Prune")
                    .accelerator("CommandOrControl+P")
                    .build(app)?])
                .build()?;

            let mut builder = MenuBuilder::new(app)
                .item(&app_submenu)
                .item(&file_submenu)
                .item(&edit_submenu)
                .item(&view_submenu)
                .item(&library_submenu);

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
            player::volume_control,
            player::playback_speed_control,
            get_waveform,
            player::loop_region,
            player::change_audio_device,
            download_file,
            scrape::get_wikipedia,
            files::delete_files
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
        .plugin(tauri_plugin_clipboard_manager::init())
        .build(tauri::generate_context!())
        .unwrap()
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            match event {
                tauri::RunEvent::Opened { urls, .. } => {
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
                _ => (),
            }
        });
}
