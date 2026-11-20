use super::*;

/// A clean folder of the test `name`: what an interrupted run left goes first.
fn fresh(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zendo-trust-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::canonicalize(&dir).unwrap()
}

/// A repository at `<t>/main` with a `.envrc`, and a worktree of it at `<t>/wt`, outside the main
/// checkout: the files of git, without the program.
fn repository(t: &Path) -> (PathBuf, PathBuf) {
    let (main, wt) = (t.join("main"), t.join("wt"));
    std::fs::create_dir_all(main.join(".git/worktrees/wt")).unwrap();
    std::fs::create_dir_all(&wt).unwrap();
    std::fs::write(main.join(".git/worktrees/wt/commondir"), "../..\n").unwrap();
    std::fs::write(wt.join(".git"), format!("gitdir: {}\n", main.join(".git/worktrees/wt").display())).unwrap();
    std::fs::write(main.join(".envrc"), "FROM_ENVRC=1\n").unwrap();
    (main, wt)
}

/// A run of zendo in `root` with the config file `path`: the answer `answer` of the user in a
/// terminal, or no terminal. The result, and what zendo wrote to the terminal.
fn run(path: &Path, root: &Path, checks: bool, answer: Option<&str>) -> (Result<AgentEnv, String>, String) {
    run_of(path, root, Command { checks, saves: true }, answer)
}

fn run_of(path: &Path, root: &Path, cmd: Command, answer: Option<&str>) -> (Result<AgentEnv, String>, String) {
    let cfg = UserConfig::load_from(path);
    let (mut input, mut out) = (answer.unwrap_or("").as_bytes(), Vec::new());
    let terminal = answer.is_some().then_some(Terminal { input: &mut input, out: &mut out });
    let env = run_env(&cfg, (path, Some(Path::new("/h"))), root, None, cmd, terminal);
    (env, String::from_utf8(out).unwrap())
}

fn from_envrc(env: &AgentEnv) -> Option<&str> {
    env.vars.get("FROM_ENVRC").map(String::as_str)
}

// TRUST-2, TRUST-4, TRUST-5: `zendo asks before it trusts`: in a repository that is not trusted,
// zendo asks before it reads `.envrc`. Another answer than `y` stops zendo and changes nothing.
// `y` adds the main checkout and the worktree outside it to `trusted_dirs`, and keeps the rest of
// the config file; the next start does not ask.
#[test]
fn zendo_asks_before_it_trusts() {
    let t = fresh("ask");
    let (main, wt) = repository(&t);
    let path = t.join("config.toml");
    let before = "# mine\nsound = false\n\n[profile_dirs]\n";
    std::fs::write(&path, before).unwrap();

    let (no, asked) = run(&path, &wt, false, Some("n\n"));
    assert!(no.as_ref().is_err_and(|e| e.contains(&main.display().to_string())), "zendo stops");
    assert!(asked.contains(&main.display().to_string()) && asked.contains(&wt.display().to_string()) && asked.contains("[y/N]"), "{asked}");
    assert!(asked.contains("Trust these folders") && no.is_err_and(|e| e.contains("are not trusted")), "two folders: {asked}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before, "the config file is as before");
    assert!(run(&path, &wt, false, Some("\n")).0.is_err(), "an empty answer is a no");

    // CMD-14: `zendo doctor` without `--fix` writes no file, so its `y` counts for the run.
    let (once, said) = run_of(&path, &wt, Command { checks: true, saves: false }, Some("y\n"));
    assert_eq!((once.as_ref().ok().and_then(from_envrc), std::fs::read_to_string(&path).unwrap().as_str()), (Some("1"), before));
    assert!(said.contains("this run"), "{said}");

    let (yes, _) = run(&path, &wt, false, Some("y\n"));
    let env = yes.unwrap();
    assert_eq!((from_envrc(&env), env.untrusted.is_none()), (Some("1"), true));
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.contains("# mine\nsound = false\n") && after.contains("[profile_dirs]"), "the other content stays: {after}");
    assert_eq!(UserConfig::load_from(&path).trusted_dirs, [main.display().to_string(), wt.display().to_string()]);

    let (again, asked) = run(&path, &wt, false, None);
    assert_eq!((again.as_ref().ok().and_then(from_envrc), asked.as_str()), (Some("1"), ""), "a trusted repository gets no question");
}

