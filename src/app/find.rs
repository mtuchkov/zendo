//! Find (FIND-1 to FIND-17): the recent files with an empty query; with a query the files by path,
//! then the markdown files by text; the preview of the selected file.
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
    pub query: String,
    pub items: Vec<FindItem>,
    pub sel: usize,
    /// The selected file, read for the preview.
    pub preview: Option<Note>,
    /// The file that zendo saw last in the note pane.
    pub(crate) seen: Option<PathBuf>,
    /// What the panel stands over: the mode, the focus, a session on the full screen (FIND-12).
    back: Option<(Mode, Focus, Option<usize>)>,
    /// The files of the repository and the markdown files with their text, read at the first query.
    files: Vec<PathBuf>,
    docs: Vec<Note>,
}

impl App {
    /// `alt-/`, `/` in read mode: open the find panel with the recent files (FIND-1, FIND-15).
    pub fn open_find(&mut self) {
        if self.mode == Mode::Find {
            return;
        }
        self.track_recent();
        if self.mode == Mode::Write {
            self.save();
        }
        self.hold = None;
        self.files.find.back = Some((self.mode, self.focus, self.cli.view.take()));
        self.files.find.query.clear();
        (self.files.find.files, self.files.find.docs) = (Vec::new(), Vec::new());
        self.mode = Mode::Find;
        self.run_find();
    }

    /// `esc`: the screen as it was under the panel (FIND-12).
    pub(crate) fn close_find(&mut self) {
        let (mode, focus, view) = self.files.find.back.take().unwrap_or((Mode::Read, self.focus, None));
        (self.mode, self.focus, self.cli.view) = (mode, focus, view);
        self.files.find.items.clear();
        self.files.find.preview = None;
    }

    /// A key in the find panel: these keys are fixed, not in the keymap.
    pub(crate) fn key_find(&mut self, k: KeyEvent) {
        let plain = !k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        match k.code {
            KeyCode::Esc => self.close_find(),
            KeyCode::Enter => self.find_enter(),
            KeyCode::Down | KeyCode::Tab => self.find_move(true),
            KeyCode::Char('j') if ctrl => self.find_move(true),
            KeyCode::Up | KeyCode::BackTab => self.find_move(false),
            KeyCode::Char('k') if ctrl => self.find_move(false),
            KeyCode::Backspace => {
                self.files.find.query.pop();
                self.run_find();
            }
            KeyCode::Char('u') if ctrl => {
                self.files.find.query.clear();
                self.run_find();
            }
            KeyCode::Char(c) if plain => {
                self.files.find.query.push(c);
                self.run_find();
            }
            _ => {}
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
    /// then by text (FIND-2, FIND-9, FIND-14).
    pub(crate) fn run_find(&mut self) {
        let q = self.files.find.query.trim().to_string();
        self.files.find.items = match q.chars().next() {
            None => self.recent_items(),
            Some('?') => Vec::new(),
            Some('#') => items_of(self.files.notes(), search::search(self.files.notes(), &q)),
            Some(_) => self.query_items(&q),
        };
        self.files.find.sel = 0;
        self.load_preview();
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

    /// The text files whose path has every word, recent first; then the markdown files whose text
    /// has the query (FIND-9, FIND-16).
    fn query_items(&mut self, q: &str) -> Vec<FindItem> {
        if self.files.find.files.is_empty() {
            let root = self.cfg.root.clone();
            self.files.find.files = crate::app::at::repo_files(&root);
            // FIND-18: the notes of the notes repo, by their absolute path (`root.join` keeps it).
            if let Some(dir) = self.repo_notes_outside() {
                let md = crate::app::at::repo_files(&dir).into_iter().filter(|f| f.extension().is_some_and(|x| x == "md"));
                self.files.find.files.extend(md.map(|f| dir.join(f)));
            }
            self.files.find.docs = self.files.find.files.iter().filter(|f| !code::is_code(f)).filter_map(|f| notes::load(&root.join(f)).ok()).collect();
        }
        let notes_dir = self.repo_notes_outside();
        let words: Vec<String> = q.to_lowercase().split_whitespace().map(str::to_string).collect();
        let rank = |rel: &str| self.state.recent.iter().position(|o| o.path == rel).unwrap_or(RECENT);
        let mut by_path: Vec<&PathBuf> = self.files.find.files.iter().filter(|f| has_words(&shown_path(f, notes_dir.as_deref()), &words)).collect();
        by_path.sort_by_key(|f| (rank(&f.display().to_string()), f.to_path_buf()));
        let root = &self.cfg.root;
        let mut items: Vec<FindItem> = by_path
            .into_iter()
            .map(|f| root.join(f))
            .filter(|p| !code::is_code(p) || crate::docs::tree::is_text(p))
            .take(BY_PATH)
            .filter_map(|path| Some(FindItem { title: notes::load(&path).ok()?.title, path, at: None, snippets: Vec::new() }))
            .collect();
        let text = items_of(&self.files.find.docs, search::search(&self.files.find.docs, q));
        items.extend(text.into_iter().filter(|t| !items.iter().any(|i| i.path == t.path)).collect::<Vec<_>>());
        items
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

/// The rows of the matches in the text.
fn items_of(docs: &[Note], groups: Vec<Group>) -> Vec<FindItem> {
    groups.into_iter().map(|g| FindItem { path: docs[g.note].path.clone(), title: docs[g.note].title.clone(), at: None, snippets: g.snippets }).collect()
}

use crate::text::code;
