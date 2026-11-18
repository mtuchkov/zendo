//! One sync of the notes repo at a time, across the zendo processes of this user (NOTESREP-15).
//!
//! The lock is `flock` on `~/.zendo/notes-sync.lock`. The system lets it go when its process ends,
//! also after a kill, so a lock never stays behind. The file holds the pid and the folder of the
//! holder only for people to read: `zendo doctor` shows it.
use super::*;
use std::fs::{File, TryLockError};
use std::io::Write;

/// After this time without a change, a `.git/index.lock` is not git at work.
const STALE_INDEX_LOCK: Duration = Duration::from_secs(60);

/// The sync lock of this user.
pub fn lock_path() -> PathBuf {
    crate::setup::config::config_dir().join("notes-sync.lock")
}

/// The sync lock while it is held; dropping it lets it go. A lock without a file (zendo could not
/// make the file) does not stop the sync.
pub struct SyncLock {
    _file: Option<File>,
}

impl SyncLock {
    /// The lock, or None when another process holds it. `who` goes into the file after the pid.
    pub fn try_take(path: &Path, who: &str) -> Option<Self> {
        let Some(mut file) = open(path) else {
            return Some(Self { _file: None });
        };
        match file.try_lock() {
            Ok(()) => {
                let _ = file.set_len(0).and_then(|_| writeln!(file, "pid {} · {who}", std::process::id()));
                Some(Self { _file: Some(file) })
            }
            Err(TryLockError::WouldBlock) => None,
            Err(TryLockError::Error(_)) => Some(Self { _file: None }),
        }
    }

    /// The lock, if it frees up within `wait`.
    pub fn take_within(path: &Path, who: &str, wait: Duration) -> Option<Self> {
        let end = Instant::now() + wait;
        loop {
            if let Some(lock) = Self::try_take(path, who) {
                return Some(lock);
            }
            if Instant::now() >= end {
                return None;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

fn open(path: &Path) -> Option<File> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok()?;
    }
    File::options().read(true).write(true).create(true).truncate(false).open(path).ok()
}

/// Who holds the sync lock now, as its file says. None when nobody holds it.
pub fn holder(path: &Path) -> Option<String> {
    let file = File::open(path).ok()?;
    match file.try_lock() {
        Err(TryLockError::WouldBlock) => {
            let text = std::fs::read_to_string(path).unwrap_or_default().trim().to_string();
            Some(if text.is_empty() { "pid unknown".into() } else { text })
        }
        _ => None,
    }
}

/// True when `.git/index.lock` of `repo` did not change for a minute and no zendo syncs: most likely
/// a git that was killed left it, and every commit fails until somebody removes it (NOTESREP-18).
pub fn stale_index_lock(repo: &Path, lock: &Path) -> bool {
    let age = std::fs::metadata(repo.join(".git/index.lock")).and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok());
    age.is_some_and(|a| a > STALE_INDEX_LOCK) && holder(lock).is_none()
}

#[cfg(test)]
#[path = "lock_tests.rs"]
mod tests;
