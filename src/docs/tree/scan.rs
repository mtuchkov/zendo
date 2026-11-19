//! Walking the repository: the files of the view, and the folders.
use super::*;

impl Tree {
    /// Walk the repo on a thread; `.gitignore`, `.git/info/exclude`, the global gitignore and hidden
    /// files are skipped. The worktree changed view asks git, on its thread (ADR 0009, IO-1).
    pub fn scan(&mut self, root: &Path) {
        if self.view == View::WorktreeChanged {
            return self.scan_worktree(root);
        }
        self.walk_view(root, root);
    }

    /// Walk only `under`, a folder in `root`; paths stay relative to `root`. Outside a repo, zendo
    /// walks only the notes folder and not the whole home folder. That walk lists every `.md`.
    pub fn scan_in(&mut self, root: &Path, under: &Path) {
        self.walk_view(root, under);
    }

    /// Older than the scan interval, or never filled; not while git reads the changes (ADR 0009).
    pub fn stale(&self) -> bool {
        !self.scanning() && !self.walking() && self.scanned.is_none_or(|t| t.elapsed() > Duration::from_secs(5))
    }

    /// The files whose path has every word of `q`, in any case; more words in the file name come first.
    pub(super) fn found(&self, q: &str) -> Vec<Row> {
        if self.find_regex {
            let Ok(re) = regex::RegexBuilder::new(q).case_insensitive(true).build() else { return Vec::new() };
            let rows = self.files.iter().filter(|f| re.is_match(&f.to_string_lossy()));
            return rows.map(|f| Row { rel: f.clone(), name: f.display().to_string(), depth: 0, kind: Kind::File, section: false }).collect();
        }
        let words: Vec<String> = q.split_whitespace().map(str::to_lowercase).collect();
        let mut hits: Vec<(std::cmp::Reverse<usize>, &PathBuf)> = self
            .files
            .iter()
            .filter_map(|f| {
                let path = f.to_string_lossy().to_lowercase();
                let name = f.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
                words.iter().all(|w| path.contains(w.as_str())).then(|| (std::cmp::Reverse(words.iter().filter(|w| name.contains(w.as_str())).count()), f))
            })
            .collect();
        hits.sort();
        hits.into_iter().map(|(_, f)| Row { rel: f.clone(), name: f.display().to_string(), depth: 0, kind: Kind::File, section: false }).collect()
    }
}

/// The text view lists a file up to this size.
const TEXT_MAX: u64 = 2 << 20;

/// A text file: not too large, and its head is UTF-8 without a NUL byte.
pub fn is_text(p: &Path) -> bool {
    use std::io::Read;
    let small = std::fs::metadata(p).is_ok_and(|m| m.len() <= TEXT_MAX);
    let mut head = Vec::with_capacity(4096);
    let read = small && std::fs::File::open(p).and_then(|f| f.take(4096).read_to_end(&mut head)).is_ok();
    // A char may be cut at the end of the head: that is no error.
    read && !head.contains(&0) && std::str::from_utf8(&head).map_or_else(|e| e.error_len().is_none(), |_| true)
}
