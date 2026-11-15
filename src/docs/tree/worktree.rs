//! The worktree of an agent session, `.worktrees/<id>` (SESSIONS-21): its files (FILES-26), and the
//! files that differ from the merge base of its branch and `main` (FILES-27).
use super::*;
use crate::setup::sync::git;

/// Git in a worktree gets this long.
const LIMIT: Duration = Duration::from_secs(5);
/// Paths as they are, not quoted.
const PLAIN: [&str; 2] = ["-c", "core.quotepath=off"];

/// The folder of the worktree of agent session `n` (from 0), from the repo root.
pub fn rel(n: usize) -> PathBuf {
    Path::new(".worktrees").join((n + 1).to_string())
}

/// The worktree of agent session `n`, when git made it: it has a `.git` file.
pub fn of(root: &Path, n: usize) -> Option<PathBuf> {
    let rel = rel(n);
    root.join(&rel).join(".git").exists().then_some(rel)
}

/// `.worktrees/3/src/main.rs` → (`.worktrees/3`, `src/main.rs`); `None` outside a worktree.
pub fn split(rel: &str) -> Option<(&str, &str)> {
    let id = rel.strip_prefix(".worktrees/")?.split('/').next()?;
    let end = ".worktrees/".len() + id.len();
    Some((&rel[..end], rel.get(end + 1..).filter(|r| !r.is_empty())?))
}

/// `.worktrees/3/src/main.rs` → `worktree 3 · src/main.rs` (FILES-34); any other path as it is.
pub fn label(rel: &str) -> String {
    match split(rel) {
        Some((wt, rest)) => format!("worktree {} · {rest}", &wt[".worktrees/".len()..]),
        None => rel.to_string(),
    }
}

/// The commit of `main` that the branch started from, or the last `main` merged into it.
fn base(wt: &Path) -> Option<String> {
    git(wt, &["merge-base", "HEAD", "main"], LIMIT).ok().filter(|b| !b.is_empty())
}

/// The files that git does not track and does not ignore, from the worktree.
fn untracked(wt: &Path) -> Vec<String> {
    let out = git(wt, &[PLAIN[0], PLAIN[1], "ls-files", "--others", "--exclude-standard"], LIMIT).unwrap_or_default();
    out.lines().filter(|l| !l.is_empty()).map(str::to_string).collect()
}

/// A file that differs from the base, from the worktree.
pub struct Change {
    pub rel: PathBuf,
    pub deleted: bool,
}

/// The files that differ from the base: changed, new (also not tracked) and deleted, in path order.
pub fn changes(wt: &Path) -> Vec<Change> {
    let Some(base) = base(wt) else { return Vec::new() };
    let tracked = git(wt, &[PLAIN[0], PLAIN[1], "diff", "--name-status", "--no-renames", &base], LIMIT).unwrap_or_default();
    let parse = |l: &str| l.split_once('\t').map(|(st, p)| Change { rel: p.into(), deleted: st.starts_with('D') });
    let new = untracked(wt).into_iter().map(|p| Change { rel: p.into(), deleted: false });
    let mut out: Vec<Change> = tracked.lines().filter_map(parse).chain(new).collect();
    out.sort_by(|a, b| a.rel.cmp(&b.rel));
    out.dedup_by(|a, b| a.rel == b.rel);
    out
}

/// The diff of the worktree against the base, with the text of each new file, up to `max` chars.
pub fn diff(wt: &Path, max: usize) -> String {
    let Some(base) = base(wt) else { return String::new() };
    let mut out = git(wt, &[PLAIN[0], PLAIN[1], "diff", "--no-renames", &base], LIMIT).unwrap_or_default();
    for f in untracked(wt) {
        let text = std::fs::read_to_string(wt.join(&f)).unwrap_or_default();
        out.push_str(&format!("\n--- new file {f}\n{text}"));
        if out.len() >= max {
            break;
        }
    }
    out.chars().take(max).collect()
}

impl Tree {
    /// Fill a worktree view: every text file of the worktree, or the files that differ from the base.
    /// The paths stay relative to `root`, so a row opens the file of the worktree.
    pub(super) fn scan_worktree(&mut self, root: &Path) {
        self.files.clear();
        self.dirs.clear();
        self.deleted.clear();
        if let Some(wt) = self.worktree.clone() {
            let dir = root.join(&wt);
            if self.view == View::Worktree {
                self.files = text_files(&dir).into_iter().map(|p| wt.join(p)).collect();
            } else {
                for c in changes(&dir) {
                    let rel = wt.join(&c.rel);
                    if c.deleted {
                        self.deleted.insert(rel.clone());
                    }
                    self.files.push(rel);
                }
            }
        }
        self.root = root.to_path_buf();
        self.scanned = Some(Instant::now());
    }

    /// The rows of a worktree view: the folders and files in the worktree, without the folder of the
    /// worktree itself.
    pub(super) fn worktree_rows(&self) -> Vec<Row> {
        let Some(wt) = self.worktree.as_deref() else { return Vec::new() };
        let files: Vec<PathBuf> = self.files.iter().filter_map(|f| f.strip_prefix(wt).ok().map(Path::to_path_buf)).collect();
        self.tree_rows(&files, &[], wt, 0)
    }
}

/// The text files of a worktree, from it, in path order; its own `.gitignore` holds.
fn text_files(dir: &Path) -> Vec<PathBuf> {
    let walk = ignore::WalkBuilder::new(dir).hidden(true).git_ignore(true).git_global(true).git_exclude(true).require_git(false).parents(false).build();
    let mut out: Vec<PathBuf> = walk
        .flatten()
        .filter(|e| e.file_type().is_some_and(|t| t.is_file()) && is_text(e.path()))
        .filter_map(|e| e.path().strip_prefix(dir).ok().map(Path::to_path_buf))
        .collect();
    out.sort();
    out
}
