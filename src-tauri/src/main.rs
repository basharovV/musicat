#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::ops::{Deref, DerefMut};
use std::{fmt, mem};

use lofty::id3::v2::{upgrade_v2, upgrade_v3};
use lofty::{read_from_path, ItemKey, ItemValue, Picture, Probe, TagItem, TagType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{CustomMenuItem, Menu, MenuItem, Runtime, Submenu, Window};
use tauri::{Event, Manager};
use window_vibrancy::{apply_blur, apply_vibrancy, NSVisualEffectMaterial};

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

#[tauri::command]
async fn write_metadata(event: WriteMetatadaEvent, app_handle: tauri::AppHandle) {
    println!("{:?}", event);

    // let payload: &str = event.payload().unwrap();

    // let v: WriteMetatadaEvent = serde_json::from_str(payload).unwrap();
    let write_result = write_metadata_track(event);
    match write_result {
        Ok(()) => {
            // Emit result back to client
            app_handle.emit_all("write-success", {}).unwrap();
        }
        Err(err) => {
            println!("{}", err)
        }
    }
}

#[tauri::command]
async fn write_metadatas(event: WriteMetatadasEvent, app_handle: tauri::AppHandle) {
    println!("{:?}", event);
    // let payload: &str = event.payload().unwrap();

    // let v: WriteMetatadaEvent = serde_json::from_str(payload).unwrap();
    for track in event.tracks.iter() {
        let write_result = write_metadata_track(track.clone());
        match write_result {
            Ok(()) => {
                // Emit result back to client
                println!("Wrote metadata")
            }
            Err(err) => {
                println!("{}", err)
            }
        }
    }

    // Emit result back to client
    app_handle.emit_all("write-success", {}).unwrap();
}


fn build_menu() -> Menu {
    Menu::new()
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
            "DevTools",
            Menu::new().add_item(CustomMenuItem::new("clear-db", "Clear DB")),
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
        ))
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

fn write_metadata_track(v: WriteMetatadaEvent) -> Result<(), Box<dyn Error>> {
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
                "ID3v1" => tag_type = Some(TagType::ID3v2),
                "ID3v2.2" => tag_type = Some(TagType::ID3v2),
                "ID3v2.3" => tag_type = Some(TagType::ID3v2),
                "ID3v2.4" => tag_type = Some(TagType::ID3v2),
                _ => println!("Unhandled tag type: {:?}", v.tag_type),
            }
            let mut tag_type_value = tag_type.unwrap();
            let probe = Probe::open(&v.file_path).unwrap().guess_file_type()?;
            // &probe.guess_file_type();
            let fileType = &probe.file_type();
            println!("fileType: {:?}", &fileType);
            let mut tag = read_from_path(&v.file_path, true).unwrap();
            let tagFileType = tag.file_type();
            let mut to_write = lofty::Tag::new(tag_type.unwrap());
            println!("tag fileType: {:?}", &tagFileType);

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

                            for tagItem in tag.tags() {
                                for tg in tagItem.items() {
                                    if tg.key().eq(&item_key)  {
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
                            to_write.insert_item_unchecked(TagItem::new(item_key, item_value));
                        }
                    }
                }
            }

            // Set image if provided
            if !v.artwork_file_to_set.is_empty() {
                let picture_file = File::options()
                    .read(true)
                    .write(true)
                    .open(v.artwork_file_to_set)
                    .unwrap();

                let mut reader = BufReader::new(picture_file);
                let picture = Picture::from_reader(reader.get_mut());
                to_write.set_picture(0, picture.unwrap());
            }

            for tagItem in tag.tags() {
                for tag in tagItem.items() {
                    println!("{:?}", tag);
                }
            }
            let mut file = File::options().read(true).write(true).open(&v.file_path)?;
            println!("{:?}", file);
            println!("FILETYPE: {:?}", fileType);

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

fn main() {
    tauri::Builder::default()
        .menu(build_menu())
        .setup(|app| {
            let app_ = app.handle();
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

            let windowClone = Box::new(window.clone());

            let _id2 = app.listen_global("show-toolbar", move |_| {
                window.set_decorations(true).unwrap();
            });

            let _id2 = app.listen_global("hide-toolbar", move |_| {
                windowClone.set_decorations(false).unwrap();
            });

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
            write_metadatas
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
