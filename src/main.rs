use anyhow::{Result, bail};
use zendo::app::{App, Config};
use zendo::setup::config::{self, UserConfig};
use zendo::setup::doctor;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;

#[path = "main_args.rs"]
mod args;
use args::{HELP, read_args};

enum Run {
    Tui(Config),
    Setup(doctor::Paths, Option<String>, Option<String>),
    /// `zendo setup fonts` (FONT-7).
    Fonts(config::AgentEnv),
    Notes(doctor::Paths, Option<String>, String),
    Init(doctor::Paths, Option<String>),
    Doctor(doctor::Paths, Option<String>, doctor::DoctorFlags),
    Upgrade(bool),
}

/// The nearest folder at or above `start` with `.git`. IF there is no such
/// folder, THEN the home folder (a scratchpad), or `start` without a home.
fn repo_root(start: &Path, home: Option<&Path>) -> PathBuf {
    let mut p = start.to_path_buf();
    loop {
        if p.join(".git").exists() {
            return p;
        }
        if !p.pop() {
            return home.map_or_else(|| start.to_path_buf(), Path::to_path_buf);
        }
    }
}

fn parse() -> Result<Run> {
    let cwd = std::env::current_dir()?;
    let home = std::env::var_os("HOME").filter(|h| !h.is_empty()).map(PathBuf::from);
    let root = repo_root(&cwd, home.as_deref());
    if let Some((old, new)) = config::old_config_to_toml() {
        eprintln!("zendo: moved {} to {}", old.display(), new.display());
    }
    let a = read_args()?;
    let repo_notes = if Path::new(&a.notes).is_absolute() { PathBuf::from(&a.notes) } else { root.join(&a.notes) };
    let user = UserConfig::load();
    let notes_cfg = user.notes.clone().filter(|_| !a.notes_set);
    let notes_dir = match &notes_cfg {
        Some(n) => n.location(&root),
        None => repo_notes.clone(),
    };
    // One agent environment for the whole run, after the question about the trust (TRUST-2); a
    // profile that is not there stops zendo (PROFILE-3).
    match a.sub {
        "notes" if a.action.is_empty() => bail!("zendo notes needs move or consolidate\n\n{HELP}"),
        "doctor" if a.with_agent && !a.fix => bail!("--agent needs --fix\n\n{HELP}"),
        _ => {}
    }
    // `zendo init` reads no `.envrc` and starts no agent, so it needs no agent environment.
    let env = match a.sub {
        "upgrade" | "keymap" | "init" => config::AgentEnv::default(),
        sub => {
            let cmd = config::trust::Command { checks: matches!(sub, "doctor" | "setup"), saves: sub != "doctor" || a.fix };
            run_env(&user, &cwd, home.as_deref(), a.profile.as_deref(), cmd)?
        }
    };
    // TRUST-6: without the trust, no program of the checks starts in the folder: an agent, npm and
    // npx take settings and programs from the folder where they start.
    if env.untrusted.is_some() {
        let dir = config::data_dir();
        std::fs::create_dir_all(&dir).and_then(|()| std::env::set_current_dir(&dir)).map_err(|e| anyhow::anyhow!("cannot work in {}: {e}", dir.display()))?;
    }
    let paths = doctor::Paths { root: root.clone(), notes_dir: notes_dir.clone(), repo_notes, env: env.clone() };
    Ok(match a.sub {
        "setup" if a.fonts => Run::Fonts(env),
        "setup" => Run::Setup(paths, a.agent, a.notes_repo),
        "notes" => Run::Notes(paths, a.agent, a.action),
        "init" => Run::Init(paths, a.agent),
        "doctor" => Run::Doctor(paths, a.agent, doctor::DoctorFlags { live: a.live, fix: a.fix, agent: a.with_agent }),
        "upgrade" => Run::Upgrade(a.live),
        "keymap" => {
            print!("{}", zendo::keys::DEFAULT_KEYMAP);
            std::process::exit(0);
        }
        _ => {
            let (agent, _) = config::resolve_agent(a.agent, &user);
            Run::Tui(Config { root, notes_dir, agent, agent_env: env, offline: a.offline, notes_repo: notes_cfg, user_config: config::config_path() })
        }
    })
}

