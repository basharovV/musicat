use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use rusqlite::Connection;

type Job = Box<dyn FnOnce(&mut Connection) + Send>;

/// The single owner of the database's writable connection. All mutations go
/// through here, applied one at a time on a dedicated OS thread — see the
/// "Connection strategy" decision in
/// `openspec/changes/sqlite-storage-foundation/design.md`.
#[derive(Clone)]
pub struct WriterHandle {
    tx: Sender<Job>,
}

impl WriterHandle {
    pub fn spawn(mut conn: Connection) -> Self {
        let (tx, rx): (Sender<Job>, Receiver<Job>) = channel();
        thread::Builder::new()
            .name("db-writer".into())
            .spawn(move || {
                for job in rx {
                    job(&mut conn);
                }
            })
            .expect("failed to spawn db writer thread");
        Self { tx }
    }

    /// Runs `f` against the writer's connection and blocks until it
    /// completes. Every call site is expected to already be inside
    /// `spawn_blocking` (see `db::commands`), so blocking here never stalls
    /// the async executor. Multi-step writes should open their own
    /// `conn.transaction()` inside `f` so a crash mid-write can't leave
    /// partial state.
    pub fn execute<F, T, E>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&mut Connection) -> Result<T, E> + Send + 'static,
        T: Send + 'static,
        E: ToString,
    {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        let job: Job = Box::new(move |conn| {
            let result = f(conn).map_err(|e| e.to_string());
            let _ = reply_tx.send(result);
        });
        self.tx
            .send(job)
            .map_err(|_| "db writer thread is not running".to_string())?;
        reply_rx
            .recv()
            .map_err(|_| "db writer dropped the reply channel without a response".to_string())?
    }
}
