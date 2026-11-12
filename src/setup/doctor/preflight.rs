//! The start check: the agent answers, and the login works, before the screen opens.
use super::*;

/// A working agent, handed to the app after the start check.
pub struct Live {
    pub handle: agent::AgentHandle,
    pub events: tokio::sync::mpsc::UnboundedReceiver<Event>,
    pub name: String,
}

/// The start check: `zendo` opens only when Claude answers.
/// Prints what to fix and returns `None` when it does not.
pub async fn preflight(cmd: &str, env: &BTreeMap<String, String>, root: &Path, log: PathBuf) -> Option<Live> {
    use std::io::Write;
    let npx = cmd.starts_with("npx");
    let kind = config::kind_of(cmd);
    let who = config::Profile::by_kind(kind).map_or("the agent", |p| p.name);
    if npx {
        eprint!("zendo: starting {who} (the first run downloads the adapter; `zendo setup` installs it once)… ");
    } else {
        eprint!("zendo: checking {who}… ");
    }
    let _ = std::io::stderr().flush();

    let login = logged_in(kind, env);
    let _ = std::fs::remove_file(&log);
    let (handle, mut events) = agent::spawn(cmd.to_string(), env.clone(), root.to_path_buf(), Some(log.clone()));
    let wait = Duration::from_secs(if npx { 180 } else { 60 });
    let hello = tokio::time::timeout(wait, events.recv());
    let (login, hello) = tokio::join!(login, hello);
    if std::io::stderr().is_terminal() {
        eprint!("\r\x1b[2K");
    } else {
        eprintln!();
    }

    let mut o = Out { color: std::io::stderr().is_terminal(), failed: false, failures: Vec::new() };
    let name = match hello {
        Ok(Some(Event::Ready { agent, .. })) => Some(agent),
        Ok(Some(Event::Exited { text })) => {
            o.fail("claude", format!("the agent did not start: {}", short(&text)));
            explain_log(&o, &log);
            None
        }
        Ok(_) => {
            o.fail("claude", "the agent sent an unexpected first message");
            None
        }
        Err(_) => {
            o.fail("claude", format!("the agent did not answer in {} s", wait.as_secs()));
            explain_log(&o, &log);
            None
        }
    };
    if login.is_err() && kind == config::Kind::Codex {
        o.fail("login", "Codex is not logged in");
        o.hint(CODEX_LOGIN);
    } else if login.is_err() {
        o.fail("login", "Claude Code is not logged in");
        o.hint("Run `claude auth login`, then start zendo again.");
    }
    match (name, o.failed) {
        (Some(name), false) => Some(Live { handle, events, name }),
        _ => {
            println!();
            println!("  zendo did not open, because {who} does not work yet.");
            println!("  `zendo doctor` checks every step. `zendo --offline` opens the notes without Claude.");
            None
        }
    }
}

/// The login of the agent: `Err` only when it is sure that there is none.
async fn logged_in(kind: config::Kind, env: &BTreeMap<String, String>) -> Result<(), ()> {
    if kind == config::Kind::Codex {
        return if codex_logged_in() { Ok(()) } else { Err(()) };
    }
    if kind != config::Kind::Claude || std::env::var("ANTHROPIC_API_KEY").is_ok() {
        return Ok(());
    }
    match run("claude", &["auth", "status"], env, 30).await {
        // Without the `claude` command we cannot tell; the handshake still runs.
        None => Ok(()),
        Some((_, out)) => match serde_json::from_str::<serde_json::Value>(&out) {
            Ok(j) if j["loggedIn"] == false => Err(()),
            _ => Ok(()),
        },
    }
}
