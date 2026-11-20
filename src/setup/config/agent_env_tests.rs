use super::*;

/// A clean folder of the test `name`: what an interrupted run left goes first.
fn fresh(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zendo-agent-env-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::canonicalize(&dir).unwrap()
}

fn config(text: &str) -> UserConfig {
    let (cfg, problems) = super::super::read::read_text(text);
    assert!(problems.is_empty(), "{problems:?}");
    cfg
}

/// The environment of zendo in a test: `HOME=/h`, and `node` as its `NODE_OPTIONS`.
fn env_with(node: Option<&'static str>) -> impl Fn(&str) -> Option<String> {
    move |k: &str| match k {
        "HOME" => Some("/h".to_string()),
        "NODE_OPTIONS" => node.map(str::to_string),
        _ => None,
    }
}

fn git(dir: &Path, args: &[&str]) {
    let ok = std::process::Command::new("git").args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "init.defaultBranch=main"]).args(args).current_dir(dir).output().unwrap();
    assert!(ok.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&ok.stderr));
}

// ENV-4, ENV-7, ENV-8: `envrc is read not run`: zendo takes only the variable lines, puts the
// values of `$NAME` itself, runs nothing, and names the line that it skipped once.
#[test]
fn envrc_is_read_not_run() {
    let dir = fresh("read");
    let text = "export A=1\nB='two words'\n# a comment\nC=\"$HOME/c\"\ntouch marker\n\nexport D='$HOME ~'\n";
    std::fs::write(dir.join(".envrc"), text).unwrap();
    let env = build(&UserConfig::default(), &dir, Some(&dir), None, Path::new("/h"), &env_with(None)).unwrap();
    let get = |k: &str| env.vars.get(k).map(String::as_str);
    assert_eq!((get("A"), get("B"), get("C"), get("D")), (Some("1"), Some("two words"), Some("/h/c"), Some("$HOME ~")));
    assert_eq!(env.vars.len(), 4, "{:?}", env.vars);
    assert!(!dir.join("marker").exists(), "zendo ran no line of .envrc");
    assert_eq!(env.problems.len(), 1, "one message: {:?}", env.problems);
    assert!(env.problems[0].contains(".envrc") && env.problems[0].contains("line 5"), "{:?}", env.problems);
}

// ENV-2: the profile is over `.envrc`, and the variables of `.envrc` that the profile does not set
// stay.
#[test]
fn profile_is_over_envrc() {
    let dir = fresh("order");
    std::fs::write(dir.join(".envrc"), "CLAUDE_CONFIG_DIR=A\nFROM_ENVRC=1\n").unwrap();
    let cfg = config("[profiles.work]\nCLAUDE_CONFIG_DIR = \"~/.claude-work\"\n");
    let env = build(&cfg, &dir, Some(&dir), Some("work"), Path::new("/h"), &env_with(None)).unwrap();
    assert_eq!(env.vars.get("CLAUDE_CONFIG_DIR").map(String::as_str), Some("/h/.claude-work"), "`~/` of a profile is the home folder");
    assert_eq!(env.vars.get("FROM_ENVRC").map(String::as_str), Some("1"));
    assert_eq!(env.profile.as_deref(), Some("work"));
    let env = build(&cfg, &dir, Some(&dir), None, Path::new("/h"), &env_with(None)).unwrap();
    assert_eq!((env.vars.get("CLAUDE_CONFIG_DIR").map(String::as_str), env.profile), (Some("A"), None));
}

// ENV-3: in a worktree outside the repository, the gitignored `.envrc` of the main checkout counts,
// and a `.envrc` of the worktree does not.
#[test]
fn envrc_of_the_main_checkout_reaches_a_worktree() {
    let base = fresh("worktree");
    let main = base.join("main");
    std::fs::create_dir_all(&main).unwrap();
    git(&main, &["init", "-q"]);
    std::fs::write(main.join(".gitignore"), ".envrc\n").unwrap();
    git(&main, &["add", ".gitignore"]);
    git(&main, &["commit", "-q", "-m", "init"]);
    git(&main, &["worktree", "add", "-q", "-b", "wt", "../wt"]);
    std::fs::write(main.join(".envrc"), "export FROM_MAIN=1\n").unwrap();
    std::fs::write(base.join("wt/.envrc"), "export FROM_WORKTREE=1\n").unwrap();
    let main = main_checkout(&base.join("wt")).unwrap();
    let env = build(&UserConfig::default(), &main, Some(&main), None, Path::new("/h"), &env_with(None)).unwrap();
    assert_eq!(env.vars.get("FROM_MAIN").map(String::as_str), Some("1"), "{:?}", env.vars);
    assert!(!env.vars.contains_key("FROM_WORKTREE"), "{:?}", env.vars);
}

