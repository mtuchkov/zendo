//! What the files of the stats of one zendo process share: the usage counts (USAGE-4) and the times
//! of the actions (RESP-3). `stats = false` in the config turns both off. Each file has the same
//! name, the same head and the same write.

use chrono::{DateTime, SecondsFormat, Utc};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The time between two writes of a file (USAGE-3).
pub const EVERY: Duration = Duration::from_secs(15);

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
