//! The agent environment (docs/zendo/agent-env.md) and the threads of the profiles
//! (docs/zendo/agent-profiles.md): every agent that zendo starts gets the same variables, also in a
//! worktree whose `.envrc` git ignores.

mod common;
use common::*;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use zendo::agent::api::{Api, Asker, Question};
use zendo::app::{App, Config, Focus, session_key};
use zendo::session::Command;
use zendo::setup::config::{self, AgentEnv, UserConfig};

/// A clean folder of the test `name`: what an interrupted run left goes first.
fn fresh(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zendo-agent-env-it-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::canonicalize(&dir).unwrap()
}

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git").args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "init.defaultBranch=main"]).args(args).current_dir(dir).output().unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

/// A program at `path` that writes the probe line of its environment to `<path>.env`, answers
/// `auth status` with a login, and answers a question of `claude -p`.
fn probe(path: &Path) -> PathBuf {
    let script = "#!/bin/sh\nprintf 'probe=%s:%s:%s\\n' \"$CLAUDE_CONFIG_DIR\" \"$FROM_ENVRC\" \"$HOME\" >> \"$0.env\"\n\
        [ \"$1 $2\" = \"auth status\" ] && { echo '{\"loggedIn\": true}'; exit 0; }\n\
        read -r line\necho '{\"type\": \"result\", \"subtype\": \"success\", \"is_error\": false, \"result\": \"ok\"}'\n";
    std::fs::write(path, script).unwrap();
    let mut p = std::fs::metadata(path).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut p, 0o755);
    std::fs::set_permissions(path, p).unwrap();
    path.with_extension("env")
}

