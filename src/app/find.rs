//! Find (find.md, find-modes.md, find-background.md, find-regex.md): the panel, its modes and keys;
//! the worker searches off the UI thread (ADR 0017), and `find_list.rs` makes the lists.
use super::*;
use crate::docs::grep::{self, Ask, Hit, Out, Worker};
pub use find_list::FindMode;
use std::sync::Arc;

#[path = "find_list.rs"]
mod find_list;

/// The most recent files that zendo keeps for a repository.
pub(crate) const RECENT: usize = 50;

/// One row of the find panel.
#[derive(Clone, Debug)]
pub struct FindItem {
    pub path: PathBuf,
    pub title: String,
    /// When the user opened it, for a recent file.
    pub at: Option<chrono::DateTime<Local>>,
    /// The lines with a match in the text: the first one is where `enter` opens the file.
    pub snippets: Vec<grep::Snippet>,
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
    pub mode: FindMode,
    /// `[.*]`: the query is a regular expression (FIND-44); it stays from one open to the next.
    pub regex: bool,
    /// The regular expression does not compile (FIND-46).
    pub bad_regex: bool,
    /// The search in the text of the query runs (FIND-43).
    pub running: bool,
    /// The number of rows of each mode, in the order of `FindMode::ALL`.
    pub counts: [usize; 4],
    /// The marks of a match in a line, for the preview.
    pub(crate) mark: Option<regex::Regex>,
    /// The files of the repository, walked when the panel opened (FIND-39); None until they come.
    files: Option<Vec<PathBuf>>,
    /// The matches in the text of the query.
    hits: Vec<Hit>,
    /// The number of the query; the worker drops older ones (FIND-40).
    seq: u64,
    worker: Option<Worker>,
    /// The worker wakes the event loop when it sends results.
    pub wake: Arc<tokio::sync::Notify>,
    /// The forms of a requirement ID: the tag patterns (FIND-34).
    id_patterns: Vec<regex::Regex>,
    /// Titles and the text test of files, read once while the panel is open.
    meta: std::cell::RefCell<std::collections::HashMap<PathBuf, (String, bool)>>,
}

impl Find {
    /// The search of a query runs, or the walk of the files of an open panel has not ended.
    pub(crate) fn busy(&self, open: bool) -> bool {
        self.running || (open && self.files.is_none())
    }
}

