use std::path::PathBuf;

use r2d2::ManageConnection;
use rusqlite::Connection;

/// A minimal `r2d2::ManageConnection` for read-only `rusqlite::Connection`s.
///
/// We hand-roll this instead of pulling in `r2d2_sqlite`: at the time this was
/// written, `r2d2_sqlite` 0.35 pins `rusqlite` to `^0.40`, whose matching
/// `libsqlite3-sys` 0.38.1 release has a build-script bug (`cfg_select!` isn't
/// stable on the toolchain this repo builds with). Staying on the
/// already-vendored `rusqlite` 0.38 (used today by `beets.rs`) avoids that
/// entirely; this manager is the only bit of glue r2d2 actually needs.
pub struct SqliteConnectionManager {
    db_path: PathBuf,
}

impl SqliteConnectionManager {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }
}

impl ManageConnection for SqliteConnectionManager {
    type Connection = Connection;
    type Error = rusqlite::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let conn = Connection::open_with_flags(
            &self.db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        // Readers still need to see the writer's WAL contents promptly and
        // tolerate a short wait rather than erroring under contention.
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        Ok(conn)
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.execute_batch("SELECT 1")
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

pub type Pool = r2d2::Pool<SqliteConnectionManager>;

pub fn build_pool(db_path: PathBuf) -> Result<Pool, r2d2::Error> {
    r2d2::Pool::builder()
        .max_size(4)
        .build(SqliteConnectionManager::new(db_path))
}
