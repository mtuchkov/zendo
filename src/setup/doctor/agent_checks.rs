//! `zendo doctor`: the agent checks, from Node to a real ACP handshake.
use super::*;

pub(super) async fn check_node(cmd: &str, source: Source, env: &BTreeMap<String, String>, o: &mut Out) {
    // Node.
    let uses_node = cmd.starts_with("node") || cmd.starts_with("npx");
    if uses_node {
        match run("node", &["--version"], env, 20).await {
            Some((true, v)) => {
                let ca = matches!(run("node", &[SYSTEM_CA, "-e", "0"], &BTreeMap::new(), 20).await, Some((true, _)));
                o.ok("node", format!("{v}{}", if ca { " · can use the system CA store" } else { "" }));
            }
            _ => {
                o.fail("node", "not found");
                o.hint("Install Node.js 22 LTS.");
            }
        }
    }
    if source == Source::Npx {
        match run("npm", &["ping"], env, 60).await {
            Some((true, _)) => o.ok("registry", "npm can reach the registry"),
            Some((false, out)) => {
                if is_tls_error(&out) {
                    o.fail("registry", "TLS: npm does not trust the certificate of your network");
                    o.more(npm_errors(&out));
                    o.hint("Run `zendo setup`: it installs the adapter once and fixes this if your system trusts the CA.");
                } else {
                    o.fail("registry", "npm cannot reach the registry");
                    o.more(npm_errors(&out));
                }
            }
            None => o.fail("npm", "not found"),
        }
    }
}

/// The installed adapter and, for Codex, its login. The kind of agent the command is.
pub(super) fn check_adapter(cmd: &str, source: Source, handoff: &str, o: &mut Out) -> config::Kind {
    let kind = config::kind_of(cmd);
    if source == Source::Installed
        && let Some(p) = config::Profile::by_kind(kind)
    {
        match p.installed_version() {
            Some(v) if v == p.version => o.ok("adapter", format!("{}@{v}", p.package)),
            Some(v) => {
                o.warn("adapter", format!("{}@{v}; zendo expects {}", p.package, p.version));
                o.hint("Run `zendo setup` to install the expected version.");
            }
            None => o.fail("adapter", "installed files are broken; run `zendo setup`"),
        }
    }
    if kind == config::Kind::Codex {
        if codex_logged_in() {
            o.ok("login", "Codex has a login or an API key");
        } else {
            o.fail("login", "Codex is not logged in");
            o.hint(CODEX_LOGIN);
        }
        o.more(codex_note(handoff));
    }

    kind
}

/// The hand-off key of the keymap of the user, for the messages of the doctor (PANES-28).
pub(super) fn handoff_key(cfg: &config::UserConfig) -> String {
    cfg.keymap().0.key(crate::keys::Pane::Chat, "handoff").unwrap_or_else(|| "the hand-off key".into())
}

/// What Codex cannot do (CLAUDE-14), with the hand-off key of the user (PANES-28).
pub(super) fn codex_note(handoff: &str) -> String {
    format!("With Codex: no hand-off to the command line ({handoff}). Everything else works the same.")
}

/// Claude Code and its login, when the agent is Claude.
pub(super) async fn check_claude(kind: config::Kind, env: &BTreeMap<String, String>, handoff: &str, o: &mut Out) {
    // Claude Code login, when the agent is Claude.
    let is_claude = kind == config::Kind::Claude;
    match if is_claude { run("claude", &["--version"], env, 20).await } else { Some((false, String::new())) } {
        Some((false, v)) if v.is_empty() => {}
        Some((true, v)) => {
            o.ok("claude", first_line(&v));
            match run("claude", &["auth", "status"], env, 30).await {
                Some((_, out)) => match serde_json::from_str::<serde_json::Value>(&out) {
                    Ok(j) if j["loggedIn"] == true => o.ok("login", format!("logged in ({})", j["authMethod"].as_str().unwrap_or("?"))),
                    Ok(_) if std::env::var("ANTHROPIC_API_KEY").is_ok() => o.ok("login", "ANTHROPIC_API_KEY is set"),
                    Ok(_) => {
                        o.fail("login", "Claude Code is not logged in");
                        o.hint("Run `claude auth login` (or `claude` and /login), then `zendo doctor` again.");
                    }
                    Err(_) => o.warn("login", "could not read `claude auth status`"),
                },
                None => o.warn("login", "could not run `claude auth status`"),
            }
        }
        _ => o.warn("claude", format!("the `claude` command is not on PATH; login and the {handoff} hand-off cannot be checked")),
    }

}

/// The real thing: start the agent and do the ACP handshake.
pub(super) async fn handshake(paths: &Paths, cmd: &str, source: Source, env: &BTreeMap<String, String>, o: &mut Out) -> Option<(agent::AgentHandle, tokio::sync::mpsc::UnboundedReceiver<Event>)> {
    let log = config::data_dir().join("doctor.log");
    let _ = std::fs::create_dir_all(config::data_dir());
    let _ = std::fs::remove_file(&log);
    let started = Instant::now();
    let (h, mut rx) = agent::spawn(cmd.to_string(), env.clone(), paths.root.clone(), Some(log.clone()));
    let wait = Duration::from_secs(if source == Source::Npx { 180 } else { 60 });
    let first = tokio::time::timeout(wait, rx.recv()).await;
    let ready = match first {
        Ok(Some(Event::Ready { agent, can_load, .. })) => {
            o.ok("handshake", format!("{agent} answered initialize in {:.1} s{}", started.elapsed().as_secs_f32(), if can_load { " · can resume threads" } else { "" }));
            true
        }
        Ok(Some(Event::Exited { text })) => {
            o.fail("handshake", format!("the agent did not start: {}", short(&text)));
            explain_log(o, &log);
            false
        }
        Ok(_) => {
            o.fail("handshake", "unexpected first message from the agent");
            false
        }
        Err(_) => {
            o.fail("handshake", format!("no answer in {} s", wait.as_secs()));
            explain_log(o, &log);
            false
        }
    };

    ready.then_some((h, rx))
}

/// One short prompt, to see that the agent answers.
pub(super) async fn live_prompt(h: &agent::AgentHandle, rx: &mut tokio::sync::mpsc::UnboundedReceiver<Event>, o: &mut Out) {
    let t = Instant::now();
    h.send(Cmd::Prompt { key: "doctor".into(), known: None, text: "This is a connection test. Reply with the single word OK.".into(), attach: vec![] });
    let mut text = String::new();
    let mut error = None;
    loop {
        match tokio::time::timeout(Duration::from_secs(120), rx.recv()).await {
            Ok(Some(Event::Text { text: t, .. })) => text.push_str(&t),
            Ok(Some(Event::Error { text: e, .. })) => error = Some(e),
            Ok(Some(Event::Permission { request, .. })) => h.send(Cmd::Answer { request, option: None }),
            Ok(Some(Event::TurnEnd { .. })) => break,
            Ok(Some(Event::Exited { text: e })) => {
                error = Some(e);
                break;
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => {
                error = Some("no answer in 120 s".into());
                break;
            }
        }
    }
    match error {
        Some(e) if e.contains("Authentication required") || text.contains("Authentication error") => {
            o.fail("prompt", "Claude answered: authentication required");
            o.hint("Run `claude auth login`, then `zendo doctor --live` again.");
        }
        Some(e) => o.fail("prompt", e),
        None => o.ok("prompt", format!("Claude answered \"{}\" in {:.1} s", text.trim(), t.elapsed().as_secs_f32())),
    }
}
