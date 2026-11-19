//! The heartbeats of the zendo processes: each writes `zendo-<pid>.json` with its repository, the
//! time and the state of its sessions, about each second (ADR 0021).

use super::{HEARTBEAT_MS, Journal, now_ms};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SessionInfo {
    pub id: usize,
    pub name: String,
    /// `works`, `waits`, `idle` or `no thread`.
    pub state: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Beat {
    pub pid: u32,
    pub repo: String,
    pub at: u64,
    #[serde(default)]
    pub sessions: Vec<SessionInfo>,
}

impl Journal {
    pub fn beat_path(&self, pid: u32) -> std::path::PathBuf {
        self.dir().join(format!("zendo-{pid}.json"))
    }

    /// Write the heartbeat of this process, whole or not at all (FAIL-2).
    pub fn beat(&self, beat: &Beat) -> std::io::Result<()> {
        std::fs::create_dir_all(self.dir())?;
        crate::atomic::write(&self.beat_path(beat.pid), serde_json::to_vec(beat).unwrap_or_default())
    }

    /// Remove the heartbeat of this process: zendo quits.
    pub fn unbeat(&self, pid: u32) {
        let _ = std::fs::remove_file(self.beat_path(pid));
    }

    /// The heartbeats of every zendo process, fresh or not.
    pub fn beats(&self) -> Vec<Beat> {
        let Ok(dir) = std::fs::read_dir(self.dir()) else { return Vec::new() };
        dir.flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("zendo-"))
            .filter_map(|e| serde_json::from_slice(&std::fs::read(e.path()).ok()?).ok())
            .collect()
    }

    /// The last heartbeat of a zendo process of `repo`.
    pub fn last_beat(&self, repo: &str) -> Option<u64> {
        self.beats().into_iter().filter(|b| b.repo == repo).map(|b| b.at).max()
    }

    /// A zendo process of `repo` beats now.
    pub fn alive(&self, repo: &str) -> bool {
        self.last_beat(repo).is_some_and(|at| now_ms().saturating_sub(at) <= HEARTBEAT_MS)
    }
}
