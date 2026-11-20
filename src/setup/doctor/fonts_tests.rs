use super::*;
use std::os::unix::fs::PermissionsExt;

fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-font-session-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A mock agent: it keeps its first message in `log` and writes `recipe` into the recipe file.
fn mock(d: &Path, recipes: &Path, recipe: &str) -> PathBuf {
    let path = d.join("mock-agent");
    let script = format!("#!/bin/sh\nprintf '%s\\n---\\n' \"$1\" >> '{log}'\nprintf '%s\\n' '{recipe}' > '{out}'\n", log = d.join("log").display(), out = recipes.display());
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn starts(d: &Path) -> Vec<String> {
    std::fs::read_to_string(d.join("log")).unwrap_or_default().split("\n---\n").filter(|s| !s.trim().is_empty()).map(String::from).collect()
}

/// No variables over the environment of zendo.
static NO_ENV: BTreeMap<String, String> = BTreeMap::new();

fn session<'a>(d: &'a Path, recipes: &'a Path) -> Session<'a> {
    Session { program: "claude", recipes, terminal: "test-term", cwd: d, env: &NO_ENV }
}

// ENV-1, PROFILE-7: the agent that writes the font recipe starts with the agent environment, so it
// uses the account of the profile.
#[test]
fn the_agent_of_the_font_recipe_gets_the_agent_env() {
    let d = folder("env");
    let agent = d.join("probe");
    std::fs::write(&agent, format!("#!/bin/sh\nprintf '%s' \"$CLAUDE_CONFIG_DIR\" > '{}'\n", d.join("seen").display())).unwrap();
    std::fs::set_permissions(&agent, std::fs::Permissions::from_mode(0o755)).unwrap();
    let env = BTreeMap::from([("CLAUDE_CONFIG_DIR".to_string(), "/of-work".to_string())]);
    let recipes = d.join("terminal-fonts.toml");
    let s = Session { env: &env, ..session(&d, &recipes) };
    s.start(&Out::new(), &|_| Some(agent.clone())).unwrap();
    assert_eq!(std::fs::read_to_string(d.join("seen")).unwrap(), "/of-work");
    let _ = std::fs::remove_dir_all(&d);
}

// FONT-22, FONT-26, FONT-27, FONT-28: no question and no agent without a terminal; no agent after
// no; after yes the agent starts once with the terminal, the fonts, the recipe file and its format,
// and the recipe works at its end.
#[test]
fn setup_fonts_starts_the_agent_for_the_recipe_after_yes() {
    let d = folder("yes");
    let recipes = d.join("terminal-fonts.toml");
    let agent = mock(&d, &recipes, "[test-term]\nescape = \"x{font}\"");
    let s = session(&d, &recipes);
    let found = |_: &str| Some(agent.clone());
    let o = Out::new();
    assert_eq!(recipe_step(&o, &mut "y\n".as_bytes(), false, &s, &found), Step::NotTerminal);
    assert_eq!(recipe_step(&o, &mut "n\n".as_bytes(), true, &s, &found), Step::Declined);
    assert!(starts(&d).is_empty(), "no agent without a terminal and after no");
    assert_eq!(recipe_step(&o, &mut "y\n".as_bytes(), true, &s, &found), Step::Works);
    let first = starts(&d);
    assert_eq!(first.len(), 1, "{first:?}");
    let m = &first[0];
    for want in ["test-term", &recipes.display().to_string(), "iA Writer Mono S", "Iosevka Term", "prefix", "whole", "command", "escape", "reload", "{size}", "ask me"] {
        assert!(m.contains(want), "the first message names {want}: {m}");
    }
    assert_eq!(recipe_step(&o, &mut "y\n".as_bytes(), true, &s, &found), Step::Works, "a working recipe needs no session");
    assert_eq!(starts(&d).len(), 1);
    let _ = std::fs::remove_dir_all(&d);
}

// FONT-33: without the command of the agent on the PATH, no session starts.
#[test]
fn a_missing_agent_names_how_to_install_it() {
    let d = folder("missing");
    let recipes = d.join("terminal-fonts.toml");
    let s = session(&d, &recipes);
    assert_eq!(recipe_step(&Out::new(), &mut "y\n".as_bytes(), true, &s, &|_| None), Step::NoAgent);
    let _ = std::fs::remove_dir_all(&d);
}

// FONT-44: Codex gets `codex`, every other agent `claude`.
#[test]
fn the_session_starts_the_command_of_the_agent() {
    assert_eq!(agent_program(config::Kind::Codex), "codex");
    assert_eq!(agent_program(config::Kind::Claude), "claude");
    assert_eq!(agent_program(config::Kind::Other), "claude");
}

// FONT-23, FONT-24, FONT-25, FONT-31: doctor names a broken recipe and the repair, starts no agent
// and changes no file; with `--fix --agent` the agent starts once and the recipe works again.
#[test]
fn doctor_checks_the_recipe_and_repairs_it_with_claude() {
    let d = folder("doctor");
    let recipes = d.join("terminal-fonts.toml");
    let broken = format!("[test-term]\nfile = \"{}\"\nprefix = \"font\"\nline = \"font = {{font}}\"\n", d.join("gone.conf").display());
    std::fs::write(&recipes, &broken).unwrap();
    let agent = mock(&d, &recipes, "[test-term]\nescape = \"x{font}\"");
    let s = session(&d, &recipes);
    let found = |_: &str| Some(agent.clone());
    assert!(!recipe_doctor(&Out::new(), &s, false, &found), "a broken recipe");
    assert!(starts(&d).is_empty(), "plain doctor starts no agent");
    assert_eq!(std::fs::read_to_string(&recipes).unwrap(), broken, "plain doctor changes no file");
    assert!(recipe_doctor(&Out::new(), &s, true, &found), "the agent repaired it");
    assert_eq!(starts(&d).len(), 1);
    assert!(recipe_doctor(&Out::new(), &session(&d, &d.join("none.toml")), false, &found), "no recipe is no failure");
    let _ = std::fs::remove_dir_all(&d);
}

// ENV-1: zendo looks for the program of the agent on the PATH of the agent environment.
#[test]
fn the_program_of_the_agent_is_on_the_path_of_the_agent_env() {
    let d = folder("path");
    std::fs::write(d.join("zendo-test-agent"), "").unwrap();
    let env = BTreeMap::from([("PATH".to_string(), d.display().to_string())]);
    assert_eq!(find_program(&env, "zendo-test-agent"), Some(d.join("zendo-test-agent")));
    assert_eq!(find_program(&NO_ENV, "zendo-test-agent"), None);
    let _ = std::fs::remove_dir_all(&d);
}
