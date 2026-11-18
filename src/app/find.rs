//! Find (FIND-1 to FIND-21): the recent files with an empty query; with a query the files by path,
//! then the markdown files by text from the search thread; the preview of the selected file.
use super::find_worker::{Found, Worker, items_of};
use super::*;
use crate::setup::state::Opened;

/// The most recent files that zendo keeps for a repository.
const RECENT: usize = 50;
/// The most files that a query lists by their path.
const BY_PATH: usize = 50;

/// One row of the find panel.
#[derive(Clone, Debug)]
pub struct FindItem {
    pub path: PathBuf,
    pub title: String,
    /// When the user opened it, for a recent file.
    pub at: Option<chrono::DateTime<Local>>,
    /// The lines with a match in the text: the first one is where `enter` opens the file.
    pub snippets: Vec<search::Snippet>,
}

#[derive(Default)]
pub struct Find {
    pub query: crate::text::field::Field,
    pub items: Vec<FindItem>,
    pub sel: usize,
    /// The selected file, read for the preview.
    pub preview: Option<Note>,
    /// The file that zendo saw last in the note pane.
    pub(crate) seen: Option<PathBuf>,
    /// What the panel stands over: the mode, the focus, a session on the full screen (FIND-12).
    back: Option<(Mode, Focus, Option<usize>)>,
    /// The files of the repository, from the search thread.
    files: Vec<PathBuf>,
    /// The thread of the text search, while the panel is open (FIND-21).
    worker: Option<Worker>,
    /// The number of the newest query, and of the query whose text matches the list shows.
    asked: u64,
    answered: u64,
    /// The rows of the query: the files by path, and the markdown files by text.
    by_path: Vec<FindItem>,
    by_text: Vec<FindItem>,
}

impl Find {
    /// The search thread has not answered the newest query yet.
    pub fn searching(&self) -> bool {
        self.asked != self.answered
    }
}

/// A query of text: not empty, not a tag, not a question; or a query across lines (FIND-20).
fn is_text(q: &str) -> bool {
    q.contains(search::NEW_LINE) || !matches!(q.chars().next(), None | Some('?' | '#'))
}

impl App {
    /// `alt-/`, `/` in read mode: open the find panel with the recent files, or with the selected
    /// text as the query (FIND-1, FIND-15, FIND-19).
    pub fn open_find(&mut self) {
        if self.mode == Mode::Find {
            return;
        }
        self.track_recent();
        let seed = self.find_seed();
        if self.mode == Mode::Write {
            self.save();
        }
        self.hold = None;
        self.files.find.back = Some((self.mode, self.focus, self.cli.view.take()));
        self.files.find.query.set(&seed);
        self.files.find.files = Vec::new();
        // The thread reads the files now, while the user types the query (FIND-21).
        self.files.find.worker = Some(Worker::start(self.cfg.root.clone(), self.repo_notes_outside(), self.cli.notify.clone()));
        self.mode = Mode::Find;
        self.run_find();
    }

    /// `esc`: the screen as it was under the panel (FIND-12).
    pub(crate) fn close_find(&mut self) {
        let (mode, focus, view) = self.files.find.back.take().unwrap_or((Mode::Read, self.focus, None));
        (self.mode, self.focus, self.cli.view) = (mode, focus, view);
        let f = &mut self.files.find;
        (f.worker, f.answered) = (None, f.asked);
        (f.items, f.by_path, f.by_text) = (Vec::new(), Vec::new(), Vec::new());
        f.preview = None;
    }

    /// A key in the find panel: the keys of `[list]`, then the field; `esc` is fixed (PANES-48).
    pub(crate) fn key_find(&mut self, k: KeyEvent) {
        match (k.code, self.keys.local(crate::keys::Pane::List, &k)) {
            (KeyCode::Esc, _) => self.close_find(),
            (_, Some("open")) => self.find_enter(),
            (_, Some("down")) => self.find_move(true),
            (_, Some("up")) => self.find_move(false),
            _ => {
                if self.field_key(&k, |a| Some(&mut a.files.find.query)) == Some(true) {
                    self.run_find();
                }
            }
        }
    }

