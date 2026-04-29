use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use tauri::{AppHandle, Emitter};
use tempfile::Builder;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFilesEvent {
    files: Vec<String>,
}

#[tauri::command]
pub fn delete_files(event: DeleteFilesEvent) {
    trash::delete_all(&event.files).unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetFileSizeRequest {
    path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFileSizeResponse {
    file_size: Option<u64>,
}

#[tauri::command]
pub fn get_file_size(event: GetFileSizeRequest) -> GetFileSizeResponse {
    if let Ok(metadata) = fs::metadata(event.path.unwrap()) {
        return GetFileSizeResponse {
            file_size: Some(metadata.len()),
        };
    }
    return GetFileSizeResponse { file_size: None };
}

#[tauri::command]
pub async fn download_file(
    url: String,
    path: String,
    _app_handle: AppHandle,
) -> Result<(), String> {
    let client = Client::new();

    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let total_size = response
        .content_length()
        .ok_or("Failed to get content length")?;

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

        let progress = (downloaded as f64 / total_size as f64) * 100.0;
        _app_handle
            .emit("download-progress", progress)
            .map_err(|e| e.to_string())?;
    }

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
        temp_path.persist(&final_path).map_err(|e| e.to_string())?;
    } else {
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
