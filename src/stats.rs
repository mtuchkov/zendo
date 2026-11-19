//! What the files of the stats of one zendo process share: the usage counts (USAGE-4) and the times
//! of the actions (RESP-3). `stats = false` in the config turns both off. Each file has the same
//! name, the same head and the same write. The stats stay in memory; a thread of each file writes
//! it, so that keys and the draw never wait for the disk (USAGE-15, RESP-4).

use chrono::{DateTime, SecondsFormat, Utc};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// The time between two writes of a file (USAGE-3).
pub const EVERY: Duration = Duration::from_secs(15);

/// How long the exit waits for the last write.
pub(crate) const LAST_WRITE: Duration = Duration::from_secs(1);

/// The function that writes the text to the file.
pub(crate) type Write = Box<dyn FnMut(&Path, &str) -> std::io::Result<()> + Send>;

/// The writes of one file: the thread that writes it, and when the stats changed and went last.
pub(crate) struct Flush {
    texts: mpsc::Sender<String>,
    /// The answer of the thread after each write: true when the write worked.
    done: mpsc::Receiver<bool>,
    /// The writes that the thread has not answered yet.
    open: usize,
    /// The last answered write failed.
    lost: bool,
    /// The stats changed since the last write.
    changed: bool,
    /// The last write, or the last try.
    written: Instant,
}

impl Flush {
    pub(crate) fn spawn(file: PathBuf, mut write: Write) -> Flush {
        let (texts, rx) = mpsc::channel::<String>();
        let (done_tx, done) = mpsc::channel();
        std::thread::spawn(move || {
            for text in rx {
                // USAGE-5, RESP-7: a write that fails says nothing; the next write has all the stats.
                let _ = done_tx.send(write(&file, &text).is_ok());
            }
        });
        Flush { texts, done, open: 0, lost: false, changed: false, written: Instant::now() }
    }

    pub(crate) fn change(&mut self) {
        self.changed = true;
    }

    /// The stats changed, or the last write failed, and the last write is 15 seconds old (USAGE-3).
    pub(crate) fn due(&mut self, now: Instant) -> bool {
        self.pending(Instant::now()) && now.saturating_duration_since(self.written) >= EVERY
    }

    /// Give the text of the stats to the thread.
    pub(crate) fn send(&mut self, now: Instant, text: String) {
        (self.written, self.changed) = (now, false);
        if self.texts.send(text).is_ok() {
            self.open += 1;
        }
    }

    /// Take the answers of the open writes, waiting for them at most until `until`. True when the
    /// stats must go again: they changed, or the last write failed and no other write is open.
    pub(crate) fn pending(&mut self, until: Instant) -> bool {
        while self.open > 0 {
            let Ok(ok) = self.done.recv_timeout(until.saturating_duration_since(Instant::now())) else { break };
            (self.open, self.lost) = (self.open - 1, !ok);
        }
        self.changed |= self.open == 0 && std::mem::take(&mut self.lost);
        self.changed
    }
}

/// `<dir>/<folder>/<date of the start>-<pid>.toml`: the file of process `pid` (USAGE-4, RESP-3).
pub(crate) fn file(dir: &Path, folder: &str, pid: u32) -> PathBuf {
    dir.join(folder).join(format!("{}-{pid}.toml", chrono::Local::now().format("%Y-%m-%d")))
}

/// The head of a file: the version, the system, the start and the write (USAGE-11, RESP-6).
pub(crate) fn head(started: DateTime<Utc>, written: DateTime<Utc>) -> String {
    let time = |t: DateTime<Utc>| t.to_rfc3339_opts(SecondsFormat::Secs, true);
    format!("zendo = \"{}\"\nos = \"{}\"\nstarted = \"{}\"\nwritten = \"{}\"\n", env!("CARGO_PKG_VERSION"), std::env::consts::OS, time(started), time(written))
}

/// Replace the file whole, through a temporary file and a rename (USAGE-13); the folder comes when
/// it is missing, and the temporary file goes when the write fails. These files are not user data:
/// no `fsync`, so that a write at the exit or on the UI thread stays short.
pub(crate) fn write(file: &Path, text: &str) -> std::io::Result<()> {
    let tmp = file.with_extension("toml.tmp");
    let done = file.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|()| std::fs::write(&tmp, text)).and_then(|()| std::fs::rename(&tmp, file));
    if done.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    done
}

#[cfg(test)]
#[path = "stats_tests.rs"]
mod tests;
