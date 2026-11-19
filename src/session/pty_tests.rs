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
