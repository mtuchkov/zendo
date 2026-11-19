//! IO-1, IO-7: the modules that run on the UI thread call no file, program or system IO; they hand
//! it to `zendo::worker` (ADR 0024). The files in `ALLOWED` say why their IO is not on the UI thread.
//! The check sees only direct calls; the slow-tick log of debug builds finds the rest (IO-8).

use std::path::{Path, PathBuf};

/// The folders whose code runs on the UI thread.
const UI: [&str; 3] = ["src/app", "src/ui", "src/tui"];

/// The files of the UI folders whose IO runs before the first frame (IO-7) or on a worker.
const ALLOWED: [(&str, &str); 3] = [
    ("src/app/new.rs", "App::new seeds the worktrees of the sessions before the first frame (IO-7)"),
    ("src/app/facts.rs", "the functions of the memos run on their workers"),
    ("src/tui/sound.rs", "the player starts on a thread of ring()"),
];

/// The calls that do IO.
const IO: [&str; 16] = [
    "std::fs::",
    "fs::read",
    "fs::write",
    "fs::create_dir",
    "fs::remove",
    "fs::rename",
    "fs::metadata",
    "File::open",
    "File::create",
    "OpenOptions::new(",
    "Command::new(",
    "atomic::write",
    ".sync_all()",
    ".exists()",
    ".is_file()",
    ".is_dir()",
];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") && !p.to_string_lossy().ends_with("_tests.rs") {
            out.push(p);
        }
    }
}

/// Each line of the UI code that calls IO: `file:line: call`, outside comments.
fn io_lines() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    UI.iter().for_each(|d| sources(&root.join(d), &mut files));
    files.sort();
    let rel = |p: &Path| p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
    files
        .iter()
        .filter(|p| !ALLOWED.iter().any(|(f, _)| rel(p) == *f))
        .flat_map(|p| {
            let text = std::fs::read_to_string(p).unwrap();
            let file = rel(p);
            text.lines()
                .enumerate()
                .filter(|(_, l)| !l.trim_start().starts_with("//"))
                .filter_map(|(i, l)| IO.iter().find(|c| l.contains(*c)).map(|c| format!("{file}:{}: {c}", i + 1)))
                .collect::<Vec<_>>()
        })
        .collect()
}

// IO-1, IO-7: no IO in the code of the UI thread, except the reads at the start.
#[test]
#[ignore = "IO-1: on in the last commit of the branch docs/io-spec, when every group of IO is on zendo::worker"]
fn ui_code_does_no_io() {
    let found = io_lines();
    assert!(found.is_empty(), "the UI thread calls IO here; hand it to zendo::worker (IO-1, ADR 0024):\n{}", found.join("\n"));
}

// IO-7: each entry of ALLOWED still does IO; a stale entry goes.
#[test]
fn each_allowed_entry_still_does_io() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (file, why) in ALLOWED {
        let text = std::fs::read_to_string(root.join(file)).unwrap();
        assert!(IO.iter().any(|c| text.contains(c)), "{file} does no IO any more ({why}); remove it from ALLOWED");
    }
}
