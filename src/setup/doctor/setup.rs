//! `zendo setup`: install the agent adapter, check the login, set the notes repo.
use super::*;

pub async fn setup(paths: &Paths, agent_flag: Option<String>, notes_repo: Option<String>) -> bool {
    let mut o = Out::new();
    let mut cfg = UserConfig::load();
    o.head("zendo setup");
    let Some(turned_on) = set_notes_repo(&mut cfg, notes_repo.as_deref(), &mut o) else { return false };

    // 1. Node and npm.
    let env = config::agent_env(&cfg);
    match run("node", &["--version"], &env, 20).await {
        Some((true, v)) => o.ok("node", v),
        _ => {
            o.fail("node", "not found");
            o.hint("Install Node.js 18 or newer (22 LTS is best), then run `zendo setup` again.");
            return false;
        }
    }
    let system_ca = matches!(run("node", &[SYSTEM_CA, "-e", "0"], &BTreeMap::new(), 20).await, Some((true, _)));

    // The agent: a profile (claude, codex) from --agent or the config, else a command of the user.
    let custom = agent_flag.as_deref().filter(|f| config::Profile::by_id(f).is_none()).map(str::to_string);
    let profile = agent_flag.as_deref().and_then(config::Profile::by_id).or_else(|| cfg.agent.as_deref().and_then(config::Profile::by_id)).unwrap_or(&config::CLAUDE);
    let keep = if profile.kind == config::Kind::Claude { None } else { Some(profile.id.to_string()) };
    if custom.is_some() {
        cfg.agent = custom;
    } else if profile.installed() {
        // 2. The adapter zendo needs is there: no network.
        o.ok("adapter", format!("{}@{} is installed", profile.package, profile.version));
        cfg.agent = keep.clone();
    } else {
        if !install_adapter(profile, &env, system_ca, &mut cfg, &mut o).await {
            return false;
        }
        cfg.agent = keep; // use the installed adapter
    }

    if let Err(e) = cfg.save() {
        o.fail("config", format!("cannot write {}: {e}", config::config_path().display()));
        return false;
    }

    // 4. Check the whole path.
    let ok = checks(paths, None, false, true, &mut o).await;
    // 5. The notes repo is new: Claude moves the notes of this repo into it.
    let mut moved = true;
    if ok && turned_on && paths.root.join(".git").exists() && !crate::setup::sync::notes_in(&paths.repo_notes).is_empty() {
        let n = cfg.notes.clone().expect("set above");
        moved = move_notes(paths, None, &n, &mut o).await;
    }
    println!();
    if ok {
        println!("  Ready. Run `zendo` in a repository.");
    }
    ok && moved
}

/// `--notes-repo`: turn the notes repo on or off. `Some(true)` when it was turned on now; `None` on an error.
fn set_notes_repo(cfg: &mut UserConfig, spec: Option<&str>, o: &mut Out) -> Option<bool> {
    match spec {
        None => Some(false),
        Some("off") => {
            cfg.notes = None;
            o.ok("notes repo", "off: notes stay in each repository");
            Some(false)
        }
        Some(spec) => match use_notes_repo(spec, cfg.notes.as_ref()) {
            Ok(n) => {
                o.ok("notes repo", format!("{} · this machine: {}", n.path.display(), n.machine));
                cfg.notes = Some(n);
                Some(true)
            }
            Err(e) => {
                o.fail("notes repo", e);
                o.hint("Give a folder with a git clone, or a git URL: zendo setup --notes-repo git@github.com:you/notes.git");
                None
            }
        },
    }
}

