//! FAIL-2: zendo writes user data only through `zendo::atomic`, so a write is complete or does not
//! happen. A file of `src` may write a file in another way only when it is in `OWN`: it writes a
//! file of zendo itself, not user data.

use std::path::{Path, PathBuf};

/// The files of `src` that write files of zendo itself, with what they write.
const OWN: [(&str, &str); 13] = [
    ("src/atomic.rs", "the atomic write itself"),
    ("src/stats.rs", "the usage counts and the times of the actions, with a temporary file and a rename"),
    ("src/app/background.rs", "the log of the background mode"),
    ("src/agent/acp.rs", "the log of the agent"),
    ("src/session/pty.rs", "the hook settings and the MCP config of a session"),
    ("src/setup/state.rs", "the state in .zendo and its .gitignore"),
    ("src/setup/logs.rs", "the index of the logs"),
    ("src/setup/doctor/upgrade.rs", "the cache of the update check"),
    ("src/setup/fonts/mod.rs", "the list of the recommended fonts that zendo named"),
    ("src/setup/fonts/install.rs", "the font files, with a temporary file and a rename"),
    ("src/text/clipboard.rs", "a pasted image in the data folder"),
    ("src/board/journal.rs", "the journal of the message board: appends under a lock, compaction through zendo::atomic (ADR 0021)"),
    ("src/bin/", "the mock programs of the tests"),
];

/// The calls that write a file.
const WRITES: [&str; 3] = ["fs::write(", "File::create(", "OpenOptions::new("];

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

/// The files of `src`, from the root of the crate, that call a write of `WRITES`.
fn writers() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    sources(&root.join("src"), &mut files);
    let mut out: Vec<String> = files
        .iter()
        .filter(|p| WRITES.iter().any(|w| std::fs::read_to_string(p).unwrap().contains(w)))
        .map(|p| p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"))
        .collect();
    out.sort();
    out
}

fn own(file: &str) -> bool {
    OWN.iter().any(|(p, _)| if p.ends_with('/') { file.starts_with(p) } else { file == *p })
}

#[test]
fn user_data_goes_through_the_atomic_write() {
    let writers = writers();
    let direct: Vec<&String> = writers.iter().filter(|f| !own(f)).collect();
    assert!(direct.is_empty(), "these files write a file without zendo::atomic (FAIL-2); use zendo::atomic::write, or add the file to OWN when it writes a file of zendo itself:\n{direct:#?}");
    let stale: Vec<&str> = OWN.iter().map(|(p, _)| *p).filter(|p| !writers.iter().any(|f| if p.ends_with('/') { f.starts_with(p) } else { f == p })).collect();
    assert!(stale.is_empty(), "these entries of OWN write no file any more; remove them:\n{stale:#?}");
}
