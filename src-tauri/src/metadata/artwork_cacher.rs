use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use log::info;
use tauri::AppHandle;
use tauri::Manager;

use crate::store::load_settings;
use crate::store::UserSettings;

const CACHE_DIR: &str = if cfg!(debug_assertions) {
    "cache-dev"
} else {
    "cache"
};

#[derive(Debug)]
pub enum CacheError {
    CreateDirError(String),
    DeleteDirError(String),
    WriteFileError(String),
    AppDataDirError,
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::CreateDirError(err) => write!(f, "Error creating cache directory: {}", err),
            CacheError::DeleteDirError(err) => write!(f, "Error deleting cache directory: {}", err),
            CacheError::WriteFileError(err) => {
                write!(f, "Error writing image data to cache: {}", err)
            }
            CacheError::AppDataDirError => write!(f, "Error fetching app data directory"),
        }
    }
}

impl std::error::Error for CacheError {}

pub fn cache_artwork(
    image_data: &Vec<u8>,
    album_id: &str,
    format: &str,
    app: &AppHandle,
) -> Result<PathBuf, CacheError> {
    let data_dir = app_data_dir(app).or(Err(CacheError::AppDataDirError))?;

    let image_path = get_image_path(&data_dir, album_id, format)?;

    create_cache_directory(&data_dir)?;
    write_image_data_to_cache(&image_path, &image_data)?;

    Ok(image_path)
}

fn get_image_path(data_dir: &Path, image_name: &str, format: &str) -> Result<PathBuf, CacheError> {
    let extension = match format {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        _ => "",
    };

    let mut path = data_dir.to_path_buf();
    path.push(CACHE_DIR);
    path.push(format!("{}.{}", image_name, extension));
    Ok(path)
}

fn create_cache_directory(data_dir: &Path) -> Result<(), CacheError> {
    let mut path = data_dir.to_path_buf();
    path.push(CACHE_DIR);

    fs::create_dir_all(&path).map_err(|e| CacheError::CreateDirError(e.to_string()))?;
    Ok(())
}

#[allow(dead_code)]
fn delete_cache_directory(app: &AppHandle) -> Result<(), CacheError> {
    let data_dir = app_data_dir(app).or(Err(CacheError::AppDataDirError))?;

    let mut path = data_dir.to_path_buf();
    path.push(CACHE_DIR);

    if path.exists() {
        fs::remove_dir_all(&path).map_err(|e| CacheError::DeleteDirError(e.to_string()))?;
    }

    Ok(())
}

fn write_image_data_to_cache(image_path: &Path, image_data: &[u8]) -> Result<(), CacheError> {
    let mut file =
        fs::File::create(image_path).map_err(|e| CacheError::WriteFileError(e.to_string()))?;
    file.write_all(image_data)
        .map_err(|e| CacheError::WriteFileError(e.to_string()))?;
    Ok(())
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, tauri::Error> {
    return app.path().app_data_dir();
}

#[derive(Debug, Clone)]
pub struct LookForArtResult {
    pub artwork_src: String,
    pub artwork_format: String,
    #[allow(dead_code)]
    pub artwork_filename_match: String,
}

fn check_folder_artwork_by_filename(
    folder: &str,
    artwork_filename: &str,
) -> Result<Option<LookForArtResult>, anyhow::Error> {
    let src = format!("{}{}", folder, artwork_filename);
    let extension = Path::new(&src).extension().unwrap().to_str().unwrap();
    let mime_type = get_image_format(extension);
    info!("Looking for: {}", src);
    if Path::new(&src).exists() {
        return Ok(Some(LookForArtResult {
            artwork_src: src,
            artwork_format: mime_type.unwrap_or("image/jpeg").to_string(),
            artwork_filename_match: artwork_filename.to_string(),
        }));
    }
    Ok(None)
}

pub fn look_for_art(
    song_path: &str,
    song_file_name: &str,
    settings: &UserSettings,
    _app: &AppHandle,
) -> Result<Option<LookForArtResult>, anyhow::Error> {
    let folder = song_path.replace(song_file_name, "");
    let filenames_to_search = &settings.album_artwork_filenames;

    info!("Looking for artwork in: {}", folder);
    info!("Looking for filenames: {:?}", filenames_to_search);

    // Check if any of the filenames are in the folder
    for filename in filenames_to_search {
        if let Ok(cool) = check_folder_artwork_by_filename(&folder, &filename) {
            if let Some(artwork_result) = cool {
                return Ok(Some(artwork_result));
            }
        }
    }

    // Grab the first image in the folder
    let files = fs::read_dir(&folder)?;
    for file in files {
        let file = file?;
        let filename = file.file_name().into_string().unwrap();
        let extension = Path::new(&filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if is_image_file(&filename) {
            let format = get_image_format(extension);
            if let Some(format) = format {
                return Ok(Some(LookForArtResult {
                    artwork_src: file.path().to_str().unwrap().to_string(),
                    artwork_format: format.to_string(),
                    artwork_filename_match: filename,
                }));
            }
        }
    }

    Ok(None)
}

fn is_image_file(filename: &str) -> bool {
    let extensions = ["jpg", "jpeg", "png", "bmp", "gif"];
    if let Some(ext) = Path::new(filename).extension().and_then(|s| s.to_str()) {
        return extensions.contains(&ext);
    }
    false
}

fn get_image_format(extension: &str) -> Option<&'static str> {
    match extension {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "bmp" => Some("image/bmp"),
        "gif" => Some("image/gif"),
        _ => None,
    }
}
