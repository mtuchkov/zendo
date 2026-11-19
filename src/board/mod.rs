//! The message board between zendo sessions (BOARD, ADR 0021): a journal of the user that the MCP
//! servers, the watches and the zendo processes append to, and the state that it folds into.

mod beats;
mod compact;
mod deliver;
mod events;
mod journal;
pub mod schema;
mod state;
mod state_posts;
pub mod tools;
mod tools_posts;
pub mod watch;

pub use beats::{Beat, SessionInfo};
pub use deliver::{label, tagged};
pub use events::*;
pub use journal::Journal;
pub use state::State;
pub use compact::MSG_RETAIN_MS;
pub use journal::COMPACT_BYTES;
pub use state_posts::{End, HISTORY_MS, Lease, PostState};
pub use tools_posts::post_json;

use std::path::{Path, PathBuf};

/// A zendo process that wrote no heartbeat for this long is gone (BOARD-22).
pub const HEARTBEAT_MS: u64 = 5_000;
/// The fuse: this many turns in a row that the board starts without a question of the user
/// (BOARD-37).
pub const FUSE: usize = 10;

/// The time now, in milliseconds since the Unix epoch.
pub fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

/// A new id with the prefix `p`: the time and the process, so two writers never collide.
pub fn new_id(p: &str) -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    format!("{p}{:x}{:x}{:x}", now_ms(), std::process::id(), N.fetch_add(1, Ordering::Relaxed))
}

/// The folder of the board of the user: `$ZENDO_BOARD_DIR`, else `~/.zendo/board`.
pub fn default_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("ZENDO_BOARD_DIR").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d));
    }
    std::env::var_os("HOME").filter(|h| !h.is_empty()).map(|h| PathBuf::from(h).join(".zendo").join("board"))
}

/// The name of a repository on the board: the path of its main checkout, so its worktrees share
/// it (ADR 0021). `dir` is any folder in the repository or in one of its worktrees.
pub fn repo_of(dir: &Path) -> String {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .current_dir(dir)
        .output();
    let common = out.ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let main = common.map(PathBuf::from).and_then(|c| c.parent().map(Path::to_path_buf));
    let path = main.unwrap_or_else(|| dir.to_path_buf());
    std::fs::canonicalize(&path).unwrap_or(path).display().to_string()
}

/// The start time of the process `pid`, so that another process with the same id does not count
/// (BOARD-30). None where zendo cannot read it.
pub fn proc_start(pid: u32) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after = stat.rsplit_once(')')?.1;
    after.split_whitespace().nth(19)?.parse().ok()
}

/// The process `pid` runs, and started at `start` when that is known.
pub fn proc_alive(pid: u32, start: Option<u64>) -> bool {
    let Some(p) = rustix::process::Pid::from_raw(pid as i32) else { return false };
    let alive = rustix::process::test_kill_process(p).is_ok();
    alive && (start.is_none() || proc_start(pid) == start)
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
