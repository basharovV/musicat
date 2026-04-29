use log::info;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_updater::UpdaterExt;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateStatus {
    status: String,
    version: Option<String>,
    notes: Option<String>,
    error: Option<String>,
}

pub struct PendingUpdate(pub Mutex<Option<tauri_plugin_updater::Update>>);

pub async fn check_for_updates_on_startup(app: AppHandle) {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            info!("Updater not available: {e}");
            return;
        }
    };
    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            let notes = update.body.clone();
            if let Some(pending) = app.try_state::<PendingUpdate>() {
                *pending.0.lock().unwrap() = Some(update);
            }
            let _ = app.emit(
                "update-status",
                UpdateStatus {
                    status: "available".into(),
                    version: Some(version),
                    notes,
                    error: None,
                },
            );
        }
        Ok(None) => {
            info!("App is up to date");
        }
        Err(e) => {
            info!("Update check failed: {e}");
        }
    }
}

#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    state: State<'_, PendingUpdate>,
) -> Result<(), String> {
    let _ = app.emit(
        "update-status",
        UpdateStatus {
            status: "checking".into(),
            version: None,
            notes: None,
            error: None,
        },
    );

    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            let notes = update.body.clone();
            *state.0.lock().unwrap() = Some(update);
            let _ = app.emit(
                "update-status",
                UpdateStatus {
                    status: "available".into(),
                    version: Some(version),
                    notes,
                    error: None,
                },
            );
        }
        Ok(None) => {
            let _ = app.emit(
                "update-status",
                UpdateStatus {
                    status: "up-to-date".into(),
                    version: None,
                    notes: None,
                    error: None,
                },
            );
        }
        Err(e) => {
            let _ = app.emit(
                "update-status",
                UpdateStatus {
                    status: "error".into(),
                    version: None,
                    notes: None,
                    error: Some(e.to_string()),
                },
            );
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    state: State<'_, PendingUpdate>,
) -> Result<(), String> {
    let update = state.0.lock().unwrap().take();

    if let Some(update) = update {
        let app_handle = app.clone();
        let app_handle2 = app.clone();
        let mut downloaded: u64 = 0;
        let mut total_size: u64 = 0;

        update
            .download_and_install(
                move |chunk_length, content_length| {
                    if let Some(t) = content_length {
                        total_size = t;
                    }
                    downloaded += chunk_length as u64;
                    if total_size > 0 {
                        let percent = (downloaded as f64 / total_size as f64) * 100.0;
                        let _ = app_handle.emit("update-progress", percent);
                    }
                },
                move || {
                    let _ = app_handle2.emit(
                        "update-status",
                        UpdateStatus {
                            status: "installing".into(),
                            version: None,
                            notes: None,
                            error: None,
                        },
                    );
                },
            )
            .await
            .map_err(|e| e.to_string())?;

        app.restart();
    }

    Ok(())
}