// TRUST-3: a folder is trusted with every folder in it; a folder beside it with a longer name is
// not. `~` is the home folder.
#[test]
fn a_folder_is_trusted_with_the_folders_in_it() {
    let cfg = UserConfig { trusted_dirs: vec!["/t/src".into(), "~/own".into()], ..Default::default() };
    let not = |dirs: &[&str]| untrusted(&cfg, &dirs.iter().map(PathBuf::from).collect::<Vec<_>>(), Path::new("/h"));
    assert!(not(&["/t/src", "/t/src/app/.worktrees/4", "/h/own/x"]).is_empty());
    assert_eq!(not(&["/t/srcx", "/t", "/h/other"]), [PathBuf::from("/t/srcx"), PathBuf::from("/t"), PathBuf::from("/h/other")]);
}

// TRUST-1, TRUST-6: without a terminal zendo cannot ask. A command that checks the machine goes on
// without the `.envrc` of the repository, and names the folder and the two ways to trust it; any
// other command stops.
#[test]
fn without_a_terminal_a_check_goes_on_without_the_repository() {
    let t = fresh("checks");
    let (main, wt) = repository(&t);
    let path = t.join("config.toml");
    let (env, _) = run(&path, &main, true, None);
    let env = env.unwrap();
    let said = env.untrusted.clone().unwrap_or_default();
    assert_eq!(from_envrc(&env), None, "no .envrc of a repository that is not trusted");
    assert!(said.contains(&main.display().to_string()) && said.contains("trusted_dirs") && said.contains("start zendo"), "{said}");
    assert!(!path.exists(), "zendo wrote no config file");
    assert!(run(&path, &wt, false, None).0.is_err(), "zendo itself does not start");
}

// TRUST-7, ENV-10: outside a repository, the folder where zendo starts needs the trust: zendo asks
// about it, and reads its `.envrc` after `y`. A folder in a trusted folder gets no question.
#[test]
fn outside_a_repository_zendo_asks_about_its_folder() {
    let t = fresh("outside");
    let path = t.join("config.toml");
    let (at, below) = (t.join("at"), t.join("at/below"));
    std::fs::create_dir_all(&below).unwrap();
    std::fs::write(at.join(".envrc"), "FROM_ENVRC=1\n").unwrap();
    std::fs::write(below.join(".envrc"), "FROM_ENVRC=2\n").unwrap();
    let (no, asked) = run(&path, &at, false, Some("n\n"));
    assert!(no.is_err() && asked.contains(&at.display().to_string()), "{asked}");
    let (yes, _) = run(&path, &at, false, Some("y\n"));
    assert_eq!(yes.as_ref().ok().and_then(from_envrc), Some("1"));
    assert_eq!(UserConfig::load_from(&path).trusted_dirs, [at.display().to_string()]);
    let (inner, asked) = run(&path, &below, false, Some("n\n"));
    assert_eq!((inner.as_ref().ok().and_then(from_envrc), asked.as_str()), (Some("2"), ""), "a folder in a trusted folder");
}

// PROFILE-3: a command that checks the machine goes on without a profile that the config does not
// have, with the `.envrc`, and names the profile; any other command stops.
#[test]
fn a_check_goes_on_without_an_unknown_profile() {
    let t = fresh("profile");
    let (main, _) = repository(&t);
    let path = t.join("config.toml");
    std::fs::write(&path, format!("trusted_dirs = [\"{0}\"]\n\n[profile_dirs]\n\"{0}\" = \"gone\"\n", t.display())).unwrap();
    let (env, _) = run(&path, &main, true, None);
    let env = env.unwrap();
    assert_eq!((env.profile.as_deref(), from_envrc(&env)), (None, Some("1")));
    assert!(env.problems.iter().any(|p| p.contains("`gone`") && p.contains("without a profile")), "{:?}", env.problems);
    assert!(run(&path, &main, false, None).0.is_err_and(|e| e.contains("`gone`")), "zendo itself does not start");
}

