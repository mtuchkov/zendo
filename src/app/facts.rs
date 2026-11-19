//! The small facts of the disk that a frame shows: the text of a file at HEAD for the delta, the
//! branch of a checkout, whether a file exists. Workers read them; a frame reads the memos (IO-1,
//! IO-6, ADR 0024).
use super::*;
use crate::worker::Memo;

/// `git show` for the delta gets this long (IO-5).
const HEAD_LIMIT: Duration = Duration::from_secs(5);

/// A checkout and a path in it.
pub type InCheckout = (PathBuf, String);

pub struct Facts {
    /// The text of a file at HEAD; None for a file that HEAD does not have.
    pub head: Memo<InCheckout, Option<String>>,
    /// The branch of the checkout that holds a path (FILES-65).
    pub branch: Memo<PathBuf, Option<String>>,
    pub exists: Memo<PathBuf, bool>,
    /// A stamp of a file that changes when the file changes; None when it is not there.
    pub stamp: Memo<PathBuf, Option<u64>>,
}

impl Default for Facts {
    fn default() -> Self {
        Facts::with_head(head_of)
    }
}

impl Facts {
    /// Facts whose text at HEAD comes from `head`: a test puts a slow git here.
    pub fn with_head(head: fn(&InCheckout) -> Option<String>) -> Facts {
        Facts {
            head: Memo::new(head, Duration::from_secs(5)),
            branch: Memo::new(|p: &PathBuf| crate::docs::notes::git_branch(p), Duration::from_secs(1)),
            exists: Memo::new(|p: &PathBuf| p.exists(), Duration::from_secs(1)),
            stamp: Memo::new(|p: &PathBuf| p.exists().then(|| crate::app::composer::chip_view::stamp_of(p)), Duration::from_secs(1)),
        }
    }

    /// For tests: the workers compute each 5 ms.
    pub fn quick(&self) {
        let d = Duration::from_millis(5);
        self.head.set_every(d);
        self.branch.set_every(d);
        self.exists.set_every(d);
        self.stamp.set_every(d);
    }

    /// For tests: wait at most 2 s until each worker ended two more rounds, so that each value
    /// that a frame asked for is known and fresh.
    pub fn settle(&self) {
        let rounds = || [self.head.rounds(), self.branch.rounds(), self.exists.rounds(), self.stamp.rounds()];
        let start = rounds();
        let end = std::time::Instant::now() + Duration::from_secs(2);
        while rounds().iter().zip(start).any(|(now, then)| *now < then + 2) && std::time::Instant::now() < end {
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Whether `p` exists. A path that the worker did not look at yet counts as there, so that
    /// nothing shows as missing before it looked.
    pub fn exists(&self, p: &Path) -> bool {
        self.exists.get(&p.to_path_buf()).unwrap_or(true)
    }

    /// The branch of the checkout that holds `p`, when the worker knows it.
    pub fn branch(&self, p: &Path) -> Option<String> {
        self.branch.get(&p.to_path_buf()).flatten()
    }
}

impl App {
    /// Save the state in `.zendo/db.json` through the writer, without the sync: a file of zendo
    /// (IO-1, IO-3).
    pub fn save_state(&self) {
        if let Ok(bytes) = serde_json::to_vec_pretty(&self.state) {
            self.disk.write(&crate::setup::state::dir(&self.cfg.root).join("db.json"), bytes, crate::worker::Sync::Own);
        }
    }

    /// Save the loglets in `.zendo/db.log.json` through the writer (UNDO-4, IO-1).
    pub fn save_logs(&self) {
        if let Ok(bytes) = serde_json::to_vec(&self.logs) {
            self.disk.write(&crate::setup::state::dir(&self.cfg.root).join(crate::setup::logs::FILE), bytes, crate::worker::Sync::Own);
        }
    }

    /// For tests: the workers of the facts and of the tree compute each 5 ms.
    pub fn quick_facts(&self) {
        self.facts.quick();
        self.files.tree().is_file.set_every(Duration::from_millis(5));
    }

    /// For tests: wait until the workers know each fact that the last frame asked for.
    pub fn settle_facts(&self) {
        let tree = self.files.tree().is_file.rounds();
        self.facts.settle();
        let end = std::time::Instant::now() + Duration::from_secs(2);
        while self.files.tree().is_file.rounds() < tree + 2 && std::time::Instant::now() < end {
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// The worktree of agent session `n`, from the memo that the start seeded (SESSIONS-23, IO-1).
    pub fn worktree_of(&self, n: usize) -> Option<PathBuf> {
        let rel = crate::docs::tree::worktree::rel(n);
        (n > 0 && self.facts.exists.get(&self.cfg.root.join(&rel).join(".git")) == Some(true)).then_some(rel)
    }
}

/// The text of a file at HEAD of its checkout, from git with a time limit.
pub fn head_of((dir, rel): &InCheckout) -> Option<String> {
    crate::setup::sync::git_out(dir, &["show", &format!("HEAD:{rel}")], HEAD_LIMIT).ok()
}
