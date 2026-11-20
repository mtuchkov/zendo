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

fn git(dir: &Path, args: &[&str]) {
    let ok = std::process::Command::new("git").args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "init.defaultBranch=main"]).args(args).current_dir(dir).output().unwrap();
    assert!(ok.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&ok.stderr));
}

// ENV-4, ENV-7: `envrc is read not run`: zendo takes only the variable lines, as they are written,
// runs nothing, and names the line that it skipped once.
#[test]
fn envrc_is_read_not_run() {
    let dir = fresh("read");
    let text = "export A=1\nB='two words'\n# a comment\nC=\"$HOME\"\ntouch marker\n\nexport D=~/x\n";
    std::fs::write(dir.join(".envrc"), text).unwrap();
    let env = build(&UserConfig::default(), &dir, None, &dir, None).unwrap();
    let get = |k: &str| env.vars.get(k).map(String::as_str);
    assert_eq!((get("A"), get("B"), get("C"), get("D")), (Some("1"), Some("two words"), Some("$HOME"), Some("~/x")));
    assert_eq!(env.vars.len(), 4, "{:?}", env.vars);
    assert!(!dir.join("marker").exists(), "zendo ran no line of .envrc");
    assert_eq!(env.problems.len(), 1, "one message: {:?}", env.problems);
    assert!(env.problems[0].contains(".envrc") && env.problems[0].contains("line 5"), "{:?}", env.problems);
}

// ENV-4: a line that the shell would run, or that is not a whole assignment, is no variable.
#[test]
fn lines_that_run_code_are_skipped() {
    let (vars, skipped) = read_envrc("A=$(id)\nB=`id`\nC=1; id\nD=a b\n1E=2\nexport\nF='open\nexportG=1\nexport\tH=2\n");
    assert_eq!(vars.into_iter().collect::<Vec<_>>(), [("H".to_string(), "2".to_string()), ("exportG".to_string(), "1".to_string())]);
    assert_eq!(skipped, [1, 2, 3, 4, 5, 6, 7]);
}

// ENV-2: the profile is over `.envrc`, and the variables of `.envrc` that the profile does not set
// stay.
#[test]
fn profile_is_over_envrc() {
    let dir = fresh("order");
    std::fs::write(dir.join(".envrc"), "CLAUDE_CONFIG_DIR=A\nFROM_ENVRC=1\n").unwrap();
    let cfg = config("[profiles.work]\nCLAUDE_CONFIG_DIR = \"~/.claude-work\"\n");
    let env = build(&cfg, &dir, Some("work"), Path::new("/h"), None).unwrap();
    assert_eq!(env.vars.get("CLAUDE_CONFIG_DIR").map(String::as_str), Some("/h/.claude-work"), "`~/` of a profile is the home folder");
    assert_eq!(env.vars.get("FROM_ENVRC").map(String::as_str), Some("1"));
    assert_eq!(env.profile.as_deref(), Some("work"));
    let env = build(&cfg, &dir, None, Path::new("/h"), None).unwrap();
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
    let env = agent_env(&UserConfig::default(), &base.join("wt"), None).unwrap();
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
// profile, and keeps the options that are there.
#[test]
fn system_ca_goes_with_every_profile() {
    let dir = fresh("ca");
    let cfg = config("system_ca = true\n\n[profiles.work]\nNODE_OPTIONS = \"--max-old-space-size=99\"\n");
    let node = |flag| build(&cfg, &dir, flag, &dir, Some("--trace".into())).unwrap().vars.get("NODE_OPTIONS").cloned();
    assert_eq!(node(None).as_deref(), Some("--trace --use-system-ca"));
    assert_eq!(node(Some("work")).as_deref(), Some("--max-old-space-size=99 --use-system-ca"));
    let again = build(&cfg, &dir, None, &dir, Some("--use-system-ca".into())).unwrap();
    assert!(!again.vars.contains_key("NODE_OPTIONS"), "zendo has it already, and the agents get it from zendo");
    let off = build(&UserConfig::default(), &dir, None, &dir, Some("--trace".into())).unwrap();
    assert!(!off.vars.contains_key("NODE_OPTIONS"), "without system_ca zendo adds nothing");
}
