use anyhow::{Context, Result, bail};
use zendo::app::{App, Config};
use zendo::setup::config::{self, UserConfig};
use zendo::setup::doctor;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;

const HELP: &str = "\
zendo — architecture notes in your repo, with Claude beside them

USAGE
  zendo [--notes DIR] [--agent CMD | --mock | --offline]
                                              open the notes; checks Claude first
  zendo setup [--agent CMD] [--notes-repo DIR|URL|off]
                                              install the Claude adapter once and check everything;
                                              --notes-repo keeps the notes in your private notes repo
  zendo notes move | consolidate [--agent CMD | --mock]
                                              Claude moves docs/notes into the notes repo, or merges
                                              the notes of this project from every machine (by origin)
  zendo doctor [--live] [--fix] [--agent CMD | --mock]
                                              check each step and say what to fix;
                                              --fix repairs project-structure.toml (zendo, then Claude)
  zendo upgrade [--force]                      build the newest main from GitHub, then run setup
  zendo keymap                                 print the default keys: start ~/.zendo/keymap.toml from it

OPTIONS
  --notes DIR   notes folder, relative to the repo root (default: docs/notes, or $ZENDO_NOTES)
  --agent CMD   ACP agent command (default: $ZENDO_AGENT, then ~/.zendo/config.toml,
                then the adapter that `zendo setup` installed, then npx)
  --mock        use the bundled mock agent instead of Claude
  --offline     open the notes without Claude (no start check)
  --live        doctor only: also send one short prompt to Claude
  -V, --version show the version
  -h, --help    show this help

zendo uses the repo root (the nearest folder with .git) as the working folder.
Outside a repo, zendo uses your home folder as a scratchpad.";

enum Run {
    Tui(Config),
    Setup(doctor::Paths, Option<String>, Option<String>),
    Notes(doctor::Paths, Option<String>, String),
    Doctor(doctor::Paths, Option<String>, bool, bool),
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

/// The command line.
#[derive(Default)]
struct Args {
    sub: &'static str,
    notes: String,
    notes_set: bool,
    agent: Option<String>,
    live: bool,
    fix: bool,
    offline: bool,
    notes_repo: Option<String>,
    action: String,
}

fn read_args() -> Result<Args> {
    let mut o = Args { notes: std::env::var("ZENDO_NOTES").unwrap_or_else(|_| "docs/notes".into()), notes_set: std::env::var("ZENDO_NOTES").is_ok(), ..Default::default() };
    let mut args = std::env::args().skip(1).peekable();
    if let Some(first) = args.peek() {
        o.sub = match first.as_str() {
            "setup" => "setup",
            "doctor" => "doctor",
            "upgrade" => "upgrade",
            "keymap" => "keymap",
            "notes" => "notes",
            _ => "",
        };
        if !o.sub.is_empty() {
            args.next();
        }
    }
    let sub = o.sub;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--live" if sub == "doctor" => o.live = true,
            "--fix" if sub == "doctor" => o.fix = true,
            "--force" if sub == "upgrade" => o.live = true,
            "--offline" if sub.is_empty() => o.offline = true,
            "-V" | "--version" | "version" => {
                println!("{}", doctor::version_line());
                std::process::exit(0);
            }
            "-h" | "--help" => {
                println!("{HELP}");
                std::process::exit(0);
            }
            "--notes" => {
                o.notes = args.next().context("--notes needs a folder")?;
                o.notes_set = true;
            }
            "--notes-repo" if sub == "setup" => o.notes_repo = Some(args.next().context("--notes-repo needs a folder or a git URL")?),
            "move" | "consolidate" if sub == "notes" && o.action.is_empty() => o.action = a.clone(),
            "--agent" => o.agent = Some(args.next().context("--agent needs a command")?),
            "--mock" => o.agent = Some(mock_agent()?),
            other => bail!("unknown argument: {other}\n\n{HELP}"),
        }
    }
    Ok(o)
}