// PROFILE-1, PROFILE-2: `profile dirs select nearest`: the nearest mapped folder of the main
// checkout gives the profile, no folder gives none, and `--profile` goes over the map.
#[test]
fn profile_dirs_select_nearest() {
    let tmp = fresh("dirs");
    let t = tmp.display();
    let cfg = config(&format!("[profiles.home]\nA = \"1\"\n\n[profiles.work]\nA = \"2\"\n\n[profile_dirs]\n\"{t}/src\" = \"home\"\n\"~/work\" = \"work\"\n"));
    let pick = |main: &str, flag| select_profile(&cfg, flag, &tmp.join(main), &tmp.join("src"));
    assert_eq!(pick("src/work/app", None), Ok(Some("work".into())), "`~/work` is nearer than `src`");
    assert_eq!(pick("src/other", None), Ok(Some("home".into())));
    assert_eq!(pick("other", None), Ok(None));
    assert_eq!(pick("src/work/app", Some("home")), Ok(Some("home".into())));
    assert_eq!(pick("src/workshop", None), Ok(Some("home".into())), "a folder counts by its whole name");
}

// PROFILE-3: `unknown profile stops start`: a profile that the config does not have, from the flag
// or from the map, is an error that names it and the config file.
#[test]
fn unknown_profile_stops_start() {
    let cfg = config("[profile_dirs]\n\"/r\" = \"gone\"\n");
    let err = select_profile(&cfg, Some("home"), Path::new("/x"), Path::new("/h")).unwrap_err();
    assert!(err.contains("`home`") && err.contains("config.toml"), "{err}");
    let err = select_profile(&cfg, None, Path::new("/r/app"), Path::new("/h")).unwrap_err();
    assert!(err.contains("`gone`") && err.contains("[profile_dirs]"), "{err}");
}

// ENV-5: `system_ca` adds the option of the system CA to NODE_OPTIONS once, with and without a
// profile, and keeps the options that are there. The shell of a terminal does not get it.
#[test]
fn system_ca_goes_with_every_profile() {
    let dir = fresh("ca");
    let cfg = config("system_ca = true\n\n[profiles.work]\nNODE_OPTIONS = \"--max-old-space-size=99\"\n");
    let of = |cfg: &UserConfig, flag, node| build(cfg, &dir, Some(&dir), flag, &dir, &env_with(node)).unwrap();
    let node = |e: &AgentEnv| (e.vars.get("NODE_OPTIONS").cloned(), e.shell_vars().get("NODE_OPTIONS").cloned());
    assert_eq!(node(&of(&cfg, None, Some("--trace"))), (Some("--trace --use-system-ca".into()), None), "the shell keeps the value of zendo");
    assert_eq!(node(&of(&cfg, Some("work"), Some("--trace"))), (Some("--max-old-space-size=99 --use-system-ca".into()), Some("--max-old-space-size=99".into())));
    assert_eq!(node(&of(&cfg, None, Some("--use-system-ca"))), (None, None), "zendo has it already, and the agents get it from zendo");
    assert_eq!(node(&of(&UserConfig::default(), None, Some("--trace"))), (None, None), "without system_ca zendo adds nothing");
}

// ENV-3: zendo finds the main checkout from the files of git, without a program: the
// checkout itself, the main checkout of a worktree, and the folder itself for a submodule and for
// a worktree of a bare repository. Outside a repository there is none.
#[test]
fn the_main_checkout_needs_no_git() {
    let t = fresh("checkout");
    let dir = |p: &str| {
        std::fs::create_dir_all(t.join(p)).unwrap();
        t.join(p)
    };
    let file = |p: &str, text: String| std::fs::write(t.join(p), text).unwrap();
    dir("plain/.git/worktrees/wt");
    dir("plain/.git/modules/sub");
    dir("bare.git/worktrees/b");
    file("plain/.git/worktrees/wt/commondir", "../..\n".into());
    file("bare.git/worktrees/b/commondir", "../..\n".into());
    let (wt, sub, b, deep) = (dir("wt"), dir("plain/sub"), dir("b"), dir("plain/docs/notes"));
    file("wt/.git", format!("gitdir: {}\n", t.join("plain/.git/worktrees/wt").display()));
    file("plain/sub/.git", "gitdir: ../.git/modules/sub\n".into());
    file("b/.git", format!("gitdir: {}\n", t.join("bare.git/worktrees/b").display()));
    let plain = Some(t.join("plain"));
    assert_eq!(main_checkout(&t.join("plain")), plain);
    assert_eq!(main_checkout(&deep), plain, "from a folder of the checkout");
    assert_eq!(main_checkout(&wt), plain, "the main checkout of a worktree");
    assert_eq!(main_checkout(&sub), Some(sub.clone()), "a submodule is a checkout of its own");
    assert_eq!(main_checkout(&b), Some(b.clone()), "a bare repository has no main checkout");
    let out = dir("out");
    assert_eq!(main_checkout(&out), None, "outside a repository");
}

// ENV-8: `$PWD` in `.envrc` is the folder of the file, as when a shell runs the file there, not
// the folder where zendo started.
#[test]
fn pwd_is_the_folder_of_envrc() {
    let dir = fresh("pwd");
    std::fs::write(dir.join(".envrc"), "export TOOLS=\"$PWD/tools\"\n").unwrap();
    let env = build(&UserConfig::default(), &dir, Some(&dir), None, Path::new("/h"), &env_with(None)).unwrap();
    assert_eq!(env.vars.get("TOOLS").cloned(), Some(format!("{}/tools", dir.display())));
}