impl App {
    /// `alt-/`, `/`: open the find panel in Docs mode, with the selected text as the query
    /// (FIND-1, FIND-15, FIND-19, FIND-22, FIND-27).
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
        let f = &mut self.files.find;
        f.back = Some((self.mode, self.focus, self.cli.view.take()));
        f.query.set(&seed);
        f.mode = FindMode::Docs;
        (f.files, f.hits) = (None, Vec::new());
        f.meta.borrow_mut().clear();
        let wake = f.wake.clone();
        let roots: Vec<PathBuf> = std::iter::once(self.cfg.root.clone()).chain(self.repo_notes_outside()).collect();
        self.files.find.worker.get_or_insert_with(|| Worker::start(wake)).walk(roots);
        self.files.find.id_patterns = self.id_patterns();
        self.mode = Mode::Find;
        self.run_find();
    }

    /// The forms of a requirement ID: the `tag_patterns` of `project-structure.toml` that compile,
    /// or the default when none is left (FIND-34, FIND-35).
    fn id_patterns(&self) -> Vec<regex::Regex> {
        let set = crate::docs::structure::Structure::load(&self.cfg.root).and_then(Result::ok).and_then(|s| s.project.tag_patterns);
        let ok: Vec<regex::Regex> = set.into_iter().flatten().filter_map(|p| regex::Regex::new(&p).ok()).collect();
        if ok.is_empty() { vec![regex::Regex::new(grep::DEFAULT_ID).expect("the default compiles")] } else { ok }
    }

    /// `esc`: the screen as it was under the panel (FIND-12).
    pub(crate) fn close_find(&mut self) {
        let (mode, focus, view) = self.files.find.back.take().unwrap_or((Mode::Read, self.focus, None));
        (self.mode, self.focus, self.cli.view) = (mode, focus, view);
        let f = &mut self.files.find;
        f.seq += 1;
        if let Some(w) = &f.worker {
            w.stop(f.seq);
        }
        (f.files, f.hits, f.items, f.preview, f.running) = (None, Vec::new(), Vec::new(), None, false);
        f.meta.borrow_mut().clear();
    }

    /// A key in the find panel: the keys of `[list]`, then the field; `esc` is fixed (PANES-48).
    pub(crate) fn key_find(&mut self, k: KeyEvent) {
        match (k.code, self.keys.local(crate::keys::Pane::List, &k)) {
            (KeyCode::Esc, _) => self.close_find(),
            (_, Some("open")) => self.find_enter(),
            (_, Some("down")) => self.find_move(true),
            (_, Some("up")) => self.find_move(false),
            (_, Some("next_mode")) => self.find_mode(false),
            (_, Some("prev_mode")) => self.find_mode(true),
            (_, Some("regex")) => self.toggle_find_regex(),
            _ => {
                if self.field_key(&k, |a| Some(&mut a.files.find.query)) == Some(true) {
                    self.run_find();
                }
            }
        }
    }

    /// `tab`, `shift-tab`: the next or the previous mode; the query stays (FIND-21, FIND-24).
    fn find_mode(&mut self, back: bool) {
        self.files.find.mode = self.files.find.mode.step(back);
        self.files.find.sel = 0;
        self.make_find_list();
    }

    /// `[.*]` or `alt-r`: the regular expression on or off (FIND-44).
    pub fn toggle_find_regex(&mut self) {
        self.files.find.regex = !self.files.find.regex;
        self.run_find();
    }

    fn find_move(&mut self, down: bool) {
        let last = self.files.find.items.len().saturating_sub(1);
        self.files.find.sel = if down { (self.files.find.sel + 1).min(last) } else { self.files.find.sel.saturating_sub(1) };
        self.load_preview();
    }

    /// `enter`: open the selected file in the note pane at the line of the match (FIND-11); after
    /// `?` in Notes mode, ask the agent about all notes (FIND-41).
    fn find_enter(&mut self) {
        if let Some(q) = self.files.find.query.strip_prefix('?').map(|q| q.trim().to_string()) {
            if self.files.find.mode != FindMode::Notes {
                return;
            }
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

    /// The query changed: the search by path now, the search in the text on the worker (FIND-37).
    pub(crate) fn run_find(&mut self) {
        let ask = self.find_ask();
        let f = &mut self.files.find;
        f.bad_regex = matches!(&ask, Some(Ask::Text { query, regex: true }) if regex::Regex::new(query).is_err());
        if f.bad_regex {
            // FIND-46: the list keeps the results of the last valid query.
            return;
        }
        f.seq += 1;
        f.hits.clear();
        f.mark = ask.as_ref().and_then(Ask::mark);
        f.running = ask.is_some();
        if let Some(w) = &f.worker {
            match ask {
                Some(a) => w.search(f.seq, a),
                None => w.stop(f.seq),
            }
        }
        f.sel = 0;
        self.make_find_list();
    }

    /// What the worker looks for: nothing for an empty query and for `?` (FIND-41).
    fn find_ask(&self) -> Option<Ask> {
        let q = self.files.find.query.trim();
        // A query across lines is text, also when it starts with `#` or `?` (FIND-20).
        if q.contains(crate::docs::search::NEW_LINE) {
            return Some(Ask::Lines(q.to_string()));
        }
        match q.chars().next()? {
            '?' => None,
            '#' => {
                let w = q[1..].trim().to_string();
                // The patterns that the whole word matches: the word has the form of their IDs.
                let whole = |p: &&regex::Regex| p.find(&w).is_some_and(|m| m.start() == 0 && m.end() == w.len());
                let patterns: Vec<regex::Regex> = self.files.find.id_patterns.iter().filter(whole).cloned().collect();
                (!w.is_empty()).then(|| if patterns.is_empty() { Ask::Tag(w.clone()) } else { Ask::Id { word: w.clone(), patterns } })
            }
            _ => Some(Ask::Text { query: q.to_string(), regex: self.files.find.regex }),
        }
    }

    /// Take what the worker sent: the list of files, the matches of the query (FIND-36).
    pub fn poll_find(&mut self) {
        let Some(w) = &self.files.find.worker else { return };
        let outs = w.poll();
        if !outs.is_empty() {
            outs.into_iter().for_each(|o| self.take_find(o));
            self.make_find_list();
        }
    }

    fn take_find(&mut self, out: Out) {
        let f = &mut self.files.find;
        match out {
            Out::Files(files) if self.mode == Mode::Find => f.files = Some(files),
            Out::Hits { seq, hits } if seq == f.seq => (f.hits, f.running) = (hits, false),
            _ => {}
        }
    }

    /// Wait until the walk and the search of the query end: for tests and benchmarks.
    pub fn settle_find(&mut self) {
        let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while (self.files.find.running || self.files.find.files.is_none()) && self.mode == Mode::Find && std::time::Instant::now() < until {
            let Some(out) = self.files.find.worker.as_ref().and_then(|w| w.wait(std::time::Duration::from_millis(50))) else { continue };
            self.take_find(out);
            self.make_find_list();
        }
    }

    fn load_preview(&mut self) {
        self.files.find.preview = self.files.find.items.get(self.files.find.sel).and_then(|i| notes::load(&i.path).ok());
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
        self.state.recent.insert(0, crate::setup::state::Opened { path: rel, at: Local::now() });
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
