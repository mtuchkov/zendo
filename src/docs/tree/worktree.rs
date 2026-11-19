//! The worktree of an agent session, `.worktrees/<id>` (SESSIONS-21): its files (FILES-26), and the
//! files that differ from the merge base of its branch and `main` (FILES-27).
use super::*;
use crate::setup::sync::git;

/// Git in a worktree gets this long.
const LIMIT: Duration = Duration::from_secs(5);
/// Paths as they are, not quoted.
const PLAIN: [&str; 2] = ["-c", "core.quotepath=off"];

/// The folder of the worktree of agent session `n`, from the repo root: the id of the session is the
/// number of its worktree (SESSIONS-23).
pub fn rel(n: usize) -> PathBuf {
    Path::new(".worktrees").join(n.to_string())
}

/// The worktree of agent session `n`, when git made it: it has a `.git` file. Agent session 0 works
/// in the main checkout and has none (SESSIONS-23).
pub fn of(root: &Path, n: usize) -> Option<PathBuf> {
    if n == 0 {
        return None;
    }
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

/// The branch that the changes of `dir` count from: `main` for a worktree; `origin/main` for the main
/// checkout, where it is there, so that the commits that origin does not have count too (FILES-81).
fn against(dir: &Path) -> &'static str {
    let main_checkout = dir.join(".git").is_dir();
    let origin = || git(dir, &["rev-parse", "--verify", "-q", "refs/remotes/origin/main"], LIMIT).is_ok_and(|s| !s.is_empty());
    if main_checkout && origin() { "origin/main" } else { "main" }
}

/// The commit of `main` (or of `origin/main`, see `against`) that the branch started from, or the
/// last one merged into it. Git says nothing when they have no commit in common.
fn base(wt: &Path) -> Result<String, String> {
    let to = against(wt);
    let none = || format!("the branch has no commit in common with {to}");
    match git(wt, &["merge-base", "HEAD", to], LIMIT) {
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
    /// The folder whose changes the changed view shows, from the repo root: the worktree of the
    /// session, or the main checkout (the empty path) for agent session 0 (FILES-81).
    pub fn changes_dir(&self) -> Option<PathBuf> {
        self.worktree.clone().or_else(|| self.main_checkout.then(PathBuf::new))
    }

    /// Fill the worktree changed view: the files that differ from the base. Git reads the changes
    /// on a thread, and `poll` takes them (ADR 0009). The paths stay relative to `root`, so a row
    /// opens the file of the worktree.
    pub(super) fn scan_worktree(&mut self, root: &Path) {
        self.dirs.clear();
        match self.changes_dir() {
            Some(wt) => self.scan_changes(root, wt),
            None => {
                self.drop_changes();
                self.deleted.clear();
                self.files.clear();
            }
        }
        self.root = root.to_path_buf();
        self.scanned = Some(Instant::now());
    }

    /// The rows of a worktree view: the folders and files in the worktree, without the folder of the
    /// worktree itself.
    pub(super) fn worktree_rows(&self) -> Vec<Row> {
        let wt = if self.view == View::WorktreeChanged { self.changes_dir() } else { self.worktree.clone() };
        let Some(wt) = wt.as_deref() else { return Vec::new() };
        let files: Vec<PathBuf> = self.files.iter().filter_map(|f| f.strip_prefix(wt).ok().map(Path::to_path_buf)).collect();
        self.tree_rows(&files, &[], wt, 0)
    }
}
