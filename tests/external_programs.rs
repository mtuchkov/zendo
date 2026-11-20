//! zendo needs no utility (AGENTS.md, "Code"): it does its work in its own code, and it starts a
//! program only from a file of `KNOWN`. A file of `src` that starts a program is in `KNOWN`, with
//! the programs and what zendo needs them for. A new program needs the approval of the user.

use std::path::{Path, PathBuf};

/// The files of `src` that start a program, with the programs and what zendo needs them for.
const KNOWN: [(&str, &str); 23] = [
    ("src/main.rs", "zendo itself: the new version after an upgrade"),
    ("src/agent/acp.rs", "the agent of the chat: the command of `--agent`"),
    ("src/agent/api/cli.rs", "claude: the name model and its login check"),
    ("src/agent/work/mod.rs", "git: the changes of a session"),
    ("src/agent/work/recap.rs", "git: the changes for a recap"),
    ("src/app/version.rs", "git: a document as it is on main"),
    ("src/board/mod.rs", "git: the main checkout of a worktree"),
    ("src/board/tools_posts.rs", "git: the branch of a session"),
    ("src/music/output.rs", "paplay, aplay, pw-cat: the sound of the music on Linux"),
    ("src/session/pty.rs", "Claude Code, Codex and the shell of the user in a terminal of zendo"),
    ("src/setup/doctor/mod.rs", "the helper `run` of the checks; its callers name the programs"),
    ("src/setup/doctor/agent_checks.rs", "node, npm, claude: the checks of the agent"),
    ("src/setup/doctor/preflight.rs", "claude: the login check before the first message"),
    ("src/setup/doctor/setup.rs", "node, npm: the install of the adapter of the agent"),
    ("src/setup/doctor/upgrade.rs", "git, cargo and zendo itself: `zendo upgrade`"),
    ("src/setup/doctor/fonts.rs", "fc-cache, and Claude Code or Codex for a font recipe"),
    ("src/setup/fonts/recipe.rs", "the commands of a font recipe, without a shell (FONT-30)"),
    ("src/setup/fonts/remote.rs", "curl: the download of a font"),
    ("src/setup/sync/git.rs", "git: the sync of the notes repository"),
    ("src/setup/sync/mod.rs", "git, and scutil or hostname for the name of the machine"),
    ("src/text/clipboard.rs", "pbcopy, pbpaste, wl-copy, wl-paste, xclip, xsel, osascript: the clipboard; open, xdg-open: a link"),
    ("src/tui/sound.rs", "afplay: the sound of a notice on macOS"),
    ("src/bin/", "the mock programs of the tests"),
];

/// The calls that start a program: of the standard library and of tokio, of the terminal, of the
/// chat agent, and the helper `run` of `zendo doctor`.
const STARTS: [&str; 4] = ["Command::new(", "CommandBuilder::new(", "AcpAgent::from_str(", "run(\""];

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

/// The files of `src`, from the root of the crate, that call a start of `STARTS`.
fn starters() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    sources(&root.join("src"), &mut files);
    let mut out: Vec<String> = files
        .iter()
        .filter(|p| STARTS.iter().any(|s| std::fs::read_to_string(p).unwrap().contains(s)))
        .map(|p| p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"))
        .collect();
    out.sort();
    out
}

fn covers(known: &str, file: &str) -> bool {
    if known.ends_with('/') { file.starts_with(known) } else { file == known }
}

#[test]
fn zendo_starts_only_the_known_programs() {
    let starters = starters();
    let new: Vec<&String> = starters.iter().filter(|f| !KNOWN.iter().any(|(k, _)| covers(k, f))).collect();
    assert!(new.is_empty(), "these files start a program, and they are not in KNOWN. zendo needs no utility: do the work in the code of zendo. Add a file to KNOWN only when the user approves its program:\n{new:#?}");
    let stale: Vec<&str> = KNOWN.iter().map(|(k, _)| *k).filter(|k| !starters.iter().any(|f| covers(k, f))).collect();
    assert!(stale.is_empty(), "these entries of KNOWN start no program any more; remove them:\n{stale:#?}");
}