/// The agent environment of this run of the command `cmd`. zendo asks about the trust only in a
/// terminal (TRUST-2).
fn run_env(user: &UserConfig, dir: &Path, home: Option<&Path>, profile: Option<&str>, cmd: config::trust::Command) -> Result<config::AgentEnv> {
    use std::io::IsTerminal;
    let (stdin, mut err) = (std::io::stdin(), std::io::stderr());
    let mut input = stdin.lock();
    let terminal = (stdin.is_terminal() && err.is_terminal()).then_some(config::trust::Terminal { input: &mut input, out: &mut err });
    config::trust::run_env(user, (&config::config_path(), home), dir, profile, cmd, terminal).map_err(anyhow::Error::msg)
}

/// `zendo board watch --session N [--for ID] [--timeout S]`: the watch of the board (BOARD-20).
/// None for other arguments.
fn board_watch() -> Result<Option<i32>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [b, w, rest @ ..] = args.as_slice() else { return Ok(None) };
    if b != "board" || w != "watch" {
        return Ok(None);
    }
    let Some((session, for_id, timeout)) = zendo::board::watch::parse(rest) else {
        bail!("usage: zendo board watch --session N [--for ID] [--timeout SECONDS]");
    };
    let dir = zendo::board::default_dir().ok_or_else(|| anyhow::anyhow!("no home folder for the board"))?;
    let repo = zendo::board::repo_of(&std::env::current_dir()?);
    let watch = zendo::board::watch::Watch { journal: zendo::board::Journal::at(dir), repo, session, for_id, timeout, poll: std::time::Duration::from_millis(500) };
    Ok(Some(watch.run(&mut std::io::stdout().lock())?))
}

/// `zendo mcp --session N`: the MCP server of zendo session N (SESSIONS-21).
fn mcp_session() -> Option<usize> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    match a.as_slice() {
        [m, s, n] if m == "mcp" && s == "--session" => n.parse().ok(),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Before anything else: the server only answers on stdin and stdout.
    if let Some(n) = mcp_session() {
        return Ok(zendo::mcp::serve(std::io::stdin().lock(), std::io::stdout().lock(), n)?);
    }
    if let Some(code) = board_watch()? {
        std::process::exit(code);
    }
    let cfg = match parse()? {
        Run::Tui(cfg) => cfg,
        Run::Setup(paths, agent, notes_repo) => std::process::exit(if doctor::setup(&paths, agent, notes_repo).await { 0 } else { 1 }),
        Run::Fonts(env) => std::process::exit(if doctor::setup_fonts(&env) { 0 } else { 1 }),
        Run::Notes(paths, agent, action) => std::process::exit(if doctor::notes(&paths, agent, &action).await { 0 } else { 1 }),
        Run::Init(paths, agent) => std::process::exit(if doctor::init(&paths, agent) { 0 } else { 1 }),
        Run::Doctor(paths, agent, flags) => std::process::exit(if doctor::doctor(&paths, agent, flags).await { 0 } else { 1 }),
        Run::Upgrade(force) => std::process::exit(if doctor::upgrade(force).await { 0 } else { 1 }),
    };
    // The path of this binary, before an upgrade replaces it.
    let exe = std::env::current_exe().ok();
    let mut app = open_app(cfg).await?;
    // The message board between the sessions, in the folder of the user (ADR 0021).
    if let Some(dir) = zendo::board::default_dir() {
        app.board = zendo::app::BoardLink::open(dir, &app.cfg.root);
    }

    let mut term = ratatui::init();
    zendo::tui::terminal_on();
    let (keys_tx, mut keys) = mpsc::unbounded_channel();
    zendo::tui::spawn_input(keys_tx.clone());

    let result = zendo::tui::run(&mut term, &mut app, &mut keys, keys_tx).await;

    if app.mode == zendo::app::Mode::Write {
        app.save();
    }
    app.end_visit();
    app.board_quit();
    app.end_telemetry();
    zendo::tui::terminal_off();
    ratatui::restore();
    if let Some(s) = app.files.take_syncer()
        && let Some(label) = s.finish().label() {
            eprintln!("zendo: {label}");
        }
    result?;
    if app.update.upgrade_now() {
        upgrade_and_restart(exe).await;
    }
    Ok(())
}

