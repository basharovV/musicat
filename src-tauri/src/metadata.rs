use crate::{Artwork, FileInfo, Song};
use chksum_md5::MD5;
use lofty::{read_from_path, Accessor, AudioFile, FileType, ItemKey, TagType, TaggedFileExt};
use std::path::Path;

pub fn extract_metadata(file_path: &Path) -> Option<Song> {
    if let Some(extension) = file_path.extension() {
        if let Some(ext_str) = extension.to_str() {
            if ext_str.eq_ignore_ascii_case("mp3")
                || ext_str.eq_ignore_ascii_case("flac")
                || ext_str.eq_ignore_ascii_case("wav")
                || ext_str.eq_ignore_ascii_case("aiff")
                || ext_str.eq_ignore_ascii_case("ape")
                || ext_str.eq_ignore_ascii_case("ogg")
            {
                if let Ok(tagged_file) = read_from_path(&file_path) {
                    let id = MD5::hash(file_path.to_str().unwrap().as_bytes()).to_hex_lowercase();
                    let path = file_path.to_string_lossy().into_owned();
                    let file = file_path.file_name()?.to_string_lossy().into_owned();

                    let mut title = String::new();
                    let mut artist = String::new();
                    let mut album = String::new();
                    let mut year = 0;
                    let mut genre = Vec::new();
                    let mut composer = Vec::new();
                    let mut track_number = -1;
                    let mut duration = String::new();
                    let mut file_info = FileInfo::new();
                    let mut artwork = None;

                    if tagged_file.tags().is_empty() {
                        title = file.to_string();
                    }
                    // println!("bit depth {:?}", tagged_file.properties().bit_depth());
                    file_info = FileInfo {
                        duration: Some(tagged_file.properties().duration().as_secs_f64()),
                        channels: tagged_file.properties().channels(),
                        bit_depth: tagged_file.properties().bit_depth().or(Some(16)),
                        sample_rate: tagged_file.properties().sample_rate(),
                        audio_bitrate: tagged_file.properties().audio_bitrate(),
                        overall_bitrate: tagged_file.properties().overall_bitrate(),
                        lossless: vec![FileType::Flac, FileType::Wav]
                            .iter()
                            .any(|f| f.eq(&tagged_file.file_type())),
                        tag_type: if let Some(tag) = tagged_file.primary_tag() {
                            match tag.tag_type() {
                                TagType::VorbisComments => Some("vorbis".to_string()),
                                TagType::Id3v1 => Some("ID3v1".to_string()),
                                TagType::Id3v2 => Some("ID3v2".to_string()),
                                TagType::Ape
                                | TagType::Mp4Ilst
                                | TagType::RiffInfo
                                | TagType::AiffText => None,
                                _ => todo!(),
                            }
                        } else {
                            match tagged_file.file_type() {
                                FileType::Flac | FileType::Wav | FileType::Vorbis => {
                                    Some("vorbis".to_string())
                                }
                                FileType::Mpeg => Some("ID3v2".to_string()),
                                FileType::Ape | FileType::Opus | FileType::Speex => None,
                                _ => None,
                            }
                        },
                        codec: match tagged_file.file_type() {
                            FileType::Flac => Some("FLAC".to_string()),
                            FileType::Mpeg => Some("MPEG".to_string()),
                            FileType::Aiff => Some("AIFF".to_string()),
                            FileType::Wav => Some("WAV".to_string()),
                            FileType::Ape => Some("APE".to_string()),
                            FileType::Opus => Some("Opus".to_string()),
                            FileType::Speex => Some("Speex".to_string()),
                            FileType::Vorbis => Some("Vorbis".to_string()),
                            _ => None,
                        },
                    };

                    if duration.is_empty() {
                        duration = seconds_to_hms(tagged_file.properties().duration().as_secs());
                    }

                    // println!("Tag properties {:?}", file_info);
                    tagged_file.tags().iter().for_each(|tag| {
                        // println!("Tag type {:?}", tag.tag_type());
                        // println!("Tag items {:?}", tag.items());
                        if title.is_empty() {
                            title = tag
                                .title()
                                .filter(|x| !x.is_empty())
                                .unwrap_or(std::borrow::Cow::Borrowed(&file))
                                .to_string();
                        }
                        if artist.is_empty() {
                            artist = tag.artist().unwrap_or_default().to_string();
                        }
                        if album.is_empty() {
                            album = tag.album().unwrap_or_default().to_string();
                        }
                        if genre.is_empty() {
                            genre = tag.genre().map_or_else(Vec::new, |g| {
                                g.split('/').map(String::from).collect()
                            });
                        }
                        if year == 0 {
                            year = tag.year().unwrap_or(0) as i32;
                        }
                        if composer.is_empty() {
                            composer = tag
                                .get_items(&ItemKey::Composer)
                                .map(|c| c.value().to_owned().into_string().unwrap_or_default())
                                .clone()
                                .collect()
                        }
                        if track_number == -1 {
                            track_number = tag.track().unwrap_or(0) as i32;
                        }
                    });

                    if tagged_file.primary_tag().is_some() {
                        if let Some(pic) = tagged_file.primary_tag().unwrap().pictures().first() {
                            artwork = Some(Artwork {
                                data: pic.data().to_vec(),
                                format: pic.mime_type().to_string(),
                            })
                        }
                    }

                    return Some(Song {
                        id,
                        path,
                        file,
                        title,
                        artist,
                        album,
                        year,
                        genre,
                        composer,
                        track_number,
                        duration,
                        file_info,
                        artwork,
                        // We default the origin country to "" to allow Dexie to return results when using orderBy,
                        // even if there are zero songs with a non-empty country
                        origin_country: Some(String::from(""))
                    });
                }
            }
        }
    }
    None
}

fn seconds_to_hms(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    if hours == 0 {
        format!("{:02}:{:02}", minutes, seconds)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}
