//! Scans racing tag edits, against a real file and real lofty saves. The
//! generation logic has its own tests in `tag_edits`, and the database side
//! in `db::tests`; these check the two hold up together end to end.

use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;

use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use lofty::tag::Accessor;
use rusqlite::Connection;

use super::{write_metadata_track, MetadataEntry, Song, WriteMetatadaEvent};
use crate::db::{mutations, query, schema};
use crate::tag_edits::TagEdits;

const TRACK_ID: &str = "track";

fn open_test_db() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    schema::run_migrations(&mut conn).unwrap();
    conn
}

/// A tenth of a second of silence, the smallest file lofty can tag and save.
fn write_silent_wav(path: &Path) {
    let samples = vec![0u8; 8820];
    let mut wav = Vec::new();
    wav.extend(b"RIFF");
    wav.extend((36 + samples.len() as u32).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend(1u16.to_le_bytes()); // PCM
    wav.extend(1u16.to_le_bytes()); // mono
    wav.extend(44100u32.to_le_bytes());
    wav.extend(88200u32.to_le_bytes()); // bytes per second
    wav.extend(2u16.to_le_bytes()); // bytes per frame
    wav.extend(16u16.to_le_bytes()); // bits per sample
    wav.extend(b"data");
    wav.extend((samples.len() as u32).to_le_bytes());
    wav.extend(samples);
    std::fs::write(path, wav).unwrap();
}

fn save_title(path: &Path, title: &str) {
    write_metadata_track(&WriteMetatadaEvent {
        metadata: vec![MetadataEntry { id: "TrackTitle".to_string(), value: Some(Some(title.to_string())) }],
        tag_type: Some("id3v2".to_string()),
        file_path: path.to_str().unwrap().to_string(),
        artwork_file: String::new(),
        artwork_data: vec![],
        artwork_data_mime_type: None,
        delete_artwork: false,
    })
    .unwrap();
}

/// What `extract_metadata` does to a file, minus the `AppHandle` it needs.
fn read_track(edits: &TagEdits, path: &Path) -> Song {
    let (tagged_file, read_generation) = edits.read_file(path, || read_from_path(path));
    let tagged_file = tagged_file.unwrap();
    Song {
        id: TRACK_ID.to_string(),
        path: path.to_string_lossy().into_owned(),
        file: "track.wav".to_string(),
        title: tagged_file.first_tag().and_then(|tag| tag.title().map(|t| t.to_string())).unwrap_or_default(),
        read_generation: Some(read_generation),
        ..Default::default()
    }
}

fn title_in_db(conn: &Connection) -> String {
    query::get_tracks_by_ids(conn, &[TRACK_ID.to_string()]).unwrap().remove(0).unwrap().title
}

#[test]
fn a_scan_read_before_a_save_cannot_overwrite_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("track.wav");
    write_silent_wav(&path);
    save_title(&path, "Old");

    let edits = TagEdits::new();
    let mut conn = open_test_db();
    mutations::upsert_read_tracks(&mut conn, vec![read_track(&edits, &path)], &edits).unwrap();

    let scan_snapshot = read_track(&edits, &path);
    edits.write_file(&path, &[TRACK_ID], || Ok::<_, ()>(save_title(&path, "New"))).unwrap();
    let edited = read_track(&edits, &path);

    mutations::upsert_read_tracks(&mut conn, vec![edited], &edits).unwrap();
    mutations::upsert_read_tracks(&mut conn, vec![scan_snapshot], &edits).unwrap();

    assert_eq!(title_in_db(&conn), "New");
    assert_eq!(read_track(&edits, &path).title, "New");
}

/// Scan threads read the file while an editor keeps saving it, with titles of
/// varying length so the tag changes size. Every read has to parse as a whole
/// tag, and the database has to end up on what's on disk.
#[test]
fn parallel_reads_during_saves_stay_whole_and_end_on_the_disk_values() {
    const SCAN_THREADS: usize = 4;
    const READS_PER_THREAD: usize = 300;
    const SAVES: usize = 200;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("track.wav");
    write_silent_wav(&path);
    save_title(&path, "v0");

    let edits = TagEdits::new();
    let mut conn = open_test_db();
    mutations::upsert_read_tracks(&mut conn, vec![read_track(&edits, &path)], &edits).unwrap();

    let (reads_tx, reads_rx) = channel::<Song>();
    let mut commits = 0;

    thread::scope(|scope| {
        for _ in 0..SCAN_THREADS {
            let reads_tx = reads_tx.clone();
            let (edits, path) = (&edits, &path);
            scope.spawn(move || {
                for _ in 0..READS_PER_THREAD {
                    reads_tx.send(read_track(edits, path)).unwrap();
                }
            });
        }

        let (edits, path) = (&edits, &path);
        scope.spawn(move || {
            for save in 1..=SAVES {
                let title = format!("{}v{save}", "x".repeat(save % 40));
                edits.write_file(path, &[TRACK_ID], || Ok::<_, ()>(save_title(path, &title))).unwrap();
                reads_tx.send(read_track(edits, path)).unwrap();
            }
        });

        // One consumer, like the db writer thread
        for song in reads_rx {
            assert!(song.title.contains('v'), "torn read: {:?}", song.title);
            mutations::upsert_read_tracks(&mut conn, vec![song], &edits).unwrap();
            commits += 1;
        }
    });

    assert_eq!(commits, SCAN_THREADS * READS_PER_THREAD + SAVES);
    assert_eq!(title_in_db(&conn), read_track(&edits, &path).title);
}
