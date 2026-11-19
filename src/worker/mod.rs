//! IO off the UI thread (docs/zendo/io.md, ADR 0024): the writer of files, the tasks that read, run
//! programs or wait, and the memos of small facts of the disk. The UI thread only sends a request or
//! takes an answer.

mod clock;
mod memo;
mod serial;
mod task;
mod writer;

pub use clock::{Clock, LIMIT, READING_AFTER, reading};
pub use memo::Memo;
pub use serial::Serial;
pub use task::Task;
pub use writer::{Checked, Sync, Writer};

use std::sync::{Arc, LazyLock};
use tokio::sync::Notify;

static WAKE: LazyLock<Arc<Notify>> = LazyLock::new(|| Arc::new(Notify::new()));

/// Woken when a worker has an answer: the main loop takes the answers and draws.
pub fn wake() -> Arc<Notify> {
    WAKE.clone()
}
