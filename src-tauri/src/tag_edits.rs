//! Keeps library scans and tag edits from overwriting each other.
//!
//! A scan reads a file, then holds the result in memory until its chunk is
//! written to the database — on a big library that's minutes later, with the
//! frontend in between. If the user saves new tags in that window, the scan's
//! snapshot lands on top of the edit and the row goes back to the old values
//! while the file keeps the new ones.
//!
//! So every read of a file's tags is stamped with a generation, every saved
//! edit records a newer one, and whatever arrives at the database older than
//! the last edit of its track or album gets dropped: the edit's own write is
//! the current data. This lives in memory only, because a snapshot can't
//! outlive the process that took it, so there's nothing to persist or migrate.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError, RwLock};

const FILE_LOCK_COUNT: usize = 64;

pub struct TagEdits {
    generation: AtomicU64,
    /// Track/album id -> generation of the last edit that touched it.
    last_edits: Mutex<HashMap<String, u64>>,
    /// Striped by path, so unrelated files can share a lock. Lofty saves in
    /// place, so without these a read could see a half-written file, and it's
    /// what makes "stamp, then read" atomic with respect to a save. Held for
    /// one read or save, never a whole scan, so an edit only waits for reads
    /// in flight on its own stripe. Only `read_file` callers are covered.
    file_locks: Vec<RwLock<()>>,
}

impl TagEdits {
    pub fn new() -> Self {
        Self {
            generation: AtomicU64::new(0),
            last_edits: Mutex::new(HashMap::new()),
            file_locks: (0..FILE_LOCK_COUNT).map(|_| RwLock::new(())).collect(),
        }
    }

    fn file_lock(&self, path: &Path) -> &RwLock<()> {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        &self.file_locks[hasher.finish() as usize % FILE_LOCK_COUNT]
    }

    /// Runs `read` against the file, returning its result and the generation
    /// to stamp on whatever was read.
    pub fn read_file<T>(&self, path: &Path, read: impl FnOnce() -> T) -> (T, u64) {
        let _guard = self.file_lock(path).read().unwrap_or_else(PoisonError::into_inner);
        let generation = self.generation.load(Ordering::SeqCst);
        (read(), generation)
    }

    /// Runs `write` against the file and, if it succeeds, marks `edited_ids`
    /// as edited. Returns the edit's generation. A failed write marks nothing,
    /// so in-flight scan results for the file still get applied.
    pub fn write_file<E>(
        &self,
        path: &Path,
        edited_ids: &[&str],
        write: impl FnOnce() -> Result<(), E>,
    ) -> Result<u64, E> {
        let _guard = self.file_lock(path).write().unwrap_or_else(PoisonError::into_inner);
        write()?;
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        for id in edited_ids {
            self.mark_edited(id, generation);
        }
        Ok(generation)
    }

    pub fn mark_edited(&self, id: &str, generation: u64) {
        let mut last_edits = self.last_edits.lock().unwrap_or_else(PoisonError::into_inner);
        let last_edit = last_edits.entry(id.to_string()).or_insert(generation);
        *last_edit = generation.max(*last_edit);
    }

    /// True if `id` was edited after `read_generation` was stamped. `None`
    /// means the data didn't come from reading a file, so no edit can make
    /// it stale.
    pub fn is_superseded(&self, id: &str, read_generation: Option<u64>) -> bool {
        let Some(read_generation) = read_generation else { return false };
        let last_edits = self.last_edits.lock().unwrap_or_else(PoisonError::into_inner);
        last_edits.get(id).is_some_and(|last_edit| *last_edit > read_generation)
    }
}

pub fn tag_edits() -> &'static TagEdits {
    static TAG_EDITS: OnceLock<TagEdits> = OnceLock::new();
    TAG_EDITS.get_or_init(TagEdits::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "/music/a.mp3";

    fn read(edits: &TagEdits) -> u64 {
        edits.read_file(Path::new(FILE), || ()).1
    }

    fn write(edits: &TagEdits, ids: &[&str]) -> u64 {
        edits.write_file::<()>(Path::new(FILE), ids, || Ok(())).unwrap()
    }

    #[test]
    fn a_read_taken_before_an_edit_is_superseded() {
        let edits = TagEdits::new();
        let before = read(&edits);
        write(&edits, &["track"]);
        assert!(edits.is_superseded("track", Some(before)));
    }

    #[test]
    fn a_read_taken_after_an_edit_is_current() {
        let edits = TagEdits::new();
        write(&edits, &["track"]);
        assert!(!edits.is_superseded("track", Some(read(&edits))));
    }

    #[test]
    fn an_older_edit_is_superseded_by_a_newer_one() {
        let edits = TagEdits::new();
        write(&edits, &["track"]);
        let first_edit_read = read(&edits);
        write(&edits, &["track"]);
        assert!(edits.is_superseded("track", Some(first_edit_read)));
    }

    #[test]
    fn edits_only_supersede_the_ids_they_touched() {
        let edits = TagEdits::new();
        let before = read(&edits);
        write(&edits, &["track"]);
        assert!(!edits.is_superseded("other-track", Some(before)));
    }

    #[test]
    fn a_failed_write_supersedes_nothing() {
        let edits = TagEdits::new();
        let before = read(&edits);
        let result = edits.write_file(Path::new(FILE), &["track"], || Err("disk full"));
        assert_eq!(result, Err("disk full"));
        assert!(!edits.is_superseded("track", Some(before)));
    }

    #[test]
    fn data_that_was_not_read_from_a_file_is_never_superseded() {
        let edits = TagEdits::new();
        write(&edits, &["track"]);
        assert!(!edits.is_superseded("track", None));
    }

    /// The two orderings that matter when a read and a save race: the read
    /// finishes first and comes back stale, or it waits and comes back
    /// current. It must never see the new tags and still be called stale, or
    /// see the old ones and be called current.
    #[test]
    fn racing_reads_are_stamped_consistently_with_what_they_saw() {
        let edits = TagEdits::new();
        let file_contents = AtomicU64::new(0);

        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    for _ in 0..2_000 {
                        let (seen, stamp) = edits
                            .read_file(Path::new(FILE), || file_contents.load(Ordering::SeqCst));
                        let latest = edits.generation.load(Ordering::SeqCst);
                        assert_eq!(seen, stamp, "stamp must match the contents read");
                        assert!(stamp <= latest);
                    }
                });
            }
            scope.spawn(|| {
                for _ in 0..2_000 {
                    edits
                        .write_file::<()>(Path::new(FILE), &["track"], || {
                            file_contents.fetch_add(1, Ordering::SeqCst);
                            Ok(())
                        })
                        .unwrap();
                }
            });
        });
    }
}
