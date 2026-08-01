//! The play queue, moved out of `queue.txt` into an ordered table (tasks.md
//! 4.3) so it lives in the same durable store as everything else.

use rusqlite::Connection;

pub fn get_queue(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT track_id FROM queue ORDER BY position")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect()
}

/// Replaces the entire queue with `track_ids`, in order. Wrapped in a
/// transaction so a crash mid-write can't leave a partially-replaced queue.
pub fn set_queue(conn: &mut Connection, track_ids: &[String]) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM queue", [])?;
    {
        let mut stmt = tx.prepare("INSERT INTO queue (position, track_id) VALUES (?1, ?2)")?;
        for (position, track_id) in track_ids.iter().enumerate() {
            stmt.execute(rusqlite::params![position as i64, track_id])?;
        }
    }
    tx.commit()
}
