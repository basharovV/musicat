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
use lofty::{read_from_path, ItemKey, ItemValue, Picture, TagItem, TagType, Probe};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{CustomMenuItem, Menu, MenuItem, Runtime, Submenu, Window};
use tauri::{Event, Manager};
use window_vibrancy::{apply_blur, apply_vibrancy, NSVisualEffectMaterial};

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
            apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow)
                .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

            #[cfg(target_os = "windows")]
            apply_blur(&window, Some((18, 18, 18, 125)))
                .expect("Unsupported platform! 'apply_blur' is only supported on Windows");

            // Listen for metadata write event

            // listen to the `event-name` (emitted on any window)
            let _id = app.listen_global("write-metadata", move |event| {
                let write_result = write_metadata(event);
                match write_result {
                    Ok(()) => {
                        // Emit result back to client

                        app_.emit_all("write-success", {}).unwrap();
                    }
                    Err(err) => {
                        println!("{}", err)
                    }
                }
            });

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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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

#[derive(Serialize, Deserialize, Debug)]
struct MetadataEntry {
    id: String,
    value: Value,
}

// struct BlockPicture {
//   #[serde(with = "serde_bytes")]
//   data: Vec<u8>,
//   format: String
// }

#[derive(Serialize, Deserialize, Debug)]
struct WriteMetatadaEvent {
    metadata: Vec<MetadataEntry>,
    tag_type: Option<String>,
    file_path: String,
    artwork_file_to_set: String,
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

fn write_metadata(event: Event) -> Result<(), Box<dyn Error>> {
    // println!("got event-name with payload {:?}", event.payload());

    // Parse JSON
    let payload: &str = event.payload().unwrap();

    let v: WriteMetatadaEvent = serde_json::from_str(payload).unwrap();
    // println!("v {:?}", v);
    if v.tag_type.is_some() {
        // We know which tag type this is, continue with writing...

        if !v.metadata.is_empty() {
            // println!("{:?}", v.metadata);
            let mut tag_type: Option<TagType> = None;
            let mut tag_type_evt = v.tag_type.as_deref().unwrap();
            match tag_type_evt {
                "vorbis" => tag_type = Some(TagType::VorbisComments),
                "ID3v2.2" => tag_type = Some(TagType::ID3v2),
                "ID3v2.3" => tag_type = Some(TagType::ID3v2),
                "ID3v2.4" => tag_type = Some(TagType::ID3v2),
                _ => println!("Unhandled tag type: {:?}", v.tag_type),
            }
            let tag_type_value = tag_type.unwrap();
            let probe = Probe::open(&v.file_path).unwrap().guess_file_type()?;
            // &probe.guess_file_type();
            let fileType = &probe.file_type();
            println!("fileType: {:?}", &fileType);
            let mut tag = read_from_path(&v.file_path, true).unwrap();
            let tagFileType = tag.file_type();
            println!("tag fileType: {:?}", &tagFileType);

            let primary_tag = tag.primary_tag_mut().unwrap();
            for item in v.metadata.iter() {
                if tag_type.is_some() {
                    if item.id == "METADATA_BLOCK_PICTURE" {
                        // Ignore picture, set by artwork_file_to_set
                    } else {
                        let mut tag_key: String = item.id.clone();

                        if tag_type_evt == "ID3v2.2" {
                            println!("Upgrading to v3 tag: {}", tag_key);
                            let item_keyv3 = upgrade_v2(tag_key.as_str());
                            println!("Result v3: {:?}", item_keyv3);
                            if item_keyv3.is_some() {
                                tag_key = item_keyv3.unwrap().to_string();
                                let item_keyv4 = upgrade_v3(tag_key.as_str());
                                println!("Result v4: {:?}", item_keyv4);
                                if (item_keyv4.is_some()) {
                                    tag_key = item_keyv4.unwrap().to_string();
                                    println!("Upgraded ID3v2.2 tag to ID3v2.3: {}", tag_key);
                                } 
                            }
                        } else if tag_type_evt == "ID3v2.3" {
                            let item_keyv3 = upgrade_v2(tag_key.as_str());
                            if item_keyv3.is_some() {
                                tag_key = item_keyv3.unwrap().to_string();
                                println!("Upgraded ID3v2.3 tag to ID3v2.4: {}", tag_key);
                            }
                        }
                        let item_key = ItemKey::from_key(tag_type_value, tag_key.deref());
                        let item_value: ItemValue =
                            ItemValue::Text(String::from(item.value.as_str().unwrap()));
                        primary_tag.insert_item_unchecked(TagItem::new(item_key, item_value));
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
                primary_tag.set_picture(0, picture.unwrap());
            }

            // for tagItem in tag.items() {
            //     println!("{:?}", tagItem);
            // }
            let mut file = File::options().read(true).write(true).open(&v.file_path)?;
            println!("{:?}", file);
            println!("FILETYPE: {:?}", fileType);
            tag.save_to(&mut file)?;
            println!("File saved succesfully!");
        }
    } else {
        println!("tagType is missing");
    }
    Ok(())
    // println("title:")
}
