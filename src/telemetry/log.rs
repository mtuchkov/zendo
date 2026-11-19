//! The files of the telemetry log (ADR 0025): each process appends whole lines to files of its own,
//! `<UTC time of the start of the file>-<process id>-<n>.jsonl`, up to 1 MB each; at most 50 files
//! of all processes stay, and the oldest go first (TELEM-3, TELEM-4, TELEM-10).

use chrono::Utc;
use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Instant;

/// The largest file of the log, and the most that a failed write keeps (TELEM-3, TELEM-11).
pub(super) const MAX_FILE: u64 = 1 << 20;

/// The most files of the log, of all processes together (TELEM-3).
pub(super) const FILES: usize = 50;

/// The function that appends text to a file.
pub(super) type Append = Box<dyn FnMut(&Path, &str) -> io::Result<()> + Send>;

/// Append `text` to `file`; the folder comes when it is missing.
pub(super) fn append(file: &Path, text: &str) -> io::Result<()> {
    file.parent().map_or(Ok(()), fs::create_dir_all)?;
    OpenOptions::new().create(true).append(true).open(file)?.write_all(text.as_bytes())
}

/// The files of the log that one process writes.
pub(super) struct Roll {
    dir: PathBuf,
    pid: u32,
    /// The file of now and its size.
    file: Option<PathBuf>,
    size: u64,
    /// The number of the next file of this process.
    next: u32,
    /// The lines that are not on the disk yet, and their bytes.
    kept: VecDeque<String>,
    bytes: u64,
    append: Append,
}

impl Roll {
    pub(super) fn new(dir: PathBuf, pid: u32, append: Append) -> Roll {
        Roll { dir, pid, file: None, size: 0, next: 0, kept: VecDeque::new(), bytes: 0, append }
    }

    /// The bytes of the lines that wait for the next write.
    #[cfg(test)]
    pub(super) fn kept(&self) -> u64 {
        self.bytes
    }

    /// Append `lines` after the lines that an earlier write kept. True when all lines are on the
    /// disk; else the rest stays for the next write (TELEM-11).
    pub(super) fn write(&mut self, lines: Vec<String>) -> bool {
        self.bytes += lines.iter().map(|l| l.len() as u64 + 1).sum::<u64>();
        self.kept.extend(lines);
        while let Some(first) = self.kept.front() {
            let need = first.len() as u64 + 1;
            let file = match &self.file {
                Some(f) if self.size + need <= MAX_FILE => f.clone(),
                _ => self.roll(),
            };
            let (text, n) = self.fitting(MAX_FILE - self.size);
            if (self.append)(&file, &text).is_err() {
                self.trim();
                return false;
            }
            self.size += text.len() as u64;
            self.bytes -= text.len() as u64;
            self.kept.drain(..n);
        }
        true
    }

    /// After a failed write: at most 1 MB of lines stay, the newest (TELEM-11).
    fn trim(&mut self) {
        while self.bytes > MAX_FILE {
            let Some(old) = self.kept.pop_front() else { break };
            self.bytes -= old.len() as u64 + 1;
        }
    }

    /// The first kept lines that fit into `room` bytes, at least one, and their number.
    fn fitting(&self, room: u64) -> (String, usize) {
        let mut text = String::new();
        let mut n = 0;
        for l in &self.kept {
            if n > 0 && text.len() as u64 + l.len() as u64 + 1 > room {
                break;
            }
            text.push_str(l);
            text.push('\n');
            n += 1;
        }
        (text, n)
    }

    /// A new file: the oldest files of all processes go, so that at most 50 stay with it (TELEM-4).
    fn roll(&mut self) -> PathBuf {
        prune(&self.dir, FILES - 1);
        let name = format!("{}-{}-{}.jsonl", Utc::now().format("%Y%m%dT%H%M%S%.3fZ"), self.pid, self.next);
        let file = self.dir.join(name);
        (self.file, self.size, self.next) = (Some(file.clone()), 0, self.next + 1);
        file
    }
}

/// Delete the oldest files of the log in `dir` until at most `keep` stay. A file that another
/// process deleted first is not an error.
fn prune(dir: &Path, keep: usize) {
    let mut files: Vec<PathBuf> = fs::read_dir(dir).map(|d| d.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "jsonl")).collect()).unwrap_or_default();
    files.sort();
    let over = files.len().saturating_sub(keep);
    files.iter().take(over).for_each(|f| {
        let _ = fs::remove_file(f);
    });
}

/// The thread that writes the log, and the answers that it sends after each write: true when all
/// lines are on the disk.
pub(super) struct Writer {
    lines: mpsc::Sender<Vec<String>>,
    done: mpsc::Receiver<bool>,
    /// The writes that the thread has not answered yet.
    open: usize,
    /// The last answered write kept lines.
    kept: bool,
}

impl Writer {
    pub(super) fn spawn(mut roll: Roll) -> Writer {
        let (lines, rx) = mpsc::channel::<Vec<String>>();
        let (done_tx, done) = mpsc::channel();
        std::thread::spawn(move || {
            for batch in rx {
                // TELEM-11: a write that fails says nothing; its lines stay for the next write.
                let _ = done_tx.send(roll.write(batch));
            }
        });
        Writer { lines, done, open: 0, kept: false }
    }

    pub(super) fn send(&mut self, lines: Vec<String>) {
        if self.lines.send(lines).is_ok() {
            self.open += 1;
        }
    }

    /// Take the answers of the open writes, waiting for them at most until `until`.
    pub(super) fn settle(&mut self, until: Instant) {
        while self.open > 0 {
            let Ok(ok) = self.done.recv_timeout(until.saturating_duration_since(Instant::now())) else { break };
            (self.open, self.kept) = (self.open - 1, !ok);
        }
    }

    /// The last write failed and no other write is open: the kept lines must go again.
    pub(super) fn lost(&self) -> bool {
        self.open == 0 && self.kept
    }
}

#[cfg(test)]
#[path = "log_tests.rs"]
mod tests;
