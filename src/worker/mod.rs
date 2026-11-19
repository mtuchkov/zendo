//! IO off the UI thread (docs/zendo/io.md, ADR 0024): the writer of files and the tasks that read,
//! run programs or wait. The UI thread only sends a request or takes an answer.

mod clock;
mod task;
mod writer;

pub use clock::{Clock, LIMIT};
pub use task::Task;
pub use writer::{Sync, Writer};
