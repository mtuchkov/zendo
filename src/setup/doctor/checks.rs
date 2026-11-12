//! `zendo doctor`: check each step of the setup.
use super::*;

pub async fn doctor(paths: &Paths, agent_flag: Option<String>, live: bool, fix: bool) -> bool {
    let mut o = Out::new();
    o.head("zendo doctor");
    let mut ok = checks(paths, agent_flag.clone(), live, false, &mut o).await;
    let (agent, _) = config::resolve_agent(agent_flag.clone(), &UserConfig::load());
    setup_project(paths, &agent, &mut o);
    println!();
    if let Some(problem) = structure_problem(&paths.root) {
        let what = if matches!(problem, Problem::Missing) { "Create it from project-structure.md" } else { "Fix it" };
        let go = fix || ask_yes(&format!("  {what} with Claude? (zendo first fixes what it can by itself) [Y/n] "));
        if go {
            let others_ok = o.failures.iter().all(|f| f == "structure");
            let fixed = fix_structure(paths, agent_flag, problem, &mut o).await;
            // The structure file was the only failure, and now zendo understands it.
            ok = fixed && others_ok;
            println!();
        } else {
            println!("  To fix {} later: zendo doctor --fix\n", crate::docs::structure::FILE);
        }
    }
    if ok && !live {
        println!("  All checks passed. `zendo doctor --live` also sends one short prompt to Claude.");
    } else if ok {
        println!("  All checks passed.");
    }
    ok
}

pub(super) async fn checks(paths: &Paths, agent_flag: Option<String>, live: bool, quiet_outside_repo: bool, o: &mut Out) -> bool {
    let cfg = UserConfig::load();
    let env = config::agent_env(&cfg);
    check_repo(paths, &cfg, quiet_outside_repo, o);
    check_settings(&cfg, o);
    check_structure(paths, o);
    let (cmd, source) = check_config(&cfg, agent_flag, o);
    check_network(o);
    check_node(&cmd, source, &env, o).await;
    let kind = check_adapter(&cmd, source, o);
    check_claude(kind, &env, o).await;
    if let Some((h, mut rx)) = handshake(paths, &cmd, source, &env, o).await {
        if live {
            live_prompt(&h, &mut rx, o).await;
        }
        drop(h);
    }
    !o.failed
}

fn check_repo(paths: &Paths, cfg: &UserConfig, quiet_outside_repo: bool, o: &mut Out) {
    // Repo and notes. Setup runs anywhere, e.g. from the installer: skip them outside a repo.
    o.ok("zendo", env!("CARGO_PKG_VERSION"));
    let in_repo = paths.root.join(".git").exists();
    if in_repo {
        o.ok("repo", paths.root.display().to_string());
    } else if !quiet_outside_repo {
        o.warn("repo", format!("{} (no .git: zendo uses this folder)", paths.root.display()));
    }
    let rel = paths.notes_dir.strip_prefix(&paths.root).unwrap_or(&paths.notes_dir).display().to_string();
    match std::fs::read_dir(&paths.notes_dir) {
        _ if !in_repo && quiet_outside_repo => {}
        Ok(d) => {
            let n = d.filter_map(|e| e.ok()).filter(|e| e.path().extension().is_some_and(|x| x == "md")).count();
            o.ok("notes", format!("{rel} · {n} notes"));
        }
        Err(_) => o.warn("notes", format!("{rel} does not exist yet; zendo creates it on the first note")),
    }
    if let Some(n) = &cfg.notes {
        notes_repo_check(n, o);
    }
}

fn check_settings(cfg: &UserConfig, o: &mut Out) {
    let (on, hl_problems) = crate::text::highlight::problems(cfg.highlight.as_ref());
    o.ok("highlight", on.join(", "));
    for p in hl_problems {
        o.warn("highlight", p);
    }
    let (_, problems) = cfg.keymap();
    if let (Some(path), true) = (&cfg.keymap, problems.is_empty()) {
        o.ok("keys", format!("keymap {path}"));
    }
    if !cfg.keys.is_empty() && problems.is_empty() {
        o.ok("keys", format!("{} pane(s) with own keys", cfg.keys.len()));
    }
    for p in problems {
        o.warn("keys", p);
    }

}

