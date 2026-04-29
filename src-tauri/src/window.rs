use std::sync::Mutex;

pub struct OpenedUrls(pub Mutex<Option<Vec<url::Url>>>);

#[derive(Clone, serde::Serialize)]
pub struct Payload {
    pub args: Vec<String>,
    pub cwd: String,
}

pub fn handle_decorations(window: &tauri::WebviewWindow, size: &tauri::PhysicalSize<u32>) {
    let width_scaled = (size.width as f64 / window.scale_factor().unwrap()).round() as u32;
    let height_scaled = (size.height as f64 / window.scale_factor().unwrap()).round() as u32;
    let is_decorated = window.is_decorated().unwrap();
    // Decorations off when width and height are 210px (mini-player mode)
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