/// Step 2: install the adapter once, pinned, outside the repo. No `npm ping`: some company
/// registries do not answer it, but install works through them.
async fn install_adapter(profile: &config::Profile, env: &BTreeMap<String, String>, system_ca: bool, cfg: &mut UserConfig, o: &mut Out) -> bool {
    let dir = profile.install_dir();
    let _ = std::fs::create_dir_all(&dir);
    let pkg = format!("{}@{}", profile.package, profile.version);
    let dir_s = dir.display().to_string();
    let install = |env: BTreeMap<String, String>| {
        let dir_s = dir_s.clone();
        let pkg = pkg.clone();
        async move { run("npm", &["install", "--prefix", &dir_s, "--no-fund", "--no-audit", "--omit=dev", "--loglevel=error", &pkg], &env, 600).await }
    };
    if o.color {
        print!("  … {:<11}installing {pkg}", "adapter");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
    let mut r = install(env.clone()).await;
    // A certificate error: try the system CA store once, and keep it when it works.
    if let Some((false, out)) = &r
        && is_tls_error(out)
        && system_ca
    {
        let again = install(with_system_ca(env)).await;
        if matches!(again, Some((true, _))) {
            cfg.env.insert("NODE_OPTIONS".into(), SYSTEM_CA.into());
            if o.color {
                print!("\r\x1b[2K");
            }
            o.ok("registry", format!("works with NODE_OPTIONS={SYSTEM_CA}; saved for zendo"));
        }
        r = again;
    }
    if o.color {
        print!("\r\x1b[2K");
    }
    match r {
        Some((true, _)) if profile.entry_path().exists() => {
            o.ok("adapter", format!("{pkg} in {}", dir.display()));
            true
        }
        Some((_, out)) => {
            if is_tls_error(&out) {
                o.fail("adapter", "npm does not trust the certificate of your network");
                o.more(npm_errors(&out));
                tls_hints(o, system_ca);
            } else {
                o.fail("adapter", format!("npm could not install {pkg}"));
                o.more(npm_errors(&out));
                o.hint("Check the npm registry and proxy: `npm config get registry`, `npm config get proxy`, HTTPS_PROXY.");
                o.hint("On a VPN that blocks npm: run `zendo setup` once off the VPN. After that, zendo needs no npm.");
            }
            false
        }
        None => {
            o.fail("npm", "not found");
            o.hint("npm comes with Node.js. Check your Node install.");
            false
        }
    }
}

/// The notes repo from a folder with a clone, or from a git URL (cloned into the data folder).
pub(super) fn use_notes_repo(spec: &str, old: Option<&config::NotesRepo>) -> Result<config::NotesRepo, String> {
    let url = spec.contains("://") || (spec.contains('@') && spec.contains(':') && !Path::new(spec).exists());
    let path = if url {
        let dest = config::data_dir().join("notes");
        if !dest.join(".git").exists() {
            let _ = std::fs::create_dir_all(config::data_dir());
            let d = dest.display().to_string();
            crate::setup::sync::git(&config::data_dir(), &["clone", "-q", spec, &d], Duration::from_secs(300)).map_err(|e| format!("cannot clone {spec}: {e}"))?;
        } else if crate::setup::sync::origin_of(&dest).as_deref() != Some(&crate::setup::sync::normalize_origin(spec)) {
            return Err(format!("{} holds another repo; move it away or give its folder", dest.display()));
        }
        dest
    } else {
        let p = spec.strip_prefix("~/").map(|r| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(r)).unwrap_or_else(|| PathBuf::from(spec));
        let p = p.canonicalize().map_err(|_| format!("{spec} does not exist"))?;
        if !crate::setup::sync::is_work_tree(&p) {
            return Err(format!("{} is not a git repository (git init, or clone your notes repo there)", p.display()));
        }
        p
    };
    Ok(config::NotesRepo {
        path,
        machine: old.map(|o| o.machine.clone()).unwrap_or_else(crate::setup::sync::machine_name),
        sync_seconds: old.map_or(crate::setup::sync::DEFAULT_SECONDS, |o| o.sync_seconds),
    })
}

pub(super) fn notes_repo_check(n: &config::NotesRepo, o: &mut Out) {
    if !crate::setup::sync::is_work_tree(&n.path) {
        o.fail("notes repo", format!("{} is not a git repository", n.path.display()));
        o.hint("Run `zendo setup --notes-repo <folder or git URL>`, or `zendo setup --notes-repo off`.");
        return;
    }
    let remote = crate::setup::sync::origin_of(&n.path);
    o.ok("notes repo", format!("{} · {} · sync every {} s", n.path.display(), remote.as_deref().unwrap_or("no remote"), n.sync_seconds));
    if remote.is_none() {
        o.warn("notes sync", "no remote: the notes stay on this machine");
    }
    if n.path.join(".git/rebase-merge").exists() || n.path.join(".git/rebase-apply").exists() {
        o.fail("notes sync", "a rebase is in progress in the notes repo");
        o.hint(format!("Finish it in {}: git status, then git rebase --continue or --abort.", n.path.display()));
    }
}