fn wait_for(secs: u64, mut done: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(secs);
    while !done() {
        assert!(Instant::now() < end, "timeout");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The app of the worktree `wt`, with the agent environment `env`, as `main` makes it.
fn app_in(wt: &Path, env: AgentEnv) -> App {
    let cfg = Config { root: wt.to_path_buf(), notes_dir: wt.join("docs/notes"), agent: env!("CARGO_BIN_EXE_zendo-mock-agent").into(), agent_env: env, offline: false, notes_repo: None, user_config: wt.join("zendo-config.toml") };
    App::new(cfg).unwrap()
}

// ENV-1, ENV-2, ENV-3: `agent env reaches every agent`: with `--profile work`, a session in a
// worktree, the chat agent, the light model and its login check get the profile over the gitignored
// `.envrc` of the main checkout, over the environment of zendo.
#[tokio::test(flavor = "multi_thread")]
async fn agent_env_reaches_every_agent() {
    let base = fresh("every");
    let main = base.join("main");
    std::fs::create_dir_all(&main).unwrap();
    git(&main, &["init", "-q"]);
    std::fs::write(main.join(".gitignore"), ".envrc\n").unwrap();
    git(&main, &["add", ".gitignore"]);
    git(&main, &["commit", "-q", "-m", "init"]);
    git(&main, &["worktree", "add", "-q", "-b", "wt", "../wt"]);
    std::fs::write(main.join(".envrc"), "export CLAUDE_CONFIG_DIR=A\nFROM_ENVRC=1\n").unwrap();
    let (cfg, _) = UserConfig::read_from(&write(&base.join("config.toml"), "[profiles.work]\nCLAUDE_CONFIG_DIR = \"B\"\n"));
    let wt = base.join("wt");
    let home = std::env::var("HOME").unwrap_or_default();
    let want = |dir: &str| format!("probe={dir}:1:{home}");
    assert!(config::agent_env(&cfg, &wt, None).unwrap().vars.get("CLAUDE_CONFIG_DIR").is_some_and(|v| v == "A"), "without a profile, .envrc");
    let env = config::agent_env(&cfg, &wt, Some("work")).unwrap();

    // A Claude Code session in a terminal of zendo, in the worktree.
    let mut a = app_in(&wt, env.clone());
    let script = "printf 'probe=%s:%s:%s\\n' \"$CLAUDE_CONFIG_DIR\" \"$FROM_ENVRC\" \"$HOME\"; sleep 5".to_string();
    a.cli.command = Some(Command { program: "/bin/sh".into(), args: vec!["-c".into(), script], claude: false });
    a.thread(&session_key(0)).session = Some("s1".into());
    a.switch_session(0);
    a.focus = Focus::Claude;
    ctrl(&mut a, 'o');
    let end = Instant::now() + Duration::from_secs(10);
    while !a.cli.text(0).contains(&want("B")) {
        assert!(Instant::now() < end, "the session got no variables: {}", a.cli.text(0));
        a.tick();
        std::thread::sleep(Duration::from_millis(20));
    }

    // The light model: its login check and a question (`claude -p`).
    let light = probe(&base.join("claude"));
    let api = Api { path: base.join("claude"), model: "m".into(), effort: None, env: env.vars.clone() };
    assert!(api.usable());
    let mut asker = Asker::default();
    asker.ask(&api, Question::Keys(1), "q".into(), None);
    wait_for(10, || !asker.answers().is_empty());
    let lines = std::fs::read_to_string(&light).unwrap();
    assert_eq!(lines.lines().collect::<Vec<_>>(), [want("B"), want("B")], "{lines}");

    // The agent of the chat (ACP): the program starts with the variables, then stops.
    let acp = probe(&base.join("acp"));
    let (_h, _rx) = zendo::agent::spawn(base.join("acp").display().to_string(), env.vars.clone(), wt.clone(), None);
    wait_for(10, || std::fs::read_to_string(&acp).is_ok_and(|t| t.contains(&want("B"))));
}

fn write(path: &Path, text: &str) -> PathBuf {
    std::fs::write(path, text).unwrap();
    path.to_path_buf()
}

// PROFILE-4, PROFILE-6: `threads are kept per profile`: the thread of a profile resumes only with
// that profile, and a thread that zendo recorded before profiles is the thread without one.
#[test]
fn threads_are_kept_per_profile() {
    let root = repo("agent-env-threads", &[("docs/notes/a.md", "# A\n")]);
    let key = session_key(0);
    let with = |profile: Option<&str>| app_in(&root, AgentEnv { profile: profile.map(str::to_string), ..Default::default() });
    let mut old = with(None);
    old.state.set_session(&old.cfg.threads_key(), &key, "before-profiles".into());
    old.state.save(&root);
    let mut work = with(Some("work"));
    assert_eq!(work.state.session(&work.cfg.threads_key(), &key), None, "the thread without a profile is not the one of work");
    work.state.set_session(&work.cfg.threads_key(), &key, "of-work".into());
    work.state.save(&root);
    let none = with(None);
    assert_eq!(none.state.session(&none.cfg.threads_key(), &key).as_deref(), Some("before-profiles"));
    let work = with(Some("work"));
    assert_eq!(work.state.session(&work.cfg.threads_key(), &key).as_deref(), Some("of-work"));
}

// ENV-1, TERM-55: the shell of a terminal of zendo gets the agent environment, so a `claude` that
// the user starts there uses the same account; the variables that select the startup files of the
// shell stay as they are.
#[tokio::test(flavor = "multi_thread")]
async fn a_terminal_shell_gets_the_agent_env() {
    let root = repo("agent-env-shell", &[("docs/notes/a.md", "# A\n")]);
    let vars = [("FROM_ENVRC", "1"), ("ZDOTDIR", "/zendo-no-such"), ("BASH_ENV", "/zendo-no-such")];
    let mut a = app_in(&root, AgentEnv { vars: vars.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(), ..Default::default() });
    a.cli.shell = Some("/bin/sh".into());
    a.cli.open_shell(0, &root).unwrap();
    let n = zendo::session::shell(0);
    a.cli.write(n, b"echo got=$FROM_ENVRC:$ZDOTDIR:$BASH_ENV=\r");
    let want = format!("got=1:{}:{}=", std::env::var("ZDOTDIR").unwrap_or_default(), std::env::var("BASH_ENV").unwrap_or_default());
    let end = Instant::now() + Duration::from_secs(10);
    while !a.cli.text(n).contains(&want) {
        assert!(Instant::now() < end, "the shell got other variables than {want}: {}", a.cli.text(n));
        a.tick();
        std::thread::sleep(Duration::from_millis(20));
    }
}

// PROFILE-8: the name of the profile stands at the left end of the top line, before the path, as a
// badge in the blue of the theme, as the name of the pane in the status bar; without a profile the
// top line has no badge.
#[test]
fn the_profile_shows_at_the_top() {
    let root = repo("agent-env-top", &[("docs/notes/a.md", "# A\n")]);
    let mut a = app_in(&root, AgentEnv { profile: Some("work".into()), ..Default::default() });
    let buf = rekey::frame(&mut a);
    let (blue, ink) = (a.theme().ext.doc, a.theme().bg);
    let (x, _) = rekey::find(&buf, " work ", Some(0)).expect("the profile in the top line");
    let path = rekey::find(&buf, "docs/notes", Some(0)).expect("the path in the top line").0;
    assert!(x == 2 && x < path, "the badge is first: {x}, the path at {path}");
    for dx in 0..6 {
        let cell = &buf[(x + dx, 0)];
        assert_eq!((cell.bg, cell.fg), (blue, ink), "cell {dx} of the badge");
    }
    let mut plain = app_in(&root, AgentEnv::default());
    let buf = rekey::frame(&mut plain);
    assert!((0..buf.area.width).all(|x| buf[(x, 0)].bg != blue), "no badge without a profile");
    assert_eq!(rekey::find(&buf, "docs/notes", Some(0)).map(|p| p.0), Some(2), "the path stays at the left end");
}
