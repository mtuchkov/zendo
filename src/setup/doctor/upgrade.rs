//! Versions and upgrades: from GitHub or from a local clone.
use super::*;

/// The commit this binary was built from: put in by build.rs, else the one
/// `cargo install --git` recorded.
pub fn installed_commit() -> Option<String> {
    if let Some(c) = option_env!("ZENDO_COMMIT") {
        return Some(c.to_string());
    }
    recorded_commit()
}

pub(super) fn built_dirty() -> bool {
    option_env!("ZENDO_DIRTY") == Some("1")
}

/// A folder is the zendo repository: a git work tree with the zendo package and its mock agent.
pub fn is_zendo_repo(dir: &Path) -> bool {
    let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
    (dir.join(".git").exists()) && cargo.lines().any(|l| l.trim() == "name = \"zendo\"") && (dir.join("src/bin/zendo-mock-agent/main.rs").exists() || dir.join("src/bin/zendo-mock-agent.rs").exists())
}

/// The clone this binary was built from, when it was built from your own copy
/// (not from cargo's checkout of GitHub) and the copy is still there.
pub fn local_clone() -> Option<PathBuf> {
    let src = PathBuf::from(option_env!("ZENDO_SRC")?);
    let from_cargo = src.components().any(|c| c.as_os_str() == "checkouts") && src.to_string_lossy().contains(".cargo");
    (!from_cargo && is_zendo_repo(&src)).then_some(src)
}

pub(super) fn recorded_commit() -> Option<String> {
    let home = std::env::var_os("CARGO_HOME").map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cargo")))?;
    let roots = [std::env::var_os("CARGO_INSTALL_ROOT").map(PathBuf::from), Some(home)];
    for root in roots.into_iter().flatten() {
        let Ok(text) = std::fs::read_to_string(root.join(".crates2.json")) else { continue };
        let Ok(j) = serde_json::from_str::<serde_json::Value>(&text) else { continue };
        for key in j["installs"].as_object().into_iter().flat_map(|m| m.keys()) {
            if let Some(src) = key.strip_prefix("zendo ")
                && let Some(sha) = src.rsplit('#').next().filter(|_| src.contains("#")) {
                    return Some(sha.trim_end_matches(')').to_string());
                }
        }
    }
    None
}

pub(super) async fn remote_head(repo: &str, env: &BTreeMap<String, String>, secs: u64) -> Option<String> {
    let (ok, out) = run("git", &["ls-remote", repo, "refs/heads/main"], env, secs).await?;
    ok.then(|| out.split_whitespace().next().map(str::to_string)).flatten()
}

pub(super) fn same_commit(a: &str, b: &str) -> bool {
    !a.is_empty() && !b.is_empty() && (a.starts_with(b) || b.starts_with(a))
}

pub(super) fn repos() -> Vec<String> {
    match std::env::var("ZENDO_REPO") {
        Ok(r) => vec![r],
        Err(_) => vec![HTTPS_REPO.into(), SSH_REPO.into()],
    }
}

/// Git and cargo settings for builds from GitHub: no prompts, a kept build folder.
pub(super) fn git_env() -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    env.insert("CARGO_NET_GIT_FETCH_WITH_CLI".to_string(), "true".to_string());
    if std::env::var("CARGO_TARGET_DIR").is_err() {
        env.insert("CARGO_TARGET_DIR".into(), config::build_dir().display().to_string());
    }
    env.insert("GIT_TERMINAL_PROMPT".to_string(), "0".to_string());
    if std::env::var("GIT_SSH_COMMAND").is_err() {
        env.insert("GIT_SSH_COMMAND".into(), "ssh -o BatchMode=yes -o ConnectTimeout=5 -o StrictHostKeyChecking=accept-new".into());
    }
    env
}

/// `zendo --version`: the version, the commit, uncommitted changes, and the local copy it came from.
pub fn version_line() -> String {
    let Some(c) = installed_commit() else { return format!("zendo {}", env!("CARGO_PKG_VERSION")) };
    let mut parts = vec![c[..7.min(c.len())].to_string()];
    if built_dirty() {
        parts[0].push_str("+changes");
    }
    if let Some(src) = local_clone() {
        let home = std::env::var("HOME").unwrap_or_default();
        let shown = src.display().to_string();
        let shown = match shown.strip_prefix(&home).filter(|_| !home.is_empty()) {
            Some(rest) => format!("~{rest}"),
            None => shown,
        };
        parts.push(format!("local: {shown}"));
    }
    format!("zendo {} ({})", env!("CARGO_PKG_VERSION"), parts.join(", "))
}

/// The version for the help: only the version, and the details of `version_line` only for a build
/// with uncommitted changes (PANES-31).
pub fn help_version() -> String {
    help_version_for(built_dirty())
}

pub(super) fn help_version_for(dirty: bool) -> String {
    if dirty { version_line() } else { format!("zendo {}", env!("CARGO_PKG_VERSION")) }
}

/// For a build from your clone: the clone has `head` and it is in the history of the build.
pub(super) fn clone_has(src: &Path, head: &str, built: &str) -> bool {
    let ok = |args: &[&str]| std::process::Command::new("git").arg("-C").arg(src).args(args).output().is_ok_and(|o| o.status.success());
    ok(&["cat-file", "-e", &format!("{head}^{{commit}}")]) && ok(&["merge-base", "--is-ancestor", head, built])
}

