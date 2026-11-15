//! The document of the session (docs/zendo/files/files-worktree-note.md): the note pane shows, and a
//! question names, the version of the open document in the checkout of the active agent session;
//! `m` and `u` held in read mode show the version of `main` and of the main checkout on disk.
use super::*;
use crate::docs::tree::worktree;

/// A held key is up when no auto-repeat came for this long: before the first repeat, and after it
/// (FILES-46).
const FIRST_REPEAT: Duration = Duration::from_millis(700);
const NEXT_REPEAT: Duration = Duration::from_millis(250);

/// The version that a held key shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Other {
    /// The last commit of `main` (FILES-45).
    Main,
    /// The main checkout on disk (FILES-47).
    Disk,
}

impl Other {
    /// The name of the version in the head of the note and in the message when it does not exist.
    pub fn name(self) -> &'static str {
        match self {
            Other::Main => "main",
            Other::Disk => "disk",
        }
    }
}

/// `m` or `u` held in read mode: the version it shows, and when its last auto-repeat came.
pub struct Peek {
    pub on: Option<Other>,
    pub last: Instant,
    repeated: bool,
    pub note: Option<Note>,
}

impl Default for Peek {
    fn default() -> Self {
        Peek { on: None, last: Instant::now(), repeated: false, note: None }
    }
}

impl App {
    /// The path of a file of the repo without its worktree: `.worktrees/1/docs/a.md` → `docs/a.md`.
    /// None outside the repo.
    pub fn doc_rel(&self, p: &Path) -> Option<String> {
        let rel = p.strip_prefix(&self.cfg.root).ok()?.to_string_lossy().into_owned();
        Some(worktree::split(&rel).map_or(rel.clone(), |(_, rest)| rest.to_string()))
    }

    /// The version of a file in the checkout of the active agent session: its worktree, or the main
    /// checkout while it has none (FILES-38, FILES-42). A file outside the repo stays as it is.
    pub fn session_path(&self, p: &Path) -> PathBuf {
        let Some(doc) = self.doc_rel(p) else { return p.to_path_buf() };
        match worktree::of(&self.cfg.root, self.state.session) {
            Some(wt) => self.cfg.root.join(wt).join(doc),
            None => self.cfg.root.join(doc),
        }
    }

    /// The file of a new note: in the notes folder of the checkout of the session; the notes of a
    /// notes repo outside the repo stay there (FILES-49).
    pub fn new_note_path(&self, title: &str) -> PathBuf {
        notes::Store::new(self.cfg.root.clone(), self.session_path(&self.store.dir)).path_for(title, Local::now().date_naive())
    }

    /// The open document has no version in the checkout of the session (FILES-40).
    pub fn doc_missing(&self) -> bool {
        self.peek.on.is_none() && self.current.as_ref().is_some_and(|p| !p.exists())
    }

    /// After a change of the agent session: the version of the open document in that session
    /// (FILES-39, FILES-41).
    pub(crate) fn follow_session(&mut self) {
        if let Some(p) = self.current.clone().filter(|p| self.session_path(p) != *p) {
            self.open_file(p);
        }
    }

    /// `m` or `u` in read mode: show the other version, or note one more auto-repeat of the key.
    pub(crate) fn peek_at(&mut self, other: Other) {
        self.peek.last = Instant::now();
        if self.peek.on == Some(other) {
            self.peek.repeated = true;
            return;
        }
        let doc = self.current.as_ref().and_then(|p| self.doc_rel(p));
        self.peek.note = doc.and_then(|d| self.other_version(other, &d));
        self.peek.on = Some(other);
        self.peek.repeated = false;
    }

    /// The note of `doc` in the last commit of `main` or in the main checkout on disk (FILES-48).
    fn other_version(&self, other: Other, doc: &str) -> Option<Note> {
        let path = self.cfg.root.join(doc);
        match other {
            Other::Disk => notes::load(&path).ok(),
            Other::Main => {
                let out = std::process::Command::new("git").args(["show", &format!("main:{doc}")]).current_dir(&self.cfg.root).stderr(std::process::Stdio::null()).output().ok()?;
                let body = out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())?;
                let title = notes::title_of(&body).unwrap_or_else(|| doc.to_string());
                Some(Note { path, title, created: None, modified: chrono::Local::now(), body })
            }
        }
    }

    /// The held key went up, or another key came: the version of the session again.
    pub(crate) fn peek_end(&mut self) {
        self.peek.on = None;
        self.peek.note = None;
    }

    /// A key event while `m` or `u` is held: its release or another key ends the hold. True when
    /// the event is done with.
    pub(crate) fn peek_key(&mut self, k: &KeyEvent) -> bool {
        let up = k.kind == ratatui::crossterm::event::KeyEventKind::Release;
        let held = matches!(self.keys.local(crate::keys::Pane::Read, k), Some("peek_main" | "peek_disk"));
        if self.peek.on.is_some() && (up || !held) {
            self.peek_end();
        }
        up
    }

    /// No auto-repeat of the held key came for a while: it is up (FILES-46).
    pub(crate) fn peek_tick(&mut self) {
        let wait = if self.peek.repeated { NEXT_REPEAT } else { FIRST_REPEAT };
        if self.peek.on.is_some() && self.peek.last.elapsed() > wait {
            self.peek_end();
        }
    }
}
