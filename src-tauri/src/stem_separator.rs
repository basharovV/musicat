use std::path;

use crate::{
    metadata::{extract_metadata, get_song_metadata},
    store::load_settings,
};
use log::info;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SeparateStemsEvent {
    path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SeparateStemsResponse {
    outputs: Vec<Stem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Stem {
    name: String,
    path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StemSeparationEvent {
    event: String,
    message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetStemsEvent {
    song_id: String,
}

#[tauri::command]
pub fn get_stems(event: GetStemsEvent, app_handle: tauri::AppHandle) -> Vec<Stem> {
    let settings = load_settings(&app_handle).expect("Failed to load settings");
    let stems_directory = settings
        .generated_stems_location
        .expect("Stems directory not set");

    let output_dir = std::path::PathBuf::from(format!("{}/{}/", stems_directory, event.song_id));

    if !output_dir.exists() {
        return vec![];
    }

    let mut stems: Vec<Stem> = vec![];
    for entry in std::fs::read_dir(output_dir).unwrap() {
        let entry = entry.unwrap();
        let stem_name = entry.file_name().to_str().unwrap().to_string();
        let stem_path = entry.path().to_string_lossy().into_owned();
        stems.push(Stem {
            name: stem_name,
            path: stem_path,
        });
    }
    stems
}

#[tauri::command]
pub async fn separate_stems(
    event: SeparateStemsEvent,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // Own the path
    let input_path = event.path.clone();
    let settings =
        load_settings(&app_handle).map_err(|e| format!("Failed to load settings: {}", e))?;
    let stems_directory = settings
        .generated_stems_location
        .ok_or("Stems directory not set")?;

    let input_file = std::path::PathBuf::from(&input_path);
    let song_result = extract_metadata(&input_file, false, false, false, &app_handle);

    let song = song_result.ok_or("Failed to extract metadata")?;
    let output_dir = std::path::PathBuf::from(format!("{}/{}/", stems_directory, song.id));

    // Make sure the output directory exists
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output dir: {}", e))?;

    let lib = app_handle.path().app_data_dir().unwrap().join("lib");
    if !lib.exists() {
        std::fs::create_dir_all(&lib).map_err(|e| format!("Failed to create lib dir: {}", e))?;
    }
    let models_dir = lib.join("models");
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir)
            .map_err(|e| format!("Failed to create models dir: {}", e))?;
    }

    let model_name = "UVR-MDX-NET-Voc_FT.onnx".to_string();
    let model = models_dir.join(&model_name);
    if !model.exists() {
        log::error!("Model does not exist: {}", model.display());
    }

    let onnxruntime_path = lib.join("libonnxruntime.dylib");
    if !onnxruntime_path.exists() {
        log::error!(
            "libonnxruntime path does not exist: {}",
            onnxruntime_path.display()
        );
    }

    // Configure options (LIB mode)
    // let opts = pvr::SeparateOptions {
    //     models_dir: Some(models_dir),
    //     model: Some(model_name),
    //     onnx_runtime_path: Some(onnxruntime_path),
    //     backends: vec![pvr::Backend::CoreML, pvr::Backend::CPU],
    //     output_format: pvr::AudioFormat::Flac,
    //     ..Default::default()
    // };

    // // Spawn blocking task so Tauri main thread isn't blocked
    // let result = tokio::task::spawn_blocking(move || {
    //     pvr::separate_to_files(&input_file, &output_dir, &opts)
    // })
    // .await
    // .map_err(|e| format!("Task join error: {}", e))? // handle JoinError
    // .map_err(|e| format!("Separation error: {}", e))?; // handle pvr error

    // let (primary, secondary) = result;
    // info!("Separated stems: {:?}, {:?}", primary, secondary);

    // Ok(format!("Stems saved to {}", primary.to_str().unwrap()))

    let sidecar_command = app_handle.shell().sidecar("pvr").unwrap().args([
        "--input-path",
        input_file.to_str().unwrap_or_default(),
        "--output-path",
        output_dir.to_str().unwrap_or_default(),
        "--models-dir",
        models_dir.to_str().unwrap_or_default(),
        "--model",
        model_name.as_str(),
        "--onnx-runtime-path",
        onnxruntime_path.to_str().unwrap_or_default(),
        "--format",
        "flac",
        "--coreml-backend",
    ]);

    let (mut rx, mut _child) = sidecar_command
        .spawn()
        .expect("Failed to spawn pvr sidecar");

    let window = app_handle.get_webview_window("main").unwrap();

    tauri::async_runtime::spawn(async move {
        // // read events such as stdout
        // while let Some(event) = rx.recv().await {
        //     if let CommandEvent::Stdout(line_bytes) = event {
        //         let line = String::from_utf8_lossy(&line_bytes);
        //         window
        //             .emit("message", Some(format!("'{}'", line)))
        //             .expect("failed to emit event");
        //         // write to stdin
        //         _child.write("message from Rust\n".as_bytes()).unwrap();
        //     }
        // }

        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    // Get everything after last 'INFO'
                    // eg. 2025-09-01T21:12:22.686000Z  INFO 82.14% Processing... (23/28)
                    // We want to get the "82.14% Processing... (23/28)"
                    let line = line.split("INFO").last().unwrap().trim();

                    info!("stdout: {}", line);

                    // If line contains "Processing", then emit progress event
                    if line.contains("Processing") {
                        window
                            .emit(
                                "stem-separation",
                                StemSeparationEvent {
                                    event: "progress".to_string(),
                                    message: line.to_string(),
                                },
                            )
                            .expect("failed to emit event");
                    } else if line.contains("Separation complete primary") {
                        window
                            .emit(
                                "stem-separation",
                                StemSeparationEvent {
                                    event: "progress".to_string(),
                                    message: line.to_string(),
                                },
                            )
                            .expect("failed to emit event");
                    }
                }
                CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    let line = line.split("ERROR").last().unwrap().trim();

                    log::error!("stderr: {}", line);
                    window
                        .emit(
                            "stem-separation",
                            StemSeparationEvent {
                                event: "error".to_string(),
                                message: line.to_string(),
                            },
                        )
                        .expect("failed to emit event");
                }
                CommandEvent::Error(line) => {
                    log::error!("error: {}", line);

                    window
                        .emit(
                            "stem-separation",
                            StemSeparationEvent {
                                event: "error".to_string(),
                                message: line.to_string(),
                            },
                        )
                        .expect("failed to emit event");
                }
                CommandEvent::Terminated(payload) => {
                    info!("terminated: {:?}", payload);
                    window
                        .emit(
                            "stem-separation",
                            StemSeparationEvent {
                                event: "complete".to_string(),
                                message: "Separation complete".to_string(),
                            },
                        )
                        .expect("failed to emit event");
                }
                _ => {}
            }
        }
    });

    Ok(())
}
