//! Git for the notes repo: no prompts, a time limit, commit, pull and push.
use super::*;

pub fn is_work_tree(dir: &Path) -> bool {
    git(dir, &["rev-parse", "--is-inside-work-tree"], Duration::from_secs(5)).is_ok_and(|o| o == "true")
}

/// Run git in `repo` with no prompts, and kill it after `limit`.
pub fn git(repo: &Path, args: &[&str], limit: Duration) -> Result<String, String> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_SSH_COMMAND", std::env::var("GIT_SSH_COMMAND").unwrap_or_else(|_| "ssh -o BatchMode=yes -o ConnectTimeout=5".into()))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("git: {e}"))?;
    let end = Instant::now() + limit;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() >= end => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("git {} took more than {} s", args.first().copied().unwrap_or(""), limit.as_secs()));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(e.to_string()),
        }
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let text = |b: &[u8]| String::from_utf8_lossy(b).trim().to_string();
    if out.status.success() { Ok(text(&out.stdout)) } else { Err(first_line(&text(&out.stderr))) }
}

pub(super) fn first_line(s: &str) -> String {
    s.lines().find(|l| !l.trim().is_empty() && !l.starts_with("hint:")).unwrap_or(s).to_string()
}

/// Commit the changes inside `dir`. Returns true when it made a commit.
pub fn commit(repo: &Path, dir: &Path, message: &str) -> Result<bool, String> {
    let rel = dir.strip_prefix(repo).unwrap_or(dir).to_string_lossy().into_owned();
    let rel = if rel.is_empty() { ".".to_string() } else { rel };
    let t = Duration::from_secs(10);
    git(repo, &["add", "-A", "--", &rel], t)?;
    if git(repo, &["diff", "--cached", "--quiet", "--", &rel], t).is_ok() {
        return Ok(false);
    }
    git(repo, &["commit", "-q", "-m", message, "--", &rel], t)?;
    Ok(true)
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    /// Nothing pushed yet in this run.
    Starting,
    /// Everything is pushed.
    Synced,
    /// Commits wait for the network.
    Offline(String),
    /// No remote: commits stay local.
    Local,
    /// `git pull --rebase` had a conflict. Sync stopped; the files are as they were.
    Conflict(String),
    /// Another git problem.
    Failed(String),
}

impl State {
    pub fn label(&self) -> Option<String> {
        match self {
            State::Starting | State::Synced => None,
            State::Offline(_) => Some("notes: offline, will sync".into()),
            State::Local => Some("notes: no remote, local only".into()),
            State::Conflict(_) => Some("notes: sync stopped (conflict), see zendo doctor".into()),
            State::Failed(e) => Some(format!("notes: {e}")),
        }
    }
}

/// Pull with rebase, then push. A conflict aborts the rebase and stops.
pub fn pull_push(repo: &Path) -> State {
    let net = Duration::from_secs(30);
    if git(repo, &["remote"], Duration::from_secs(5)).is_ok_and(|r| r.is_empty()) {
        return State::Local;
    }
    // `symbolic-ref` also names the branch of a repo without commits.
    let branch = git(repo, &["symbolic-ref", "--short", "HEAD"], Duration::from_secs(5)).unwrap_or_else(|_| "main".into());
    // Empty output: the remote does not have the branch yet (a new notes repo).
    let heads = match git(repo, &["ls-remote", "--heads", "origin", &branch], net) {
        Ok(h) => h,
        Err(e) => return State::Offline(e),
    };
    if !heads.is_empty()
        && let Err(e) = git(repo, &["pull", "-q", "--rebase", "--autostash", "origin", &branch], net) {
            if repo.join(".git/rebase-merge").exists() || repo.join(".git/rebase-apply").exists() {
                let _ = git(repo, &["rebase", "--abort"], Duration::from_secs(10));
                return State::Conflict(e);
            }
            return State::Offline(e);
        }
    if git(repo, &["rev-parse", "-q", "--verify", "HEAD"], Duration::from_secs(5)).is_err() {
        return State::Synced; // no commits yet: nothing to push
    }
    match git(repo, &["push", "-q", "-u", "origin", &branch], net) {
        Ok(_) => State::Synced,
        Err(e) => State::Offline(e),
    }
}

/// True when the branch has commits that the remote does not have, or no upstream yet.
pub(super) fn ahead(repo: &Path) -> bool {
    match git(repo, &["rev-list", "--count", "@{u}..HEAD"], Duration::from_secs(5)) {
        Ok(n) => n != "0",
        Err(_) => git(repo, &["rev-parse", "HEAD"], Duration::from_secs(5)).is_ok(),
    }
}
