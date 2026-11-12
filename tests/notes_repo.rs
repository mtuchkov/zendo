//! `zendo notes move` and `zendo notes consolidate` with the mock agent, a notes repo and a bare remote.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git").arg("-C").arg(dir).args(args).stderr(Stdio::null()).output().unwrap();
    assert!(out.status.success(), "git {args:?} in {}", dir.display());
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

struct World {
    base: PathBuf,
    home: PathBuf,
    root: PathBuf,
    notes: PathBuf,
}

/// A home with a project repo (origin github.com/o/proj, two notes in docs/notes), a notes repo with a
/// bare remote, and a zendo config that turns the notes repo on for machine "m1".
fn world(name: &str) -> World {
    let base = std::env::temp_dir().join(format!("zendo-notes-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let home = base.join("home");
    let root = home.join("src/proj");
    std::fs::create_dir_all(root.join("docs/notes")).unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["remote", "add", "origin", "git@github.com:o/proj.git"]);
    std::fs::write(root.join("docs/notes/2026-01-01-idea.md"), "# Idea\n\nFrom docs/notes.\n").unwrap();
    std::fs::write(root.join("docs/notes/2026-01-02-plan.md"), "# Plan\n").unwrap();
    git(&base, &["init", "-q", "--bare", "-b", "main", "remote.git"]);
    git(&home, &["clone", "-q", "../remote.git", "notes"]);
    let notes = home.join("notes");
    git(&notes, &["config", "user.email", "t@example.com"]);
    git(&notes, &["config", "user.name", "t"]);
    git(&notes, &["checkout", "-q", "-B", "main"]);
    std::fs::create_dir_all(home.join(".zendo")).unwrap();
    let cfg = format!("[notes]\npath = \"{}\"\nmachine = \"m1\"\nsync_seconds = 15\n", notes.display());
    std::fs::write(home.join(".zendo/config.toml"), cfg).unwrap();
    World { base, home, root, notes }
}

fn zendo(w: &World, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_zendo"))
        .args(args)
        .arg("--mock")
        .current_dir(&w.root)
        .env("HOME", &w.home)
        .env("ZENDO_NO_UPDATE_CHECK", "1")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .env_remove("XDG_CACHE_HOME")
        .env_remove("ZENDO_NOTES")
        .output()
        .unwrap();
    (out.status.success(), String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr))
}

#[test]
fn move_puts_the_notes_of_the_repo_into_the_folder_of_this_machine() {
    let w = world("move");
    let (ok, out) = zendo(&w, &["notes", "move"]);
    assert!(ok, "{out}");
    let dir = w.notes.join("m1/src/proj");
    assert!(out.contains("refused") && out.contains("outside.md"), "an edit outside the target is refused: {out}");
    assert!(!w.notes.join("m1/src/outside.md").exists());
    assert_eq!(std::fs::read_to_string(dir.join("2026-01-01-idea.md")).unwrap(), "# Idea\n\nFrom docs/notes.\n");
    assert!(dir.join("2026-01-02-plan.md").exists());
    assert!(zendo_location(&dir).contains("origin = \"github.com/o/proj\""), "the folder knows its origin");
    assert!(std::fs::read_dir(w.root.join("docs/notes")).unwrap().next().is_none(), "the moved notes are gone from the repo");
    let log = git(&w.base.join("remote.git"), &["log", "--format=%s", "main"]);
    assert!(log.contains("notes: move 2 notes"), "pushed: {log}");
    let _ = std::fs::remove_dir_all(&w.base);
}

#[test]
fn consolidate_merges_the_folders_of_one_origin() {
    let w = world("consolidate");
    // Machine m2 has the same project at another path, with a note of the same name.
    let other = w.notes.join("m2/work/proj");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join(".zendo-location.toml"), "machine = \"m2\"\npath = \"/home/x/work/proj\"\norigin = \"github.com/o/proj\"\n").unwrap();
    std::fs::write(other.join("2026-01-01-idea.md"), "# Idea\n\nFrom m2.\n").unwrap();
    // Another project on m2 must stay where it is.
    let unrelated = w.notes.join("m2/work/other");
    std::fs::create_dir_all(&unrelated).unwrap();
    std::fs::write(unrelated.join(".zendo-location.toml"), "machine = \"m2\"\npath = \"/home/x/work/other\"\norigin = \"github.com/o/other\"\n").unwrap();
    std::fs::write(unrelated.join("keep.md"), "# Keep\n").unwrap();
    // This machine already has the idea.
    let here = w.notes.join("m1/src/proj");
    std::fs::create_dir_all(&here).unwrap();
    std::fs::write(here.join("2026-01-01-idea.md"), "# Idea\n\nFrom m1.\n").unwrap();
    git(&w.notes, &["add", "-A"]);
    git(&w.notes, &["commit", "-q", "-m", "two machines"]);

    let (ok, out) = zendo(&w, &["notes", "consolidate"]);
    assert!(ok, "{out}");
    let merged = std::fs::read_to_string(here.join("2026-01-01-idea.md")).unwrap();
    assert!(merged.contains("From m1.") && merged.contains("From m2."), "{merged}");
    assert!(!other.exists(), "the empty folder of m2 is gone");
    assert!(unrelated.join("keep.md").exists(), "another origin stays");
    let log = git(&w.base.join("remote.git"), &["log", "--format=%s", "main"]);
    assert!(log.contains("notes: consolidate github.com/o/proj"), "pushed: {log}");
    let _ = std::fs::remove_dir_all(&w.base);
}

#[test]
fn without_a_notes_repo_the_commands_say_how_to_turn_it_on() {
    let w = world("off");
    std::fs::remove_file(w.home.join(".zendo/config.toml")).unwrap();
    let (ok, out) = zendo(&w, &["notes", "move"]);
    assert!(!ok && out.contains("zendo setup --notes-repo"), "{out}");
    let _ = std::fs::remove_dir_all(&w.base);
}

fn zendo_location(dir: &Path) -> String {
    std::fs::read_to_string(dir.join(".zendo-location.toml")).unwrap_or_default()
}