/// On start: is there a newer commit on main? Asks GitHub at most every 12 hours and
/// never waits longer than 5 s. `None` for builds not installed from git.
pub async fn update_available() -> Option<String> {
    if std::env::var("ZENDO_NO_UPDATE_CHECK").is_ok() {
        return None;
    }
    let mine = installed_commit()?;
    let cache = config::data_dir().join("update.json");
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    let cached: Option<(u64, String)> = std::fs::read_to_string(&cache)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|j| Some((j["checked"].as_u64()?, j["head"].as_str()?.to_string())));
    let head = match cached {
        Some((at, head)) if now.saturating_sub(at) < UPDATE_EVERY.as_secs() => head,
        _ => {
            let env = git_env();
            let mut found = None;
            for r in repos() {
                if let Some(h) = remote_head(&r, &env, 5).await {
                    found = Some(h);
                    break;
                }
            }
            let head = found?;
            let _ = std::fs::create_dir_all(config::data_dir());
            let _ = std::fs::write(&cache, serde_json::json!({ "checked": now, "head": head }).to_string());
            head
        }
    };
    if same_commit(&mine, &head) {
        return None;
    }
    // Built from your clone, and main is already in it (you may be ahead): nothing to suggest.
    if let Some(src) = local_clone()
        && clone_has(&src, &head, &mine) {
            return None;
        }
    Some(head[..7.min(head.len())].to_string())
}

/// Built from your clone: pull it (fast-forward only) and build it again.
pub(super) async fn upgrade_clone(src: &Path, force: bool, o: &mut Out) -> bool {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("zendo"));
    let exe = PathBuf::from(exe.to_string_lossy().trim_end_matches(" (deleted)").to_string());
    let env = git_env();
    let git = |args: &[&str]| std::process::Command::new("git").arg("-C").arg(src).args(args).envs(&env).output();
    o.ok("clone", src.display().to_string());
    match git(&["pull", "--ff-only", "--quiet"]) {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            o.fail("pull", "git pull --ff-only did not work in your clone");
            o.more(String::from_utf8_lossy(&out.stderr).trim());
            o.hint("Commit or stash your changes, or pull by hand; then run `zendo upgrade` again.");
            return false;
        }
        Err(e) => {
            o.fail("git", format!("cannot run git: {e}"));
            return false;
        }
    }
    let head = git(&["rev-parse", "HEAD"]).ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let dirty = git(&["status", "--porcelain", "--untracked-files=no"]).is_ok_and(|o| !o.stdout.is_empty());
    let built = installed_commit().unwrap_or_default();
    if !force && !dirty && !built_dirty() && same_commit(&built, &head) {
        o.ok("latest", format!("your clone is at {} and zendo is built from it", &head[..7.min(head.len())]));
        return true;
    }
    println!("  building your clone… (a few minutes the first time, faster after that)\n");
    let status = std::process::Command::new("cargo")
        .args(["install", "--path"])
        .arg(src)
        .args(["--locked", "--force"])
        .envs(&env)
        .status();
    println!();
    if !status.is_ok_and(|s| s.success()) {
        o.fail("build", "cargo install failed; see the output above");
        return false;
    }
    let _ = std::fs::remove_file(config::data_dir().join("update.json"));
    let new = std::process::Command::new(&exe).arg("--version").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    o.ok("upgraded", new);
    std::process::Command::new(&exe).arg("setup").status().is_ok_and(|s| s.success())
}

/// `zendo upgrade`: build the latest `main` from GitHub when it is newer, then run setup.
pub async fn upgrade(force: bool) -> bool {
    let mut o = Out::new();
    o.head("zendo upgrade");
    o.ok("installed", version_line());
    // Built from your clone: the clone pulls from its own origin.
    if let Some(src) = local_clone() {
        return upgrade_clone(&src, force, &mut o).await;
    }
    let repos = repos();
    // Take the path now: after cargo replaces the file, Linux reports it as "… (deleted)".
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("zendo"));
    let exe = PathBuf::from(exe.to_string_lossy().trim_end_matches(" (deleted)").to_string());
    let env = git_env();

    // Find a repo URL that works, and the newest commit on main.
    let mut source = None;
    for r in &repos {
        if let Some(sha) = remote_head(r, &env, 60).await {
            source = Some((r.clone(), sha));
            break;
        }
    }
    let Some((repo, head)) = source else {
        o.fail("github", "cannot reach the zendo repository");
        o.hint("Check your SSH key (`ssh -T git@github.com`) or run `gh auth setup-git`.");
        return false;
    };
    let mine = installed_commit();
    if !force && mine.as_deref().is_some_and(|m| same_commit(m, &head)) {
        o.ok("latest", format!("you have the newest main ({})", &head[..7.min(head.len())]));
        return true;
    }
    o.ok("main", format!("{} · {}", &head[..7.min(head.len())], repo));

    println!("  building… (a few minutes the first time, faster after that)\n");
    let status = std::process::Command::new("cargo")
        .args(["install", "--git", &repo, "--locked", "--force", "zendo"])
        .envs(&env)
        .status();
    println!();
    match status {
        Ok(s) if s.success() => {}
        Ok(_) => {
            o.fail("build", "cargo install failed; see the output above");
            return false;
        }
        Err(e) => {
            o.fail("cargo", format!("cannot run cargo: {e}"));
            o.hint("Install Rust from https://rustup.rs");
            return false;
        }
    }
    // The new binary has the same path; let it check the rest.
    let new = std::process::Command::new(&exe).arg("--version").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let _ = std::fs::remove_file(config::data_dir().join("update.json"));
    o.ok("upgraded", format!("zendo {} → {}", env!("CARGO_PKG_VERSION"), new.trim_start_matches("zendo ")));
    std::process::Command::new(&exe).arg("setup").status().is_ok_and(|s| s.success())
}
