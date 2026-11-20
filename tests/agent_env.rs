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
    let config_file = write(&base.join("config.toml"), &format!("trusted_dirs = [\"{}\"]\n\n[profiles.work]\nCLAUDE_CONFIG_DIR = \"B\"\n", base.display()));
    let (cfg, _) = UserConfig::read_from(&config_file);
    let wt = base.join("wt");
    let home = std::env::var("HOME").unwrap_or_default();
    let want = |dir: &str| format!("probe={dir}:1:{home}");
    // As `main` builds it: in a trusted folder, without a terminal.
    let run = config::trust::Command { checks: false, saves: true };
    let env_of = |flag| config::trust::run_env(&cfg, (&config_file, Some(Path::new(&home))), &wt, flag, run, None).unwrap();
    assert!(env_of(None).vars.get("CLAUDE_CONFIG_DIR").is_some_and(|v| v == "A"), "without a profile, .envrc");
    let env = env_of(Some("work"));

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

// ENV-1, ENV-5, TERM-55: the shell of a terminal of zendo gets the agent environment, so a
// `claude` that the user starts there uses the same account. The variables that select the startup
// files of the shell stay as they are, and the shell does not get the system CA that zendo adds
// for the agents.
#[tokio::test(flavor = "multi_thread")]
async fn a_terminal_shell_gets_the_agent_env() {
    let root = repo("agent-env-shell", &[("docs/notes/a.md", "# A\n")]);
    let vars = [("FROM_ENVRC", "1"), ("ZDOTDIR", "/zendo-no-such"), ("BASH_ENV", "/zendo-no-such"), ("NODE_OPTIONS", "--trace --use-system-ca")];
    let env = AgentEnv { vars: vars.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(), node_before_ca: Some(Some("--trace".into())), ..Default::default() };
    let mut a = app_in(&root, env);
    a.cli.shell = Some("/bin/sh".into());
    a.cli.open_shell(0, &root).unwrap();
    let n = zendo::session::shell(0);
    a.cli.write(n, b"echo got=$FROM_ENVRC:$ZDOTDIR:$BASH_ENV:$NODE_OPTIONS=\r");
    let want = format!("got=1:{}:{}:--trace=", std::env::var("ZDOTDIR").unwrap_or_default(), std::env::var("BASH_ENV").unwrap_or_default());
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

/// Run `zendo doctor` with the agent command `agent` in `root`, with `home` as the home folder and
/// no terminal. Returns the output.
fn doctor(root: &Path, home: &Path, agent: &Path) -> String {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_zendo"))
        .args(["doctor", "--agent-cmd", &agent.display().to_string()])
        .current_dir(root)
        .env("HOME", home)
        .env("ZENDO_NO_UPDATE_CHECK", "1")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .env_remove("CLAUDE_CONFIG_DIR")
        .env_remove("CODEX_HOME")
        .env_remove("FROM_ENVRC")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

// TRUST-1, TRUST-3, TRUST-6: `an untrusted repository gives nothing to the agents`: without a
// terminal, `zendo doctor` in a repository that is not trusted names the folder, reads no `.envrc`
// and starts its agent outside the repository. After the user trusts a folder above, the agent
// starts in the repository with the variables of `.envrc`.
#[test]
fn an_untrusted_repository_gives_nothing_to_the_agents() {
    let base = fresh("trust");
    let (root, home) = (base.join("src/app"), base.join("home"));
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(home.join(".zendo")).unwrap();
    std::fs::write(root.join(".envrc"), "export FROM_ENVRC=1\n").unwrap();
    // An agent that writes where it started and what it got, and ends.
    let agent = base.join("agent");
    let seen = base.join("seen");
    std::fs::write(&agent, format!("#!/bin/sh\nprintf '%s|%s' \"$PWD\" \"$FROM_ENVRC\" > '{}'\n", seen.display())).unwrap();
    let mut p = std::fs::metadata(&agent).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut p, 0o755);
    std::fs::set_permissions(&agent, p).unwrap();

    let out = doctor(&root, &home, &agent);
    assert!(out.contains("is not trusted") && out.contains(&root.display().to_string()) && out.contains("trusted_dirs"), "{out}");
    let (cwd, var) = std::fs::read_to_string(&seen).unwrap().split_once('|').map(|(a, b)| (PathBuf::from(a), b.to_string())).unwrap();
    assert!(!cwd.starts_with(&root) && var.is_empty(), "the agent started in {} with FROM_ENVRC={var}", cwd.display());

    std::fs::write(home.join(".zendo/config.toml"), format!("trusted_dirs = [\"{}\"]\n", base.join("src").display())).unwrap();
    let out = doctor(&root, &home, &agent);
    assert!(!out.contains("is not trusted"), "{out}");
    assert_eq!(std::fs::read_to_string(&seen).unwrap(), format!("{}|1", root.display()));
}

// ENV-7: the messages of the start show one after another, each for its time, so that no message
// hides another and none is cut to make room.
#[test]
fn the_messages_of_the_start_show_one_after_another() {
    let root = repo("agent-env-said", &[("docs/notes/a.md", "# A\n")]);
    let mut a = app_in(&root, AgentEnv::default());
    let ends = |a: &mut App| {
        a.flash.as_mut().unwrap().1 -= Duration::from_secs(5);
        a.tick();
    };
    a.flash_each(["[keys] 1 problem(s)".to_string(), ".envrc: zendo skipped line 5".to_string()]);
    assert_eq!(a.flash.as_ref().map(|f| f.0.as_str()), Some("[keys] 1 problem(s)"));
    ends(&mut a);
    assert_eq!(a.flash.as_ref().map(|f| f.0.as_str()), Some(".envrc: zendo skipped line 5"));
    ends(&mut a);
    assert!(a.flash.is_none());
}
