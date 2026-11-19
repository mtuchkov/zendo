//! `zendo doctor`, `zendo doctor --fix` and `zendo init` (docs/zendo/cli): doctor writes no file,
//! `--fix` repairs what needs no judgement without the agent, `--agent` gives the rest to the agent,
//! which may change only the structure file, and init prepares the repository.

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
    zendo(root, home, &[&["doctor", "--mock"], args].concat())
}

/// Run `zendo` with `args` in `root`, with `home` as the home folder. Returns the success and the output.
fn zendo(root: &Path, home: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_zendo"))
        .args(args)
        .current_dir(root)
        .env("HOME", home)
        .env("ZENDO_NO_UPDATE_CHECK", "1")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        // The zendo section of the user (SESSIONS-22) goes under `home`, never to the real folders.
        .env_remove("CLAUDE_CONFIG_DIR")
        .env_remove("CODEX_HOME")
        .output()
        .unwrap();
    (out.status.success(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

/// The files that zendo can write in the repository, and the instructions of the user for Claude Code.
fn zendo_files(root: &Path, home: &Path) -> [PathBuf; 4] {
    [root.join("project-structure.toml"), root.join("CLAUDE.md"), root.join(".gitignore"), home.join(".claude/CLAUDE.md")]
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
    let (ok, out) = doctor(&root, &home, &["--fix", "--agent"]);
    assert!(ok, "{out}");
    assert!(out.contains("refused") && out.contains("allowed"), "{out}");
    assert_eq!(std::fs::read_to_string(root.join("README.md")).unwrap(), "# readme\n");
    assert!(understood(&root));
    // A copy of the broken file is kept outside the repo.
    let backups = home.join(".local/share/zendo/backups");
    assert_eq!(std::fs::read_dir(backups).unwrap().count(), 1);
}

// STRUCT-6, STRUCT-8: without `--fix`, doctor names the command and changes nothing.
#[test]
fn without_fix_nothing_changes() {
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
    let (ok, out) = doctor(&root, &home, &["--fix", "--agent"]);
    assert!(ok, "{out}");
    assert!(out.contains("created"), "{out}");
    assert!(understood(&root));
}

// STRUCT-2, STRUCT-7, CMD-15: `--fix` without `--agent` does not call the agent; it names `--agent`
// for what needs judgement.
#[test]
fn fix_without_claude_does_not_call_the_agent() {
    let (root, home) = repo("no-claude");
    std::fs::write(root.join("project-structure.toml"), "[[entry]\npath = docs\n").unwrap();
    let (ok, out) = doctor(&root, &home, &["--fix"]);
    assert!(!ok, "{out}");
    assert!(out.contains("zendo doctor --fix --agent") && !out.contains("is fixing the file"), "{out}");
    assert_eq!(std::fs::read_to_string(root.join("project-structure.toml")).unwrap(), "[[entry]\npath = docs\n");
}

// CMD-14: `zendo doctor` without `--fix` creates and changes no file, and names `zendo doctor --fix`.
#[test]
fn doctor_writes_no_file() {
    let (root, home) = repo("read-only");
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    let (ok, out) = doctor(&root, &home, &[]);
    assert!(ok, "{out}");
    for f in zendo_files(&root, &home) {
        assert!(!f.exists(), "{} · {out}", f.display());
    }
    assert!(out.contains("zendo doctor --fix"), "{out}");
}

// CMD-15, STRUCT-16 to STRUCT-18, SESSIONS-22: `zendo doctor --fix` repairs the repository and the
// machine without the agent, reports each file, and a second run changes nothing.
#[test]
fn doctor_fix_repairs_the_repository_and_the_machine() {
    let (root, home) = repo("fix-all");
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    let (ok, out) = doctor(&root, &home, &["--fix"]);
    assert!(ok, "{out}");
    assert!(!out.contains("is fixing the file"), "{out}");
    let files = zendo_files(&root, &home);
    for f in &files {
        assert!(f.exists(), "{} · {out}", f.display());
    }
    assert!(out.contains("project-structure.toml") && out.contains("CLAUDE.md") && out.contains(".gitignore"), "{out}");
    assert!(std::fs::read_to_string(&files[3]).unwrap().contains("MCP server `zendo`"));
    let before: Vec<String> = files.iter().map(|f| std::fs::read_to_string(f).unwrap()).collect();
    let (ok, out) = doctor(&root, &home, &["--fix"]);
    assert!(ok, "{out}");
    let after: Vec<String> = files.iter().map(|f| std::fs::read_to_string(f).unwrap()).collect();
    assert_eq!(before, after, "a second run changes nothing");
}

// CMD-16, STRUCT-16 to STRUCT-18: `zendo init` prepares the repository, tells each change, and
// writes nothing on the machine.
#[test]
fn init_prepares_the_repository_only() {
    let (root, home) = repo("init");
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    let (ok, out) = zendo(&root, &home, &["init"]);
    assert!(ok, "{out}");
    let files = zendo_files(&root, &home);
    assert!(files[..3].iter().all(|f| f.exists()) && !files[3].exists(), "{out}");
    assert!(out.contains("project-structure.toml") && out.contains("CLAUDE.md") && out.contains(".gitignore"), "{out}");
    assert!(zendo::setup::doctor::structure_problem(&root).is_none());
}

// CMD-16, CMD-2: outside a repository, `zendo init` says so and fails, and writes nothing.
#[test]
fn init_outside_a_repository_fails() {
    let (root, home) = repo("init-outside");
    std::fs::remove_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(&home).unwrap();
    let (ok, out) = zendo(&root, &home, &["init"]);
    assert!(!ok, "{out}");
    assert!(out.contains("not a git repository"), "{out}");
    assert!(!root.join("project-structure.toml").exists() && !home.join("project-structure.toml").exists());
}
