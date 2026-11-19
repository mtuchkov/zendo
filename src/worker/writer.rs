//! The writer: one thread writes the files; the UI thread only hands it the content (IO-1). While a
//! write of a file runs, only the newest content of that file waits (IO-4); appends to a log wait
//! together, in order.
use std::collections::{HashMap, VecDeque};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// How a write reaches the disk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sync {
    /// User data: whole or not at all, and synced to the disk (FAIL-2, IO-3).
    User,
    /// A file of zendo itself: whole or not at all, without the sync (IO-3).
    Own,
    /// The end of a log of zendo.
    Append,
}

type Disk = Box<dyn Fn(&Path, &[u8], Sync) -> std::io::Result<()> + Send>;

/// The answer to a checked write of user data (UNDO-6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Checked {
    Written,
    /// The disk has another text than zendo saw last: this one, without the new lines at its end.
    /// Nothing was written.
    Changed(String),
    /// A new file was to be made, but a file has its name. Nothing was written.
    Taken,
    Failed(String),
    /// A newer checked write of the same file took its place before it ran.
    Superseded,
}

/// A checked write: its id, and the text that zendo saw last on the disk; None for a new file.
#[derive(Clone)]
struct Check {
    id: u64,
    expected: Option<String>,
}

#[derive(Default)]
struct Queue {
    /// The files that wait, in the order of their first request.
    order: VecDeque<PathBuf>,
    /// The newest content of each file that waits, and its check.
    content: HashMap<PathBuf, (Vec<u8>, Sync, Option<Check>)>,
    /// The answers to the checked writes.
    checked: Vec<(u64, Checked)>,
    /// The id of the next checked write.
    next: u64,
    /// A write runs now.
    busy: bool,
    /// The writer goes: write what waits, then end.
    stop: bool,
    failures: Vec<(PathBuf, String)>,
}