/// The app, after the start check of the agent, with the notes repo and the update check.
async fn open_app(cfg: Config) -> Result<App> {
    // Ask GitHub for a newer zendo while the start check runs; the answer arrives in the app.
    let (update_tx, update_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let _ = update_tx.send(doctor::update_available().await);
    });
    // Invariant: when zendo is up, Claude works. Check before the screen opens.
    let live = if cfg.offline {
        None
    } else {
        let log = zendo::setup::state::dir(&cfg.root);
        let _ = std::fs::create_dir_all(&log);
        match doctor::preflight(&cfg.agent, &cfg.agent_env.vars, &cfg.root, log.join("agent.log")).await {
            Some(l) => Some(l),
            None => std::process::exit(1),
        }
    };
    // The notes repo: this folder, its location file, and the sync in the background.
    let syncer = match &cfg.notes_repo {
        Some(n) => match n.prepare(&cfg.root) {
            Ok(dir) => Some(zendo::setup::sync::Syncer::of_user(n.path.clone(), dir, &n.machine, Duration::from_secs(n.sync_seconds.max(1)))),
            Err(e) => {
                eprintln!("zendo: cannot use the notes repo {}: {e}", n.path.display());
                std::process::exit(1);
            }
        },
        None => None,
    };
    let old_notes = if cfg.notes_repo.is_some() && cfg.root.join(".git").exists() {
        zendo::setup::sync::notes_in(&cfg.root.join("docs/notes")).len()
    } else {
        0
    };
    let mut app = App::new(cfg)?;
    app.files.set_syncer(syncer);
    let stats = zendo::setup::config::UserConfig::load().stats();
    (app.telemetry, app.responsiveness) = (zendo::telemetry::Telemetry::for_process(stats), zendo::responsiveness::Responsiveness::for_process(stats));
    app.keys.legacy = !zendo::tui::kitty();
    let said = start_notices(app.key_help.problems().len(), &app.cfg.agent_env.problems, old_notes);
    app.flash_each(said);
    app.update.watch(update_rx);
    if let Some(l) = live {
        app.link.connect(l.handle, l.events);
        app.link.set_name(l.name);
    }
    Ok(app)
}

/// What zendo says once at the start, a message for each problem: of the keys, of the agent
/// environment, and the notes outside the notes repo. zendo shows them one after another (ENV-7).
fn start_notices(keys: usize, env: &[String], old_notes: usize) -> Vec<String> {
    let keys = (keys > 0).then(|| format!("[keys] {keys} problem(s) · zendo doctor"));
    let notes = (old_notes > 0).then(|| format!("{old_notes} note(s) in docs/notes are not in your notes repo · zendo notes move"));
    keys.into_iter().chain(env.iter().cloned()).chain(notes).collect()
}

/// `U` in the app: upgrade, then start the new zendo in place of this one, with the same arguments.
async fn upgrade_and_restart(exe: Option<PathBuf>) {
    if !doctor::upgrade(false).await {
        std::process::exit(1);
    }
    if let Some(exe) = exe {
        let exe = PathBuf::from(exe.to_string_lossy().trim_end_matches(" (deleted)").to_string());
        println!();
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let err = std::process::Command::new(&exe).args(std::env::args_os().skip(1)).exec();
            eprintln!("zendo: could not start the new version: {err}");
        }
    }
    println!("Start zendo again to use the new version.");
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
