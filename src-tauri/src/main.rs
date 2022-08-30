#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
  )]

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;

use lofty::{read_from_path, ItemKey, ItemValue, Picture, TagItem, TagType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
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
            match v.tag_type.as_deref().unwrap() {
                "vorbis" => tag_type = Some(TagType::VorbisComments),
                "ID3v2.2" => tag_type = Some(TagType::ID3v2),
                "ID3v2.3" => tag_type = Some(TagType::ID3v2),
                "ID3v2.4" => tag_type = Some(TagType::ID3v2),
                _ => println!("Unhandled tag type: {:?}", v.tag_type),
            }
            let tag_type_value = tag_type.unwrap();

            let mut tag = read_from_path(&v.file_path, false).unwrap();
            let primary_tag = tag.primary_tag_mut().unwrap();
            for item in v.metadata.iter() {
                if tag_type.is_some() {
                    if item.id == "METADATA_BLOCK_PICTURE" {
                        // let pictureObject = &item.value.as_object().unwrap();
                        // let pictureFormat = &pictureObject["format"].as_str();
                        // let mut tagFormatMimeType: MimeType = MimeType::Jpeg;
                        // match (pictureFormat) {
                        //     Some("image/jpeg") => {
                        //         tagFormatMimeType = MimeType::Jpeg;
                        //     }
                        //     _ => println!("Unknown file format for picture"),
                        // }
                        // println!("GOT PICTURE, format: {}", pictureFormat.unwrap());
                        // println!("GOT PICTURE, tagFormatMimeType: {:?}", &tagFormatMimeType);
                        // print_type_of(&pictureObject["data"]);
                        // let pictureVec = &pictureObject["data"].as_str().unwrap();
                        // let decoded = base64::decode(pictureVec);
                    } else {
                        let item_key = ItemKey::from_key(tag_type_value, &item.id.deref());
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

            tag.save_to(&mut file)?;
            println!("File saved succesfully!");
        }
    } else {
        println!("tagType is missing");
    }
    Ok(())
    // println("title:")
}