    fn find_move(&mut self, down: bool) {
        let last = self.files.find.items.len().saturating_sub(1);
        self.files.find.sel = if down { (self.files.find.sel + 1).min(last) } else { self.files.find.sel.saturating_sub(1) };
        self.load_preview();
    }

    /// `enter`: open the selected file in the note pane at the line of the match (FIND-11); after
    /// `?`, ask the agent about all notes (FIND-14).
    fn find_enter(&mut self) {
        // `enter` opens what the query finds, also when the text search has not answered yet.
        self.settle_find();
        if let Some(q) = self.files.find.query.strip_prefix('?').map(|q| q.trim().to_string()) {
            self.close_find();
            self.mode = Mode::Read;
            self.threads.pin(ALL_KEY);
            self.open_panel();
            return self.ask(q, ALL_KEY.into());
        }
        let Some(item) = self.files.find.items.get(self.files.find.sel).cloned() else { return };
        self.close_find();
        self.cli.view = None;
        self.open_file(item.path);
        (self.mode, self.focus) = (Mode::Read, Focus::Note);
        if let Some(line) = item.snippets.first().map(|s| s.line) {
            let rows = crate::ui::source_rows(self, self.column_width() as usize);
            self.reader.jump(rows.get(line).copied().unwrap_or(0));
        }
        self.track_recent();
    }

    /// The list for the query: the recent files, the notes with a tag, or the files by path and
    /// then by text (FIND-2, FIND-9, FIND-14, FIND-20).
    pub(crate) fn run_find(&mut self) {
        let q = self.files.find.query.trim().to_string();
        self.files.find.asked += 1;
        if is_text(&q) {
            // The files by path now; the text comes from the search thread (FIND-21).
            self.files.find.by_path = self.path_items(&q);
            self.ask_text(&q);
            self.files.find.items = self.files.find.rows();
        } else {
            let f = &mut self.files.find;
            (f.answered, f.by_path, f.by_text) = (f.asked, Vec::new(), Vec::new());
            self.files.find.items = match q.chars().next() {
                None => self.recent_items(),
                Some('#') => items_of(self.files.notes(), search::search(self.files.notes(), &q)),
                _ => Vec::new(),
            };
        }
        self.files.find.sel = 0;
        self.load_preview();
    }

    fn ask_text(&mut self, q: &str) {
        if self.files.find.worker.is_none() {
            self.files.find.worker = Some(Worker::start(self.cfg.root.clone(), self.repo_notes_outside(), self.cli.notify.clone()));
        }
        if let Some(w) = &self.files.find.worker {
            w.ask(self.files.find.asked, q);
        }
    }

    /// The results that the search thread sent, without a wait: before each draw (FIND-21).
    pub(crate) fn poll_find(&mut self) {
        while let Some(found) = self.files.find.worker.as_ref().and_then(Worker::try_take) {
            self.on_found(found);
        }
    }

    /// Wait until the search thread answered the newest query: `enter`, and the tests.
    pub fn settle_find(&mut self) {
        self.poll_find();
        while self.files.find.searching() {
            let Some(found) = self.files.find.worker.as_ref().and_then(Worker::take) else { break };
            self.on_found(found);
        }
    }

    /// A result of the search thread: the list gets it, and the selected file stays selected.
    fn on_found(&mut self, found: Found) {
        let q = self.files.find.query.trim().to_string();
        match found {
            Found::Files(files) => {
                self.files.find.files = files;
                if is_text(&q) {
                    self.files.find.by_path = self.path_items(&q);
                }
            }
            Found::Text(n, items) if n == self.files.find.asked => (self.files.find.by_text, self.files.find.answered) = (items, n),
            Found::Text(..) => return,
        }
        let f = &mut self.files.find;
        if !is_text(&q) {
            return;
        }
        let kept = f.items.get(f.sel).map(|i| i.path.clone());
        f.items = f.rows();
        f.sel = kept.and_then(|p| f.items.iter().position(|i| i.path == p)).unwrap_or(0);
        if f.preview.as_ref().map(|n| &n.path) != f.items.get(f.sel).map(|i| &i.path) {
            self.load_preview();
        }
    }