// TRUST-8: when zendo cannot write the config file, the answer `y` counts for this run, and zendo
// says so.
#[test]
fn an_answer_that_zendo_cannot_save_counts_for_the_run() {
    let t = fresh("nosave");
    let (main, _) = repository(&t);
    std::fs::write(t.join("file"), "").unwrap();
    let (env, said) = run(&t.join("file/config.toml"), &main, false, Some("y\n"));
    assert_eq!(env.as_ref().ok().and_then(from_envrc), Some("1"));
    assert!(said.contains("this run"), "{said}");
}

// TRUST-5: the home folder and a folder above it hold the other folders of the user, so zendo does
// not save the trust in them: the answer `y` counts for the run. A folder that the user wrote into
// `trusted_dirs` by hand counts.
#[test]
fn the_trust_of_the_home_folder_is_not_saved() {
    let t = fresh("home");
    // The home folder is a link, as on some systems.
    std::fs::create_dir_all(t.join("real/dl/new")).unwrap();
    std::os::unix::fs::symlink(t.join("real"), t.join("u")).unwrap();
    let home = t.join("u");
    std::fs::write(home.join(".envrc"), "FROM_ENVRC=1\n").unwrap();
    let path = t.join("config.toml");
    let at = |dir: &Path, answer: Option<&str>| {
        let cfg = UserConfig::load_from(&path);
        let (mut input, mut out) = (answer.unwrap_or("").as_bytes(), Vec::new());
        let terminal = answer.is_some().then_some(Terminal { input: &mut input, out: &mut out });
        let env = run_env(&cfg, (&path, Some(&home)), dir, None, Command { checks: false, saves: true }, terminal);
        (env, String::from_utf8(out).unwrap())
    };
    for dir in [&home, &t.join("real"), &t] {
        let (env, said) = at(dir, Some("y\n"));
        assert!(env.is_ok() && said.contains("does not save"), "{said}");
        assert!(!path.exists(), "zendo did not save the trust of {}", dir.display());
    }
    // Without a home folder, no folder holds it: zendo saves the trust.
    let cfg = UserConfig::load_from(&path);
    let (mut input, mut out) = ("y\n".as_bytes(), Vec::new());
    let free = run_env(&cfg, (&path, None), &t.join("real/dl"), None, Command { checks: false, saves: true }, Some(Terminal { input: &mut input, out: &mut out }));
    assert!(free.is_ok() && UserConfig::load_from(&path).trusted_dirs == [t.join("real/dl").display().to_string()]);
    std::fs::remove_file(&path).unwrap();
    assert!(at(&home.join("dl/new"), None).0.is_err(), "a folder in the home folder still needs the trust");
    std::fs::write(&path, format!("trusted_dirs = [\"{}\"]\n", home.display())).unwrap();
    assert!(at(&home.join("dl/new"), None).0.is_ok(), "the user wrote the home folder by hand");
}

// TRUST-3: an entry of `trusted_dirs` that is no full path trusts nothing, and zendo names it.
#[test]
fn an_entry_that_is_no_full_path_trusts_nothing() {
    let t = fresh("odd");
    let (main, _) = repository(&t);
    let path = t.join("config.toml");
    std::fs::write(&path, "trusted_dirs = [\"\", \".\", \"src\"]\n").unwrap();
    let (env, _) = run(&path, &main, true, None);
    let env = env.unwrap();
    assert!(env.untrusted.is_some(), "the repository is not trusted");
    assert!(env.problems.iter().any(|p| p.contains("trusted_dirs") && p.contains("`.`")), "{:?}", env.problems);
}
