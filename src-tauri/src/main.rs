#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
use tauri::Manager;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use window_vibrancy::{apply_blur, apply_vibrancy, NSVisualEffectMaterial};


fn main() {
  tauri::Builder::default()
    .menu(build_menu())
    .setup(|app| {
      let window = app.get_window("main").unwrap();
      let window_reference = window.clone();
      window.on_menu_event(move |event| {
        window_reference.emit("menu", event.menu_item_id()).unwrap();
      });
      // #[cfg(target_os = "macos")]
      // apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow)
      //   .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

      // #[cfg(target_os = "windows")]
      // apply_blur(&window, Some((18, 18, 18, 125)))
      //   .expect("Unsupported platform! 'apply_blur' is only supported on Windows");

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");

  
}

fn build_menu() -> Menu {
  Menu::new()
    .add_submenu(Submenu::new(
      "Musicat",
      Menu::new()
        .add_native_item(MenuItem::Hide)
        .add_native_item(MenuItem::Quit),
    ))
    .add_submenu(Submenu::new(
      "File",
      Menu::new()
        .add_item(CustomMenuItem::new("import", "Import Library"))
    ))
    .add_submenu(Submenu::new(
      "DevTools",
      Menu::new()
        .add_item(CustomMenuItem::new("clear-db", "Clear DB"))
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
    ))
}