/// The mock agent next to this binary.
fn mock_agent() -> Result<String> {
    let exe = std::env::current_exe()?;
    let mock = exe.with_file_name(format!("zendo-mock-agent{}", std::env::consts::EXE_SUFFIX));
    if !mock.exists() {
        bail!("mock agent not found next to zendo: {}", mock.display());
    }
    Ok(config::quote(&mock))
}

fn parse() -> Result<Run> {
    let cwd = std::env::current_dir()?;
    let home = std::env::var_os("HOME").filter(|h| !h.is_empty()).map(PathBuf::from);
    let root = repo_root(&cwd, home.as_deref());
    for (old, new) in config::move_old_folders(&root) {
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
    let paths = doctor::Paths { root: root.clone(), notes_dir: notes_dir.clone(), repo_notes };
    Ok(match a.sub {
        "setup" => Run::Setup(paths, a.agent, a.notes_repo),
        "notes" if a.action.is_empty() => bail!("zendo notes needs move or consolidate\n\n{HELP}"),
        "notes" => Run::Notes(paths, a.agent, a.action),
        "doctor" => Run::Doctor(paths, a.agent, a.live, a.fix),
        "upgrade" => Run::Upgrade(a.live),
        "keymap" => {
            print!("{}", zendo::keys::DEFAULT_KEYMAP);
            std::process::exit(0);
        }
        _ => {
            let (agent, _) = config::resolve_agent(a.agent, &user);
            Run::Tui(Config { root, notes_dir, agent, agent_env: config::agent_env(&user), offline: a.offline, notes_repo: notes_cfg })
        }
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = match parse()? {
        Run::Tui(cfg) => cfg,
        Run::Setup(paths, agent, notes_repo) => std::process::exit(if doctor::setup(&paths, agent, notes_repo).await { 0 } else { 1 }),
        Run::Notes(paths, agent, action) => std::process::exit(if doctor::notes(&paths, agent, &action).await { 0 } else { 1 }),
        Run::Doctor(paths, agent, live, fix) => std::process::exit(if doctor::doctor(&paths, agent, live, fix).await { 0 } else { 1 }),
        Run::Upgrade(force) => std::process::exit(if doctor::upgrade(force).await { 0 } else { 1 }),
    };
    // The path of this binary, before an upgrade replaces it.
    let exe = std::env::current_exe().ok();
    let mut app = open_app(cfg).await?;

    let mut term = ratatui::init();
    zendo::tui::terminal_on();
    let (keys_tx, mut keys) = mpsc::unbounded_channel();
    zendo::tui::spawn_input(keys_tx.clone());

    let result = zendo::tui::run(&mut term, &mut app, &mut keys, keys_tx).await;

    if app.mode == zendo::app::Mode::Write {
        app.save();
    }
    app.end_visit();
    zendo::tui::terminal_off();
    ratatui::restore();
    if let Some(s) = app.syncer.take()
        && let Some(label) = s.finish().label() {
            eprintln!("zendo: {label}");
        }
    result?;
    if app.upgrade_now {
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
        match doctor::preflight(&cfg.agent, &cfg.agent_env, &cfg.root, log.join("agent.log")).await {
            Some(l) => Some(l),
            None => std::process::exit(1),
        }
    };
    // The notes repo: this folder, its location file, and the sync in the background.
    let syncer = match &cfg.notes_repo {
        Some(n) => match n.prepare(&cfg.root) {
            Ok(dir) => Some(zendo::setup::sync::Syncer::start(n.path.clone(), dir, &n.machine, Duration::from_secs(n.sync_seconds.max(1)))),
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
    app.syncer = syncer;
    if !app.key_problems.is_empty() {
        app.flash(format!("[keys] in the config: {} problem(s) · zendo doctor", app.key_problems.len()));
    }
    if old_notes > 0 {
        app.flash(format!("{old_notes} note(s) in docs/notes are not in your notes repo · zendo notes move"));
    }
    app.update_rx = Some(update_rx);
    if let Some(l) = live {
        app.agent = Some(l.handle);
        app.agent_events = Some(l.events);
        app.agent_name = Some(l.name);
    }
    Ok(app)
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
