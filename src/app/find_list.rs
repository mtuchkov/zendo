//! The lists of the find panel: the modes, the recent files, the matches by path and in the text,
//! and All as the merge of the other three (FIND-2, FIND-9, FIND-28 to FIND-31, FIND-42).
use super::*;
use std::cmp::Reverse;

/// The most files that a mode lists by their path.
const BY_PATH: usize = 50;
/// The most files that a mode lists by their text.
const BY_TEXT: usize = 200;

/// What the find panel looks for (FIND-21).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FindMode {
    Notes,
    #[default]
    Docs,
    Code,
    All,
}

impl FindMode {
    /// The `tab` order, left to right in the mode row.
    pub const ALL: [FindMode; 4] = [FindMode::Notes, FindMode::Docs, FindMode::Code, FindMode::All];

    pub fn name(self) -> &'static str {
        match self {
            FindMode::Notes => "Notes",
            FindMode::Docs => "Docs",
            FindMode::Code => "Code",
            FindMode::All => "All",
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|m| *m == self).unwrap_or(0)
    }

    /// The next mode, round from All back to Notes; or the previous one.
    pub fn step(self, back: bool) -> Self {
        let n = Self::ALL.len();
        Self::ALL[(self.index() + if back { n - 1 } else { 1 }) % n]
    }

}

/// A row before its title is read: the file, how it sorts, and the lines of its matches.
struct Row {
    path: PathBuf,
    key: (u8, usize, Reverse<usize>),
    at: Option<chrono::DateTime<Local>>,
    snippets: Vec<grep::Snippet>,
}

impl App {
    /// Notes, Docs or Code: the kind of a file (FIND-28 to FIND-30).
    fn find_kind(&self, p: &Path) -> FindMode {
        let notes = std::iter::once(self.cfg.notes_dir.clone()).chain(self.repo_notes_outside());
        if notes.into_iter().any(|d| p.starts_with(&d)) {
            FindMode::Notes
        } else if p.extension().is_some_and(|e| e == "md") {
            FindMode::Docs
        } else {
            FindMode::Code
        }
    }

    /// The lists of every mode for the query: the counts, and the rows of the active mode.
    pub(crate) fn make_find_list(&mut self) {
        let rows = self.find_rows();
        let counts = FindMode::ALL.map(|m| rows.iter().filter(|(ms, _)| ms.contains(&m)).count());
        let mode = self.files.find.mode;
        let mut mine: Vec<Row> = rows.into_iter().filter(|(ms, _)| ms.contains(&mode)).map(|(_, r)| r).collect();
        mine.sort_by(|a, b| (a.key, &a.path).cmp(&(b.key, &b.path)));
        let items = mine.into_iter().filter_map(|r| self.find_item(r)).collect::<Vec<_>>();
        let f = &mut self.files.find;
        (f.counts, f.items) = (counts, items);
        f.sel = f.sel.min(f.items.len().saturating_sub(1));
        self.files.find.preview = self.files.find.items.get(self.files.find.sel).and_then(|i| notes::load(&i.path).ok());
    }

    /// Every row, with the modes that show it.
    fn find_rows(&self) -> Vec<(Vec<FindMode>, Row)> {
        let q = self.files.find.query.trim();
        let rows = match q.chars().next() {
            None => return self.recent_rows(),
            Some('?') => Vec::new(),
            Some('#') => self.text_rows(&[]),
            Some(_) => {
                let by_path = self.path_rows(q);
                let seen: Vec<PathBuf> = by_path.iter().map(|(_, r)| r.path.clone()).collect();
                by_path.into_iter().chain(self.text_rows(&seen)).collect()
            }
        };
        rows.into_iter().map(|(k, r)| (vec![k, FindMode::All], r)).collect()
    }

