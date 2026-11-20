use super::*;

// TERM-76: the terminal of zendo is not a VTE terminal: the shell does not get `VTE_VERSION`, so
// `/etc/profile.d/vte.sh` adds no marks of its own to the shell.
#[test]
fn a_shell_gets_no_vte_version() {
    let mut b = CommandBuilder::new("sh");
    b.env("VTE_VERSION", "7806");
    for_terminal(&mut b, Path::new("/"), None);
    assert_eq!(b.get_env("VTE_VERSION"), None);
    assert_eq!(b.get_env("TERM").and_then(|t| t.to_str()), Some("xterm-256color"));
}

// ENV-1, TERM-55: the shell of a terminal gets the variables of the agents, but not the ones that
// select its startup files: those stay as zendo has them.
#[test]
fn a_shell_gets_the_agent_env_without_the_startup_variables() {
    let env = [("CLAUDE_CONFIG_DIR", "/work"), ("ZDOTDIR", "/z"), ("ENV", "/e"), ("BASH_ENV", "/b")];
    let env = env.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    let mut b = CommandBuilder::new("sh");
    let before = ["ZDOTDIR", "ENV", "BASH_ENV"].map(|k| b.get_env(k).map(std::ffi::OsStr::to_os_string));
    agent_env_for_shell(&mut b, &env);
    assert_eq!(b.get_env("CLAUDE_CONFIG_DIR").and_then(|v| v.to_str()), Some("/work"));
    assert_eq!(["ZDOTDIR", "ENV", "BASH_ENV"].map(|k| b.get_env(k).map(std::ffi::OsStr::to_os_string)), before);
}
