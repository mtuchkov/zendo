//! Walking the repository: the files of the view, and the folders.
use super::*;

impl Tree {
    /// Walk the repo. `.gitignore`, `.git/info/exclude`, the global gitignore and hidden files are skipped.
    pub fn scan(&mut self, root: &Path) {
        self.scan_in(root, root);
    }

    /// Walk only `under`, a folder in `root`; paths stay relative to `root`. Outside a repo, zendo
    /// walks only the notes folder and not the whole home folder. That walk lists every `.md`.
    pub fn scan_in(&mut self, root: &Path, under: &Path) {
        let mut files = Vec::new();
        let all = under != root;
        let mut dirs = Vec::new();
        let walk = ignore::WalkBuilder::new(under).hidden(true).git_ignore(true).git_global(true).git_exclude(true).require_git(false).parents(true).build();
        for entry in walk.flatten() {
            let p = entry.path();
            if self.view == View::Folders
                && entry.file_type().is_some_and(|t| t.is_dir())
                && let Ok(rel) = p.strip_prefix(root)
                && !rel.as_os_str().is_empty()
            {
                dirs.push(rel.to_path_buf());
            }
            let md = p.extension().is_some_and(|e| e == "md");
            if !entry.file_type().is_some_and(|t| t.is_file()) || !(md || self.view == View::Text && is_text(p)) {
                continue;
            }
            let Ok(rel) = p.strip_prefix(root) else { continue };
            if all || self.view != View::Docs || self.docs.is_match(rel) {
                files.push(rel.to_path_buf());
            }
        }
        files.sort();
        self.files = files;
        self.dirs = dirs;
        self.root = root.to_path_buf();
        self.scanned = Some(Instant::now());
    }

    pub fn stale(&self) -> bool {
        self.scanned.is_none_or(|t| t.elapsed() > Duration::from_secs(5))
    }

    /// The files whose path has every word of `q`, in any case; more words in the file name come first.
    pub(super) fn found(&self, q: &str) -> Vec<Row> {
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
