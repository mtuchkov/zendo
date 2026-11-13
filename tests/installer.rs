//! The installer, with stubs for `uname`, `git`, `rustc` and `cargo`: no network and no build.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

fn stub(dir: &Path, name: &str, body: &str) {
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// Run install.sh on a system that `uname -s` names, in a terminal that `TERM_PROGRAM` names.
fn install(system: &str, term: &str) -> String {
    let tmp = std::env::temp_dir().join(format!("zendo-installer-{system}-{term}-{}", std::process::id()));
    let (stubs, cargo_home) = (tmp.join("stubs"), tmp.join("cargo"));
    std::fs::create_dir_all(&stubs).unwrap();
    std::fs::create_dir_all(cargo_home.join("bin")).unwrap();
    stub(&stubs, "uname", &format!("echo {system}"));
    stub(&stubs, "git", "exit 1");
    stub(&stubs, "rustc", "echo rustc 1.99.0");
    let zendo = cargo_home.join("bin/zendo");
    stub(&stubs, "cargo", &format!("[ \"$1\" = install ] && [ \"$2\" != --list ] && printf '#!/bin/sh\\necho zendo 9.9.9\\n' > {0} && chmod +x {0}; exit 0", zendo.display()));
    let out = Command::new("sh")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("install.sh"))
        .env("PATH", format!("{}:{}:/usr/bin:/bin", stubs.display(), cargo_home.join("bin").display()))
        .env("CARGO_HOME", &cargo_home)
        .env("HOME", &tmp)
        .env("TERM_PROGRAM", term)
        .env("ZENDO_NO_SETUP", "1")
        .env("ZENDO_NO_RC", "1")
        .output()
        .unwrap();
    std::fs::remove_dir_all(&tmp).ok();
    let text = String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{text}");
    text
}

// CMD-14: on macOS the installer says that zendo uses the macOS keys, and how the terminal of the user
// makes Option a Meta key.
#[test]
fn the_installer_names_the_keys_of_macos_and_the_terminal_setting() {
    let terminal = install("Darwin", "Apple_Terminal");
    assert!(terminal.contains("macOS keys"), "{terminal}");
    assert!(terminal.contains("Use Option as Meta key"), "{terminal}");
    let iterm = install("Darwin", "iTerm.app");
    assert!(iterm.contains("Esc+"), "{iterm}");
    let other = install("Darwin", "SomeTerm");
    assert!(other.contains("README"), "{other}");
}

// CMD-14: on the other systems the installer says nothing of the macOS keys.
#[test]
fn the_installer_on_linux_keeps_the_default_keys() {
    let text = install("Linux", "xterm");
    assert!(text.contains("installed zendo 9.9.9"), "{text}");
    assert!(!text.contains("macOS"), "{text}");
}
