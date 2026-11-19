//! `@` in the chat input: file suggestions under the input. A name matches by its start, by a
//! part, or with a typo or two (edit distance).
use super::Composer;
use super::super::*;

/// The open suggestions: what follows `@`, the matches, and the selected one.
#[derive(Clone, Debug, Default)]
pub struct AtPick {
    pub query: String,
    pub hits: Vec<PathBuf>,
    pub sel: usize,
    /// The column of the `@` in the input line.
    pub at: usize,
    /// `ctrl-t`: only templates.
    pub templates: bool,
}

/// The suggestions show 4 at a time.
pub const AT_ROWS: usize = 4;

/// The edit distance of two strings: inserts, deletes and changes of one char.
pub fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let first: Vec<usize> = (0..=b.len()).collect();
    let last = a.chars().enumerate().fold(first, |row, (i, ca)| {
        let start = i + 1;
        let tail = b.iter().enumerate().scan(start, |left, (j, cb)| {
            let here = (row[j] + usize::from(ca != *cb)).min(*left + 1).min(row[j + 1] + 1);
            *left = here;
            Some(here)
        });
        std::iter::once(start).chain(tail).collect()
    });
    last[b.len()]
}

/// How well a file matches: lower is better; `None` when it does not match.
fn score(rel: &Path, q: &str) -> Option<usize> {
    let path = rel.to_string_lossy().to_lowercase();
    let name = rel.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    let stem = rel.file_stem().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    if name.starts_with(q) {
        return Some(0);
    }
    if name.contains(q) {
        return Some(1);
    }
    if path.contains(q) {
        return Some(2);
    }
    let head: String = name.chars().take(q.chars().count()).collect();
    let d = distance(q, &head).min(distance(q, &stem));
    (d <= (q.chars().count() / 3).max(1)).then_some(3 + d)
}

/// The files that match `q`, best first.
pub fn rank(files: &[PathBuf], q: &str) -> Vec<PathBuf> {
    let q = q.to_lowercase();
    let mut hits: Vec<(usize, usize, &PathBuf)> = files.iter().filter_map(|f| score(f, &q).map(|s| (s, f.as_os_str().len(), f))).collect();
    hits.sort();
    hits.into_iter().take(50).map(|(_, _, f)| f.clone()).collect()
}

/// The templates: the ones that project-structure.toml names, and the files of `templates` folders.
pub fn template_files(named: &[PathBuf], files: &[PathBuf]) -> Vec<PathBuf> {
    let in_folder = files.iter().filter(|f| f.parent().is_some_and(|p| p.components().any(|c| c.as_os_str() == "templates")));
    let mut all: Vec<PathBuf> = named.iter().chain(in_folder).cloned().collect();
    all.sort();
    all.dedup();
    all
}

/// What a key did in the open suggestions.
#[derive(Debug, PartialEq)]
pub enum AtKey {
    /// The key is not one of the suggestions.
    Unused,
    Used,
    /// The user took a file, from the root of the repository; true for a template.
    Taken(PathBuf, bool),
}

impl Composer {
    /// The `@word` before the cursor: its column and the word.
    fn at_token(&self) -> Option<(usize, String)> {
        let before: Vec<char> = self.input.lines[self.input.row].chars().take(self.input.col).collect();
        let at = before.iter().rposition(|c| *c == '@')?;
        let word: String = before[at + 1..].iter().collect();
        let starts = at == 0 || before[at - 1].is_whitespace();
        (starts && !word.contains(char::is_whitespace)).then_some((at, word))
    }

    /// After a key in the chat input: open, update or close the suggestions. `all` is the files of
    /// the repository, when a worker read them (IO-1); `named` the templates that
    /// project-structure.toml names. Without the files, the suggestions wait for them (IO-6).
    pub fn update_at(&mut self, all: Option<Vec<PathBuf>>, named: &[PathBuf]) {
        let Some((at, query)) = self.at_token() else {
            self.at = None;
            self.at_files = None;
            self.at_templates = false;
            return;
        };
        if self.at.as_ref().is_some_and(|p| p.at == at && p.query == query) && self.at_files.is_some() {
            return;
        }
        let templates = self.at.as_ref().map_or(self.at_templates, |p| p.templates);
        if self.at_files.is_none() {
            self.at_files = all.map(|all| if templates { template_files(named, &all) } else { all });
        }
        let hits = self.at_files.as_ref().map(|files| rank(files, &query)).unwrap_or_default();
        self.at = Some(AtPick { query, hits, sel: 0, at, templates });
    }

    /// The keys of the open suggestions: the arrows choose, tab or enter takes one, esc closes.
    /// The suggestions are open and wait for the files of the repository.
    pub fn at_waits(&self) -> bool {
        self.at.is_some() && self.at_files.is_none()
    }

    pub fn key_at(&mut self, k: KeyEvent) -> AtKey {
        let Some(p) = self.at.as_mut() else { return AtKey::Unused };
        match k.code {
            KeyCode::Down => p.sel = (p.sel + 1).min(p.hits.len().saturating_sub(1)),
            KeyCode::Up => p.sel = p.sel.saturating_sub(1),
            KeyCode::Esc => self.at = None,
            KeyCode::Tab | KeyCode::Enter => return self.take_at().map_or(AtKey::Used, |(rel, templates)| AtKey::Taken(rel, templates)),
            _ => return AtKey::Unused,
        }
        AtKey::Used
    }

    /// Take the selected file: `@word` leaves the input, and the suggestions close.
    fn take_at(&mut self) -> Option<(PathBuf, bool)> {
        let p = self.at.take()?;
        let rel = p.hits.get(p.sel).cloned()?;
        let line = &mut self.input.lines[self.input.row];
        *line = line.chars().take(p.at).chain(line.chars().skip(self.input.col)).collect();
        self.input.col = p.at;
        self.at_files = None;
        self.at_templates = false;
        Some((rel, p.templates))
    }

    /// `ctrl-t` in the chat: an `@` whose suggestions show only templates.
    pub fn pick_template(&mut self) {
        let before = self.input.lines[self.input.row].chars().nth(self.input.col.wrapping_sub(1));
        if self.input.col > 0 && before.is_some_and(|c| !c.is_whitespace()) {
            self.input.type_str(" ");
        }
        self.at = None;
        self.at_files = None;
        self.at_templates = true;
        self.input.type_str("@");
    }
}

#[cfg(test)]
#[path = "at_tests.rs"]
mod tests;