#[derive(Default)]
struct Shared {
    queue: Mutex<Queue>,
    wake: Condvar,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, Queue> {
        self.queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// The files of zendo go to the disk on a thread of their own (ADR 0024).
pub struct Writer {
    shared: Arc<Shared>,
}

impl Default for Writer {
    fn default() -> Self {
        Writer::new()
    }
}

impl Writer {
    /// A writer to the disk.
    pub fn new() -> Writer {
        Writer::with(disk)
    }

    /// A writer that hands each write to `disk`: for tests, a disk that waits or fails.
    pub fn with(disk: impl Fn(&Path, &[u8], Sync) -> std::io::Result<()> + Send + 'static) -> Writer {
        let shared = Arc::new(Shared::default());
        let worker = shared.clone();
        let disk: Disk = Box::new(disk);
        let _ = std::thread::Builder::new().name("zendo-writer".into()).spawn(move || run(&worker, &disk));
        Writer { shared }
    }

    /// Write `content` to `path`, whole. Returns at once (IO-1).
    pub fn write(&self, path: &Path, content: Vec<u8>, sync: Sync) {
        self.put(path, content, sync);
    }

    /// Add `content` to the end of `path`. Returns at once (IO-1).
    pub fn append(&self, path: &Path, content: Vec<u8>) {
        self.put(path, content, Sync::Append);
    }

    fn put(&self, path: &Path, content: Vec<u8>, sync: Sync) {
        self.put_checked(path, content, sync, None);
    }

    fn put_checked(&self, path: &Path, content: Vec<u8>, sync: Sync, check: Option<Check>) {
        let mut q = self.shared.lock();
        match q.content.get_mut(path) {
            Some((c, Sync::Append, None)) if sync == Sync::Append => c.extend(content),
            Some(waiting) => {
                let old = std::mem::replace(waiting, (content, sync, check));
                if let Some(c) = old.2 {
                    q.checked.push((c.id, Checked::Superseded));
                }
            }
            None => {
                q.order.push_back(path.to_path_buf());
                q.content.insert(path.to_path_buf(), (content, sync, check));
            }
        }
        self.shared.wake.notify_one();
    }

    /// Write user data to `path`, whole and synced, only when the disk has `expected` (without the
    /// new lines at its end), or no file when `expected` is None. Returns at once with the id of
    /// the write; `checked` gives its answer (UNDO-6, IO-1).
    pub fn write_checked(&mut self, path: &Path, content: Vec<u8>, expected: Option<String>) -> u64 {
        let id = {
            let mut q = self.shared.lock();
            q.next += 1;
            q.next
        };
        self.put_checked(path, content, Sync::User, Some(Check { id, expected }));
        id
    }

    /// The answers to the checked writes since the last call.
    pub fn checked(&mut self) -> Vec<(u64, Checked)> {
        std::mem::take(&mut self.shared.lock().checked)
    }

    /// Nothing waits and nothing runs.
    pub fn idle(&self) -> bool {
        let q = self.shared.lock();
        q.order.is_empty() && !q.busy
    }

    /// Wait at most `limit` for the writes that wait: at the end of zendo, after the last frame.
    pub fn flush(&self, limit: Duration) -> bool {
        let end = Instant::now() + limit;
        while !self.idle() {
            if Instant::now() > end {
                return false;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        true
    }

    /// The writes that failed since the last call, each once (FAIL-1).
    pub fn failures(&mut self) -> Vec<(PathBuf, String)> {
        std::mem::take(&mut self.shared.lock().failures)
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        self.shared.lock().stop = true;
        self.shared.wake.notify_one();
    }
}

/// Take the next file that waits, write it, report a failure; end when the writer goes.
fn run(shared: &Shared, disk: &Disk) {
    loop {
        let mut q = shared.lock();
        while q.order.is_empty() && !q.stop {
            q = shared.wake.wait(q).unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        let Some(path) = q.order.pop_front() else { return };
        let Some((content, sync, check)) = q.content.remove(&path) else { continue };
        q.busy = true;
        drop(q);
        let verdict = check.as_ref().and_then(|c| verdict(&path, c.expected.as_deref()));
        let done = if verdict.is_some() { Ok(()) } else { disk(&path, &content, sync) };
        let mut q = shared.lock();
        q.busy = false;
        match (check, verdict, done) {
            (Some(c), Some(v), _) => q.checked.push((c.id, v)),
            (Some(c), None, Ok(())) => q.checked.push((c.id, Checked::Written)),
            (Some(c), None, Err(e)) => q.checked.push((c.id, Checked::Failed(e.to_string()))),
            (None, _, Err(e)) => q.failures.push((path, e.to_string())),
            (None, _, Ok(())) => {}
        }
        drop(q);
        super::wake().notify_one();
    }
}

/// Whether a checked write must not go: the disk has another text than `expected`, or a file
/// where a new one was to be made. None: write.
fn verdict(path: &Path, expected: Option<&str>) -> Option<Checked> {
    let now = fs::read_to_string(path).ok();
    match (expected, now) {
        (None, Some(_)) => Some(Checked::Taken),
        (Some(e), Some(d)) if d.trim_end_matches('\n') != e => Some(Checked::Changed(d.trim_end_matches('\n').to_string())),
        _ => None,
    }
}

/// The real disk.
fn disk(path: &Path, content: &[u8], sync: Sync) -> std::io::Result<()> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir)?;
    }
    match sync {
        Sync::User => crate::atomic::write(path, content),
        Sync::Own => own(path, content),
        Sync::Append => OpenOptions::new().create(true).append(true).open(path)?.write_all(content),
    }
}

/// A file of zendo: a temporary file beside it and a rename, without the sync (IO-3).
fn own(path: &Path, content: &[u8]) -> std::io::Result<()> {
    static N: AtomicU64 = AtomicU64::new(0);
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let tmp = path.with_file_name(format!(".{name}.{}-{}.tmp", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let done = fs::write(&tmp, content).and_then(|()| fs::rename(&tmp, path));
    if done.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    done
}

#[cfg(test)]
#[path = "writer_tests.rs"]
mod tests;
