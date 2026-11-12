//! The private notes repo: one folder for each machine and repo root, kept in sync with git.
//!
//! Layout of the notes repo:
//!
//! ```text
//! <machine>/<path of the repo root from home>/   the notes of that repo on that machine
//! <machine>/~/                                   the scratchpad (outside a repo)
//! <machine>/_/<absolute path>/                   a repo root outside home
//! ```
//!
//! Each folder has `.zendo-location.toml` with the machine, the path and the git origin of the
//! repo, so that `zendo notes consolidate` can find the folders of one project.

mod git;
mod syncer;

pub use git::*;
pub use syncer::*;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const LOCATION_FILE: &str = ".zendo-location.toml";
pub const DEFAULT_SECONDS: u64 = 15;

/// What a location folder holds about itself.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub machine: String,
    /// The repo root on that machine.
    pub path: String,
    /// The git origin of the repo, normalized: `github.com/owner/repo`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// A short, stable name for this machine. macOS changes `hostname` with the network,
/// so it uses the local host name. Only `a-z`, `0-9` and `-`.
pub fn machine_name() -> String {
    let try_cmd = |cmd: &str, args: &[&str]| {
        Command::new(cmd).args(args).stderr(Stdio::null()).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    let raw = (cfg!(target_os = "macos").then(|| try_cmd("scutil", &["--get", "LocalHostName"])).flatten())
        .or_else(|| try_cmd("hostname", &["-s"]))
        .or_else(|| std::fs::read_to_string("/etc/hostname").ok())
        .unwrap_or_default();
    clean_name(&raw)
}

pub fn clean_name(raw: &str) -> String {
    let short = raw.trim().split('.').next().unwrap_or("");
    let mut s: String = short.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    let s = s.trim_matches('-').to_string();
    if s.is_empty() { "machine".into() } else { s }
}

/// The folder of the notes of `root` on `machine`, inside the notes repo.
pub fn location_dir(repo: &Path, machine: &str, root: &Path, home: Option<&Path>) -> PathBuf {
    let base = repo.join(machine);
    match home.and_then(|h| root.strip_prefix(h).ok()) {
        Some(rel) if rel.as_os_str().is_empty() => base.join("~"),
        Some(rel) => base.join(rel),
        None => base.join("_").join(root.strip_prefix("/").unwrap_or(root)),
    }
}

/// `git@github.com:a/b.git`, `https://github.com/a/b`, `ssh://git@github.com/a/b.git` → `github.com/a/b`.
pub fn normalize_origin(url: &str) -> String {
    let mut s = url.trim().trim_end_matches('/').trim_end_matches(".git").to_string();
    for p in ["https://", "http://", "ssh://", "git://"] {
        if let Some(r) = s.strip_prefix(p) {
            s = r.to_string();
        }
    }
    if let Some((user, rest)) = s.split_once('@')
        && !user.contains('/') {
            s = rest.to_string();
        }
    // scp form: host:owner/repo
    if let Some((host, path)) = s.split_once(':')
        && !host.contains('/') && !path.starts_with('/') {
            s = format!("{host}/{}", path.trim_start_matches(|c: char| c.is_ascii_digit()).trim_start_matches('/'));
        }
    s.to_lowercase()
}

pub fn origin_of(root: &Path) -> Option<String> {
    let out = Command::new("git").arg("-C").arg(root).args(["remote", "get-url", "origin"]).stderr(Stdio::null()).output().ok()?;
    let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !url.is_empty()).then(|| normalize_origin(&url))
}

/// Make the location folder and write its location file.
pub fn prepare(dir: &Path, loc: &Location) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let text = toml::to_string(loc).unwrap_or_default();
    let file = dir.join(LOCATION_FILE);
    if std::fs::read_to_string(&file).ok().as_deref() != Some(text.as_str()) {
        std::fs::write(file, text)?;
    }
    Ok(())
}

/// Every location folder of the notes repo.
pub fn locations(repo: &Path) -> Vec<(PathBuf, Location)> {
    let mut out = Vec::new();
    let walk = ignore::WalkBuilder::new(repo).hidden(false).git_ignore(true).require_git(false).filter_entry(|e| e.file_name() != ".git").build();
    for e in walk.flatten() {
        if e.file_name() == LOCATION_FILE
            && let Some(loc) = std::fs::read_to_string(e.path()).ok().and_then(|t| toml::from_str::<Location>(&t).ok()) {
                out.push((e.path().parent().unwrap_or(repo).to_path_buf(), loc));
            }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// The markdown files directly in a folder.
pub fn notes_in(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "md"))
        .collect();
    v.sort();
    v
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