    fn load_preview(&mut self) {
        self.files.find.preview = self.files.find.items.get(self.files.find.sel).and_then(|i| notes::load(&i.path).ok());
    }

    /// The recent files that are on disk, without the open file (FIND-2, FIND-7).
    fn recent_items(&self) -> Vec<FindItem> {
        let open = self.files.current();
        let item = |o: &Opened| {
            let path = self.cfg.root.join(&o.path);
            let title = notes::load(&path).ok()?.title;
            (Some(&path) != open).then_some(FindItem { path, title, at: Some(o.at), snippets: Vec::new() })
        };
        self.state.recent.iter().filter_map(item).collect()
    }

    /// The text files whose path has every word, recent first (FIND-9, FIND-16); a note of the
    /// notes repo by its absolute path, which `root.join` keeps (FIND-18).
    fn path_items(&self, q: &str) -> Vec<FindItem> {
        let notes_dir = self.repo_notes_outside();
        let words: Vec<String> = q.to_lowercase().split_whitespace().map(str::to_string).collect();
        let rank = |rel: &str| self.state.recent.iter().position(|o| o.path == rel).unwrap_or(RECENT);
        let mut by_path: Vec<&PathBuf> = self.files.find.files.iter().filter(|f| has_words(&shown_path(f, notes_dir.as_deref()), &words)).collect();
        by_path.sort_by_key(|f| (rank(&f.display().to_string()), f.to_path_buf()));
        let root = &self.cfg.root;
        by_path
            .into_iter()
            .map(|f| root.join(f))
            .filter(|p| !code::is_code(p) || crate::docs::tree::is_text(p))
            .take(BY_PATH)
            .filter_map(|path| Some(FindItem { title: notes::load(&path).ok()?.title, path, at: None, snippets: Vec::new() }))
            .collect()
    }

    /// After a key or a tick: the file in the note pane is the most recent file, unless the float,
    /// the preview of the tree or a session on the full screen shows (FIND-6).
    pub(crate) fn track_recent(&mut self) {
        let shows = matches!(self.mode, Mode::Read | Mode::Write) && self.writer.float().is_none() && self.focus != Focus::Tree && self.cli.view.is_none();
        let Some(path) = self.files.current().cloned().filter(|_| shows) else { return };
        self.track_session_doc(&path);
        if self.files.find.seen.as_ref() != Some(&path) {
            self.files.find.seen = Some(path.clone());
            self.add_recent(&path);
        }
    }

    /// The file is the most recent one, and the state keeps it (FIND-4).
    pub(crate) fn add_recent(&mut self, path: &Path) {
        let rel = self.rel(path);
        self.state.recent.retain(|o| o.path != rel);
        self.state.recent.insert(0, Opened { path: rel, at: Local::now() });
        self.state.recent.truncate(RECENT);
        self.state.save(&self.cfg.root);
    }

    /// A rename or a move in zendo keeps the place of the file in the recent files (FIND-8).
    pub(crate) fn rename_recent(&mut self, old: &str, new: &str) {
        for o in self.state.recent.iter_mut().filter(|o| o.path == old) {
            o.path = new.to_string();
        }
        self.state.save(&self.cfg.root);
    }
}

/// The path has every word, without regard to letter case.
fn has_words(rel: &Path, words: &[String]) -> bool {
    let p = rel.display().to_string().to_lowercase();
    words.iter().all(|w| p.contains(w.as_str()))
}

/// The path that a query matches: a note of the notes repo as `notes/…`, not its whole path.
fn shown_path(f: &Path, notes_dir: Option<&Path>) -> PathBuf {
    match notes_dir.and_then(|d| f.strip_prefix(d).ok()) {
        Some(rel) => Path::new("notes").join(rel),
        None => f.to_path_buf(),
    }
}

impl Find {
    /// The files by path, then the files by text that are not in the list yet (FIND-9).
    fn rows(&self) -> Vec<FindItem> {
        let text = self.by_text.iter().filter(|t| !self.by_path.iter().any(|i| i.path == t.path));
        self.by_path.iter().chain(text).cloned().collect()
    }
}

use crate::text::code;