    /// For an empty query: the notes by age in Notes; the recent files of their kind in the other
    /// modes; never the open file, nor a file that is gone (FIND-2, FIND-7, FIND-30, FIND-31).
    fn recent_rows(&self) -> Vec<(Vec<FindMode>, Row)> {
        let open = self.files.current();
        let row = |path: PathBuf, i: usize, at| Row { path, key: (0, i, Reverse(0)), at, snippets: Vec::new() };
        let recent = self.state.recent.iter().enumerate().map(|(i, o)| (self.cfg.root.join(&o.path), i, o.at));
        let recent = recent.filter(|(p, _, _)| Some(p) != open && self.facts.exists(p)).map(|(p, i, at)| {
            let kind = self.find_kind(&p);
            let modes = if kind == FindMode::Notes { vec![FindMode::All] } else { vec![kind, FindMode::All] };
            (modes, row(p, i, Some(at)))
        });
        let notes = self.files.notes().iter().enumerate().filter(|(_, n)| Some(&n.path) != open).map(|(i, n)| (vec![FindMode::Notes], row(n.path.clone(), i, None)));
        recent.chain(notes).collect()
    }

    /// The files whose path has every word, or matches the expression; recent files first (FIND-9,
    /// FIND-45).
    fn path_rows(&self, q: &str) -> Vec<(FindMode, Row)> {
        let Some(files) = &self.files.find.files else { return Vec::new() };
        let re = self.files.find.regex.then(|| regex::RegexBuilder::new(q).case_insensitive(true).build().ok()).flatten();
        let words: Vec<String> = q.to_lowercase().split_whitespace().map(str::to_string).collect();
        let hit = |p: &str| re.as_ref().map_or_else(|| words.iter().all(|w| p.to_lowercase().contains(w.as_str())), |r| r.is_match(p));
        let rank = |rel: &str| self.state.recent.iter().position(|o| o.path == rel).unwrap_or(RECENT);
        let mut per_kind = [0usize; 3];
        let mut out = Vec::new();
        for p in files {
            let shown = self.shown_path(p);
            if !hit(&shown) {
                continue;
            }
            let kind = self.find_kind(p);
            let n = &mut per_kind[kind as usize];
            if *n < BY_PATH && (kind != FindMode::Code || self.find_meta(p).1) {
                *n += 1;
                out.push((kind, Row { path: p.clone(), key: (0, rank(&shown), Reverse(0)), at: None, snippets: Vec::new() }));
            }
        }
        out
    }

    /// The path that a query matches: a note of the notes repo as `notes/…` (FIND-18).
    fn shown_path(&self, p: &Path) -> String {
        match self.repo_notes_outside().and_then(|d| p.strip_prefix(d).ok().map(Path::to_path_buf)) {
            Some(rel) => Path::new("notes").join(rel).display().to_string(),
            None => self.rel(p),
        }
    }

    /// The files with matches in the text, the whole ID and the most hits first (FIND-32, FIND-37).
    fn text_rows(&self, seen: &[PathBuf]) -> Vec<(FindMode, Row)> {
        let mut hits: Vec<&grep::Hit> = self.files.find.hits.iter().filter(|h| !seen.contains(&h.path)).collect();
        hits.sort_by(|a, b| (!a.exact, Reverse(a.count), &a.path).cmp(&(!b.exact, Reverse(b.count), &b.path)));
        let mut per_kind = [0usize; 3];
        let mut out = Vec::new();
        for h in hits {
            let kind = self.find_kind(&h.path);
            let n = &mut per_kind[kind as usize];
            if *n < BY_TEXT {
                *n += 1;
                out.push((kind, Row { path: h.path.clone(), key: (1 + u8::from(!h.exact), 0, Reverse(h.count)), at: None, snippets: h.snippets.clone() }));
            }
        }
        out
    }

    /// The title of a file and whether it is text, read once while the panel is open.
    fn find_meta(&self, p: &Path) -> (String, bool) {
        if let Some(m) = self.files.find.meta.borrow().get(p) {
            return m.clone();
        }
        let md = p.extension().is_some_and(|e| e == "md");
        let m = match md {
            true => (notes::load(p).map(|n| n.title).unwrap_or_default(), true),
            false => (p.file_name().unwrap_or_default().to_string_lossy().into_owned(), crate::docs::tree::is_text(p)),
        };
        self.files.find.meta.borrow_mut().insert(p.to_path_buf(), m.clone());
        m
    }

    fn find_item(&self, r: Row) -> Option<FindItem> {
        let (title, text) = self.find_meta(&r.path);
        (text || !r.snippets.is_empty()).then_some(FindItem { path: r.path, title, at: r.at, snippets: r.snippets })
    }
}
