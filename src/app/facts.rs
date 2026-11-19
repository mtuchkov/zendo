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
    /// The text of a small file of zendo, such as `.zendo/prompt.md`; None when it is not there.
    pub text: Memo<PathBuf, Option<String>>,
    /// The files of the repository, for `@` (CLAUDE-26); what git ignores is left out.
    pub files: Memo<PathBuf, Vec<PathBuf>>,
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
            stamp: Memo::new(|p: &PathBuf| p.exists().then(|| stamp_of(p)), Duration::from_secs(1)),
            text: Memo::new(|p: &PathBuf| std::fs::read_to_string(p).ok(), Duration::from_secs(1)),
            files: Memo::new(repo_files, Duration::from_secs(5)),
        }
    }

    /// For tests: the workers compute each 5 ms.
    pub fn quick(&self) {
        let d = Duration::from_millis(5);
        self.head.set_every(d);
        self.branch.set_every(d);
        self.exists.set_every(d);
        self.stamp.set_every(d);
        self.text.set_every(d);
        self.files.set_every(d);
    }

    /// For tests: wait at most 2 s until each worker ended two more rounds, so that each value
    /// that a frame asked for is known and fresh.
    pub fn settle(&self) {
        let rounds = || [self.head.rounds(), self.branch.rounds(), self.exists.rounds(), self.stamp.rounds(), self.text.rounds(), self.files.rounds()];
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

    /// The prompt of the questions: `.zendo/prompt.md`, else the default (IO-1). The start reads it
    /// (IO-7), and each tick asks for it, so that the memo keeps it fresh.
    pub fn prompt(&self) -> String {
        let path = crate::setup::state::dir(&self.cfg.root).join("prompt.md");
        self.facts.text.get(&path).flatten().unwrap_or_else(|| crate::setup::state::DEFAULT_PROMPT.to_string())
    }

    /// The worktree of agent session `n`, from the memo that the start seeded (SESSIONS-23, IO-1).
    pub fn worktree_of(&self, n: usize) -> Option<PathBuf> {
        let rel = crate::docs::tree::worktree::rel(n);
        (n > 0 && self.facts.exists.get(&self.cfg.root.join(&rel).join(".git")) == Some(true)).then_some(rel)
    }
}

/// The window of a file chip, on a worker: the text of the file, up to 1 MB; a file that is not
/// text and an image open in the program of the system (CLAUDE-52, CLAUDE-53).
pub fn chip_file(p: &Path, image: bool, opener: fn(&Path) -> std::io::Result<()>) -> crate::app::composer::chip_view::ChipFile {
    use crate::app::composer::chip_view::ChipFile;
    use std::io::Read;
    let outside = || ChipFile::Outside(opener(p).map_err(|e| format!("{} does not open: {e}", p.display())));
    if image {
        return outside();
    }
    let mut bytes = Vec::new();
    // The most bytes of a file that the chip window reads.
    match std::fs::File::open(p).and_then(|f| f.take(1 << 20).read_to_end(&mut bytes)) {
        Ok(_) if bytes.contains(&0) => outside(),
        Ok(_) => String::from_utf8(bytes).map_or_else(|_| outside(), ChipFile::Text),
        Err(e) => ChipFile::Error(format!("{} does not open: {e}", p.display())),
    }
}

/// A stamp of the text of a file: it changes when the file changes.
pub fn stamp_of(p: &Path) -> u64 {
    let m = std::fs::metadata(p).ok();
    let nanos = m.as_ref().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_nanos() as u64);
    nanos ^ m.map_or(0, |m| m.len()).rotate_left(32)
}

/// The files of the repository, from its root, at most 20 000; what git ignores is left out.
fn repo_files(root: &PathBuf) -> Vec<PathBuf> {
    let walk = ignore::WalkBuilder::new(root).hidden(true).git_ignore(true).require_git(false).build();
    walk.flatten()
        .filter(|e| e.file_type().is_some_and(|t| t.is_file()))
        .filter_map(|e| e.path().strip_prefix(root).ok().map(Path::to_path_buf))
        .take(20_000)
        .collect()
}

/// The text of a file at HEAD of its checkout, from git with a time limit.
pub fn head_of((dir, rel): &InCheckout) -> Option<String> {
    crate::setup::sync::git_out(dir, &["show", &format!("HEAD:{rel}")], HEAD_LIMIT).ok()
}
