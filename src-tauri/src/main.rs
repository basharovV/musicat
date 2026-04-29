#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use log::info;
#[cfg(target_os = "macos")]
use mediakeys::RemoteCommandCenter;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::{Emitter, Listener, LogicalPosition, Manager, State};

use crate::stem_separator::StemProcessState;
use crate::updater::PendingUpdate;
use crate::window::{handle_decorations, OpenedUrls, Payload};

mod artwork;
mod beets;
mod constants;
mod dsp;
mod equalizer;
mod files;
mod logger;
#[cfg(target_os = "macos")]
mod mediakeys;
mod metadata;
mod output;
mod player;
mod resampler;
mod scrape;
mod stem_separator;
mod store;
mod updater;
mod window;

#[cfg(test)]
mod tests;

fn main() {
    info!("Starting Musicat");

    let streamer = player::AudioPlayer::create().unwrap();

    // Workaround for https://github.com/tauri-apps/tauri/issues/5143
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    tauri::Builder::default()
        .manage(streamer)
        .manage(OpenedUrls(Default::default()))
        .manage(StemProcessState {
            processes: Mutex::new(HashMap::new()),
        })
        .manage(PendingUpdate(Mutex::new(None)))
        .setup(|app| {
            let app_ = app.handle();
            let app2_ = app_.clone();

            tauri::async_runtime::spawn(async move {
                updater::check_for_updates_on_startup(app2_).await;
            });

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
                    .theme(None)
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
                use tauri::window::{Color, EffectsBuilder};
                use tauri_utils::WindowEffect;

                window_builder = window_builder
                    .traffic_light_position(LogicalPosition { x: 16, y: 18 })
                    .title_bar_style(tauri::TitleBarStyle::Overlay)
                    .hidden_title(true)
                    .background_color(Color(0, 0, 0, 1)) // <-
                    .effects(
                        EffectsBuilder::new()
                            .effects(vec![WindowEffect::Sidebar])
                            .build(),
                    );
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
                    &MenuItemBuilder::with_id("check-for-updates", "Check for Updates…")
                        .build(app)?,
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
                        &MenuItemBuilder::with_id("reload", "Reload")
                            .accelerator("CommandOrControl+R")
                            .build(app)?,
                        &MenuItemBuilder::with_id("clear-data", "Clear data").build(app)?,
                        &MenuItemBuilder::with_id("import-db", "Import DB").build(app)?,
                        &MenuItemBuilder::with_id("export-db", "Export DB").build(app)?,
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
            metadata::scan_playlist,
            metadata::get_song_metadata,
            metadata::get_artwork_file,
            metadata::get_artwork_metadata,
            player::get_devices,
            scrape::get_lyrics,
            files::get_file_size,
            player::play_file,
            player::queue_next,
            player::decode_control,
            player::init_webrtc,
            player::handle_answer,
            player::volume_control,
            player::playback_speed_control,
            player::analyzer_control,
            player::equalizer_control,
            player::get_waveform,
            stem_separator::separate_stems,
            stem_separator::get_stems,
            stem_separator::get_all_stems,
            stem_separator::cancel_separation,
            player::loop_region,
            player::change_audio_device,
            files::download_file,
            scrape::get_wikipedia,
            files::delete_files,
            logger::max_log_level,
            logger::write_log,
            beets::search_beets,
            beets::search_beets_albums,
            beets::get_beets_album_tracks,
            beets::get_albums_by_id,
            updater::check_for_updates,
            updater::install_update
        ])
        .plugin(tauri_plugin_updater::Builder::new().build())
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
        .plugin(tauri_plugin_opener::init())
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
