//! zendo needs no utility (AGENTS.md, "Code"): it does its work in its own code, and it starts a
//! program only from a file of `KNOWN`. A file of `src` that starts a program is in `KNOWN`, with
//! the programs that it names and what zendo needs them for. A new program needs the approval of
//! the user.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The files of `src` that start a program: the programs that the file names as a text, and what
/// zendo needs them for. A program without a name in the file, as the command of the agent, is in
/// the reason only.
const KNOWN: [(&str, &[&str], &str); 23] = [
    ("src/main.rs", &[], "zendo itself: the new version after an upgrade"),
    ("src/agent/acp.rs", &[], "the agent of the chat: the command of `--agent-cmd`"),
    ("src/agent/api/cli.rs", &[], "claude, by its path: the name model and its login check"),
    ("src/agent/work/mod.rs", &["git"], "the changes of a session"),
    ("src/agent/work/recap.rs", &["git"], "the changes for a recap"),
    ("src/app/version.rs", &["git"], "a document as it is on main"),
    ("src/board/mod.rs", &["git"], "the main checkout of a worktree, for the board"),
    ("src/board/tools_posts.rs", &["git"], "the branch of a session"),
    ("src/music/output.rs", &["paplay", "aplay", "pw-cat"], "the sound of the music on Linux"),
    ("src/session/pty.rs", &[], "Claude Code, Codex and the shell of the user in a terminal of zendo"),
    ("src/setup/doctor/mod.rs", &[], "the helper `run` of the checks; its callers name the programs"),
    ("src/setup/doctor/agent_checks.rs", &["node", "npm", "claude"], "the checks of the agent"),
    ("src/setup/doctor/preflight.rs", &["claude"], "the login check before the first message"),
    ("src/setup/doctor/setup.rs", &["node", "npm"], "the install of the adapter of the agent"),
    ("src/setup/doctor/upgrade.rs", &["git", "cargo"], "`zendo upgrade`, and zendo itself after it"),
    ("src/setup/doctor/fonts.rs", &["fc-cache"], "the font cache, and Claude Code or Codex for a font recipe"),
    ("src/setup/fonts/recipe.rs", &[], "the commands of a font recipe, without a shell (FONT-30)"),
    ("src/setup/fonts/remote.rs", &["curl"], "the download of a font"),
    ("src/setup/sync/git.rs", &["git"], "the sync of the notes repository"),
    ("src/setup/sync/mod.rs", &["git", "scutil", "hostname"], "the origin of a repository, and the name of the machine"),
    ("src/text/clipboard.rs", &["pbcopy", "pbpaste", "wl-copy", "wl-paste", "xclip", "xsel", "osascript"], "the clipboard; `open` and `xdg-open`, by a variable, for a link"),
    ("src/tui/sound.rs", &[], "afplay, by a variable: the sound of a notice on macOS"),
    ("src/bin/", &[], "the mock programs of the tests"),
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

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// The files of `src`, from the root of the crate, that call a start of `STARTS`.
fn starters() -> Vec<String> {
    let mut files = Vec::new();
    sources(&root().join("src"), &mut files);
    let mut out: Vec<String> = files
        .iter()
        .filter(|p| STARTS.iter().any(|s| std::fs::read_to_string(p).unwrap().contains(s)))
        .map(|p| p.strip_prefix(root()).unwrap().to_string_lossy().replace('\\', "/"))
        .collect();
    out.sort();
    out
}

fn covers(known: &str, file: &str) -> bool {
    if known.ends_with('/') { file.starts_with(known) } else { file == known }
}

/// The programs that `text` names as a text where it starts one: `Command::new("git")`,
/// `run("npm", …)`, and a row of a table of programs, `("xclip", &[…])`.
fn named(text: &str) -> BTreeSet<String> {
    let name = |s: &str| s.split_once('"').filter(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || "-_./".contains(c))).map(|(n, after)| (n.to_string(), after.to_string()));
    let starts = |before: &str, after: &str| before.ends_with("Command::new") || before.ends_with("run") || after.starts_with(", &[");
    let mut out = BTreeSet::new();
    let mut before = "";
    for part in text.split("(\"") {
        if let Some((n, _)) = name(part).filter(|(_, after)| starts(before, after)) {
            out.insert(n);
        }
        before = part;
    }
    out
}

#[test]
fn zendo_starts_only_the_known_programs() {
    let starters = starters();
    let new: Vec<&String> = starters.iter().filter(|f| !KNOWN.iter().any(|(k, _, _)| covers(k, f))).collect();
    assert!(new.is_empty(), "these files start a program, and they are not in KNOWN. zendo needs no utility: do the work in the code of zendo. Add a file to KNOWN only when the user approves its program:\n{new:#?}");
    let stale: Vec<&str> = KNOWN.iter().map(|(k, _, _)| *k).filter(|k| !starters.iter().any(|f| covers(k, f))).collect();
    assert!(stale.is_empty(), "these entries of KNOWN start no program any more; remove them:\n{stale:#?}");
}

// A new program in a file that starts programs already: the names in the file and in `KNOWN` are
// the same.
#[test]
fn a_known_file_names_only_its_known_programs() {
    let differs = |(file, programs, _): &(&str, &[&str], &str)| {
        let found = named(&std::fs::read_to_string(root().join(file)).unwrap());
        let known: BTreeSet<String> = programs.iter().map(|p| p.to_string()).collect();
        (found != known).then(|| format!("{file}: the file names {found:?}, KNOWN has {known:?}"))
    };
    let wrong: Vec<String> = KNOWN.iter().filter(|(f, _, _)| !f.ends_with('/')).filter_map(differs).collect();
    assert!(wrong.is_empty(), "the programs of these files and of KNOWN differ. zendo needs no utility: add a program to KNOWN only when the user approves it:\n{wrong:#?}");
}

// The check sees a program in each of the three forms, and no other text.
#[test]
fn the_check_finds_the_names_of_programs() {
    let text = r#"Command::new("git").arg("status"); run("npm", &["ping"], env); [("xclip", &["-o"]), ("pw-cat", &[])]; o.ok("node", v); format!("{x}"); Command::new("/bin/sh")"#;
    assert_eq!(named(text).into_iter().collect::<Vec<_>>(), ["/bin/sh", "git", "npm", "pw-cat", "xclip"]);
}
