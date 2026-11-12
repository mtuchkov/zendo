//! `zendo doctor --fix`: zendo fixes what needs no judgement; the agent may change only the structure file.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo(name: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir().join(format!("zendo-fix-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let root = base.join("repo");
    std::fs::create_dir_all(root.join("docs")).unwrap();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::write(root.join("README.md"), "# readme\n").unwrap();
    (root, base.join("home"))
}

fn doctor(root: &Path, home: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_zendo"))
        .args(["doctor", "--mock"])
        .args(args)
        .current_dir(root)
        .env("HOME", home)
        .env("ZENDO_NO_UPDATE_CHECK", "1")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .output()
        .unwrap();
    (out.status.success(), String::from_utf8_lossy(&out.stdout).into_owned())
}

fn understood(root: &Path) -> bool {
    zendo::setup::doctor::structure_problem(root).is_none()
}

#[test]
fn zendo_fixes_key_names_and_paths_by_itself() {
    let (root, home) = repo("quick");
    std::fs::write(root.join("project-structure.toml"), "# mine\n[[entry]]\npath = \"./docs\"\ndescription = \"Docs.\" # keep\nmain = \"yes\"\n").unwrap();
    let (ok, out) = doctor(&root, &home, &["--fix"]);
    assert!(ok, "{out}");
    assert!(out.contains("zendo fixed") && !out.contains("is fixing the file"), "{out}");
    let text = std::fs::read_to_string(root.join("project-structure.toml")).unwrap();
    assert!(text.contains("# mine") && text.contains("# keep") && text.contains("about = \"Docs.\""), "{text}");
    assert!(understood(&root));
}

#[test]
fn claude_fixes_the_rest_and_may_change_only_that_file() {
    let (root, home) = repo("claude");
    std::fs::write(root.join("project-structure.toml"), "[[entry]\npath = docs\n").unwrap();
    let (ok, out) = doctor(&root, &home, &["--fix"]);
    assert!(ok, "{out}");
    assert!(out.contains("refused") && out.contains("allowed"), "{out}");
    assert_eq!(std::fs::read_to_string(root.join("README.md")).unwrap(), "# readme\n");
    assert!(understood(&root));
    // A copy of the broken file is kept outside the repo.
    let backups = home.join(".local/share/zendo/backups");
    assert_eq!(std::fs::read_dir(backups).unwrap().count(), 1);
}

#[test]
fn without_fix_and_without_a_terminal_nothing_changes() {
    let (root, home) = repo("hint");
    std::fs::write(root.join("project-structure.toml"), "[[entry]\n").unwrap();
    let (ok, out) = doctor(&root, &home, &[]);
    assert!(!ok);
    assert!(out.contains("zendo doctor --fix"), "{out}");
    assert_eq!(std::fs::read_to_string(root.join("project-structure.toml")).unwrap(), "[[entry]\n");
}

#[test]
fn a_missing_file_is_created_from_project_structure_md() {
    let (root, home) = repo("create");
    std::fs::write(root.join("project-structure.md"), "# Project directory structure\n\n- docs/: the docs\n").unwrap();
    let (ok, out) = doctor(&root, &home, &["--fix"]);
    assert!(ok, "{out}");
    assert!(out.contains("created"), "{out}");
    assert!(understood(&root));
}
