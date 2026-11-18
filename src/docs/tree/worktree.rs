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

/// `.worktrees/3/src/main.rs` → `src/main.rs`, for the top line (FILES-34); any other path as it is.
pub fn in_worktree(rel: &str) -> String {
    split(rel).map_or_else(|| rel.to_string(), |(_, rest)| rest.to_string())
}

/// The commit of `main` that the branch started from, or the last `main` merged into it. Git says
/// nothing when the branch and `main` have no commit in common.
fn base(wt: &Path) -> Result<String, String> {
    let none = || "the branch has no commit in common with main".to_string();
    match git(wt, &["merge-base", "HEAD", "main"], LIMIT) {
        Ok(b) if !b.is_empty() => Ok(b),
        Err(e) if !e.is_empty() => Err(e),
        _ => Err(none()),
    }
}

/// The files that git does not track and does not ignore, from the worktree.
fn untracked(wt: &Path) -> Result<Vec<String>, String> {
    let out = git(wt, &[PLAIN[0], PLAIN[1], "ls-files", "--others", "--exclude-standard"], LIMIT)?;
    Ok(out.lines().filter(|l| !l.is_empty()).map(str::to_string).collect())
}

/// A file that differs from the base, from the worktree.
pub struct Change {
    pub rel: PathBuf,
    pub deleted: bool,
}

/// The files that differ from the base: changed, new (also not tracked) and deleted, in path order;
/// or the error of git.
pub fn changes(wt: &Path) -> Result<Vec<Change>, String> {
    let base = base(wt)?;
    let tracked = git(wt, &[PLAIN[0], PLAIN[1], "diff", "--name-status", "--no-renames", &base], LIMIT)?;
    let parse = |l: &str| l.split_once('\t').map(|(st, p)| Change { rel: p.into(), deleted: st.starts_with('D') });
    let new = untracked(wt)?.into_iter().map(|p| Change { rel: p.into(), deleted: false });
    let mut out: Vec<Change> = tracked.lines().filter_map(parse).chain(new).collect();
    out.sort_by(|a, b| a.rel.cmp(&b.rel));
    out.dedup_by(|a, b| a.rel == b.rel);
    Ok(out)
}

/// The diff of the worktree against the base, with the text of each new file, up to `max` chars.
pub fn diff(wt: &Path, max: usize) -> String {
    let Ok(base) = base(wt) else { return String::new() };
    let mut out = git(wt, &[PLAIN[0], PLAIN[1], "diff", "--no-renames", &base], LIMIT).unwrap_or_default();
    for f in untracked(wt).unwrap_or_default() {
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
    /// Git reads the changes on a thread, and `poll` takes them (ADR 0009). The paths stay relative
    /// to `root`, so a row opens the file of the worktree.
    pub(super) fn scan_worktree(&mut self, root: &Path) {
        self.dirs.clear();
        match self.worktree.clone() {
            Some(wt) if self.view == View::WorktreeChanged => self.scan_changes(root, wt),
            wt => {
                self.drop_changes();
                self.deleted.clear();
                self.files = wt.map(|wt| text_files(&root.join(&wt)).into_iter().map(|p| wt.join(p)).collect()).unwrap_or_default();
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