fn check_structure(paths: &Paths, o: &mut Out) {
    // The project structure file, when the repo has one.
    match structure_problem(&paths.root) {
        None => {
            if let Some(Ok(st)) = crate::docs::structure::Structure::load(&paths.root) {
                o.ok("structure", format!("{} · {}", crate::docs::structure::FILE, counts(&st)));
            }
        }
        Some(Problem::Missing) => o.warn("structure", format!("no {}, but project-structure.md is there", crate::docs::structure::FILE)),
        Some(Problem::Parse(e)) => {
            o.fail("structure", format!("{} does not parse", crate::docs::structure::FILE));
            o.more(e.lines().take(6).collect::<Vec<_>>().join("\n"));
        }
        Some(Problem::Issues(list)) => {
            o.warn("structure", format!("{}: zendo does not understand {} thing{}", crate::docs::structure::FILE, list.len(), if list.len() == 1 { "" } else { "s" }));
            let shown: Vec<String> = list.iter().take(8).map(|i| format!("{}: {}", i.at, i.what)).collect();
            o.more(shown.join("\n"));
            if list.len() > 8 {
                o.more(format!("… and {} more", list.len() - 8));
            }
        }
    }

}

fn check_config(cfg: &UserConfig, agent_flag: Option<String>, o: &mut Out) -> (String, Source) {
    // Config and agent command.
    let cpath = config::config_path();
    if let Err(e) = UserConfig::read() {
        o.fail("config", format!("cannot read {}: {}", cpath.display(), e.lines().next().unwrap_or("")));
        o.more(e);
        o.hint("Fix the file by hand, or delete it and run `zendo setup` again.");
    } else if cpath.exists() {
        let keys: Vec<String> = cfg.env.iter().map(|(k, v)| format!("{k}={v}")).collect();
        o.ok("config", format!("{}{}", cpath.display(), if keys.is_empty() { String::new() } else { format!(" · {}", keys.join(" ")) }));
    } else {
        o.warn("config", format!("{} not found (run `zendo setup`)", cpath.display()));
    }
    let (cmd, source) = config::resolve_agent(agent_flag, cfg);
    o.ok("agent", &cmd);
    o.more(format!("from {}", source.label()));

    (cmd, source)
}

fn check_network(o: &mut Out) {
    // Environment that changes how Node and npm connect.
    let mut net = Vec::new();
    for k in ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "NO_PROXY", "NODE_OPTIONS", "NODE_EXTRA_CA_CERTS"] {
        if let Ok(v) = std::env::var(k) {
            let shown = if k.contains("PROXY") || k.contains("proxy") { "set".to_string() } else { v.clone() };
            net.push(format!("{k}={shown}"));
            if k == "NODE_EXTRA_CA_CERTS" && !Path::new(&v).exists() {
                o.fail("CA file", format!("NODE_EXTRA_CA_CERTS points to a missing file: {v}"));
            }
        }
    }
    if !net.is_empty() {
        o.ok("network", net.join(" · "));
    }

}

/// Show the agent's own error lines and the fix for the known ones.
pub(super) fn explain_log(o: &Out, log: &Path) {
    let text = std::fs::read_to_string(log).unwrap_or_default();
    let errs: Vec<&str> = text.lines().filter_map(|l| l.strip_prefix("Stderr ")).filter(|l| !l.trim().is_empty()).collect();
    if !errs.is_empty() {
        let tail: Vec<&str> = errs.iter().rev().take(4).rev().copied().collect();
        o.more(tail.join("\n"));
    }
    if is_tls_error(&text) {
        o.hint("npm does not trust the certificate of your network. Run `zendo setup`.");
    } else if text.contains("command not found") || text.contains("No such file") || text.contains("ENOENT") {
        o.hint("The agent command was not found. Run `zendo setup`, or check --agent.");
    }
    o.more(format!("full log: {}", log.display()));
}
