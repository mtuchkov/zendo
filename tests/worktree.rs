//! The worktree views of the file tree and the recap of the worktree (FILES-26 to FILES-37), in a
//! real git repository with a worktree for agent session 1.

mod common;
use common::{app, draw, key, until};
use ratatui::crossterm::event::KeyCode;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use zendo::app::{App, Focus};
use zendo::docs::tree::View;

/// Run git in `dir`, as a test user.
fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git").args(["-c", "user.name=t", "-c", "user.email=t@t"]).args(args).current_dir(dir).output().unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

fn write(p: PathBuf, text: &str) {
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

/// A repository on `main` with `a.md`, `b.md` and `c.md`, and the worktree `.worktrees/1` on its own
/// branch. The folder of the test is its own, and a run that stopped before leaves nothing behind.
fn repo(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-worktree-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    git(&d, &["init", "-q", "-b", "main"]);
    for f in ["a", "b", "c"] {
        write(d.join(format!("{f}.md")), &format!("# {f}\n"));
    }
    write(d.join(".gitignore"), "/.worktrees/\n/.zendo/\n");
    git(&d, &["add", "."]);
    git(&d, &["commit", "-q", "-m", "start"]);
    git(&d, &["worktree", "add", "-q", ".worktrees/1", "-b", "agent"]);
    d
}

/// The tree open in `view`.
fn tree_in(root: &Path, view: View) -> App {
    let mut a = app(root);
    a.toggle_tree();
    while a.files.tree().view != view {
        key(&mut a, KeyCode::Char('v'));
    }
    a
}

fn files(a: &App) -> Vec<String> {
    a.files.tree().files.iter().map(|p| p.display().to_string()).collect()
}

// FILES-26
#[test]
fn the_worktree_view_lists_the_files_of_the_session_worktree() {
    let root = repo("view");
    write(root.join(".worktrees/1/src/new.rs"), "fn main() {}\n");
    write(root.join("main-only.md"), "# only in the main checkout\n");
    let a = tree_in(&root, View::Worktree);
    let files = files(&a);
    assert!(files.contains(&".worktrees/1/a.md".to_string()), "{files:?}");
    assert!(files.contains(&".worktrees/1/src/new.rs".to_string()), "{files:?}");
    assert!(files.iter().all(|f| f.starts_with(".worktrees/1/")), "no file of the main checkout: {files:?}");
    let rows: Vec<String> = a.files.tree().rows().iter().map(|r| r.name.clone()).collect();
    assert!(rows.contains(&"a.md".to_string()), "the rows start in the worktree: {rows:?}");
}

// FILES-27, FILES-33, FILES-35
#[test]
fn the_changed_view_lists_what_differs_from_main() {
    let root = repo("changed");
    let wt = root.join(".worktrees/1");
    write(wt.join("a.md"), "# a\n\nchanged\n");
    git(&wt, &["commit", "-q", "-am", "change a"]);
    std::fs::remove_file(wt.join("b.md")).unwrap();
    write(wt.join("n.txt"), "new\n");
    let mut a = tree_in(&root, View::WorktreeChanged);
    assert_eq!(files(&a), [".worktrees/1/a.md", ".worktrees/1/b.md", ".worktrees/1/n.txt"]);
    assert!(a.files.tree().deleted.contains(Path::new(".worktrees/1/b.md")), "b.md is deleted");

    // A deleted file does not open.
    a.focus = Focus::Tree;
    a.files.reveal(Path::new(".worktrees/1/b.md"));
    key(&mut a, KeyCode::Enter);
    assert!(a.files.current().is_none_or(|p| !p.ends_with("b.md")), "{:?}", a.files.current());

    // A new change shows without a restart.
    write(wt.join("c.md"), "# c\n\nchanged too\n");
    a.files.scan_again();
    a.tick();
    assert!(files(&a).contains(&".worktrees/1/c.md".to_string()), "{:?}", files(&a));
}

// FILES-31, FILES-32. Async: the chat of the other session starts the agent.
#[tokio::test]
async fn the_worktree_views_follow_the_agent_session() {
    let root = repo("session");
    let mut a = tree_in(&root, View::Worktree);
    assert_eq!(a.files.tree().worktree.as_deref(), Some(Path::new(".worktrees/1")));
    a.switch_session(1);
    assert_eq!(a.files.tree().worktree, None, "agent session 2 has no worktree");
    assert!(a.files.tree().files.is_empty(), "{:?}", a.files.tree().files);
    let screen = draw(&mut a, 120, 30).join("\n");
    assert!(screen.contains("has no") && screen.contains("worktree yet"), "{screen}");
    a.files.set_view(View::Folders);
    key(&mut a, KeyCode::Char('v'));
    assert!(!a.files.tree().view.is_worktree(), "v skips the worktree views: {:?}", a.files.tree().view);
    a.switch_session(0);
    assert_eq!(a.files.tree().worktree.as_deref(), Some(Path::new(".worktrees/1")));
}

// FILES-34
#[test]
fn a_file_of_the_worktree_opens_with_the_worktree_in_its_head() {
    let root = repo("open");
    let mut a = tree_in(&root, View::Worktree);
    a.focus = Focus::Tree;
    a.files.reveal(Path::new(".worktrees/1/a.md"));
    key(&mut a, KeyCode::Enter);
    assert_eq!(a.files.current_path(), Some(root.join(".worktrees/1/a.md").as_path()));
    let screen = draw(&mut a, 120, 30);
    assert!(screen[0].contains("worktree 1 · a.md"), "{}", screen[0]);
    // The delta compares with the HEAD of the worktree, not of the main checkout.
    assert_eq!(a.git_head(".worktrees/1/a.md").as_deref(), Some("# a\n"));
}

// FILES-37
#[test]
fn file_actions_in_a_worktree_ask_first_while_the_agent_works() {
    let root = repo("confirm");
    let mut a = tree_in(&root, View::Worktree);
    a.focus = Focus::Tree;
    a.files.reveal(Path::new(".worktrees/1/a.md"));

    // The agent is idle: the usual question.
    key(&mut a, KeyCode::Delete);
    assert!(a.files.prompt().is_some());
    key(&mut a, KeyCode::Esc);
    assert!(a.files.prompt().is_none());

    // The agent of the session answers: a popup asks first; a key other than y cancels.
    a.threads.entry("(chat)").busy = Some(Instant::now());
    key(&mut a, KeyCode::Delete);
    assert!(a.files.prompt().is_none() && a.files.worktree().confirm.is_some());
    let screen = draw(&mut a, 120, 30).join("\n");
    assert!(screen.contains("works in this worktree"), "{screen}");
    key(&mut a, KeyCode::Char('n'));
    assert!(a.files.worktree().confirm.is_none() && a.files.prompt().is_none());
    key(&mut a, KeyCode::F(2));
    key(&mut a, KeyCode::Char('y'));
    assert!(matches!(a.files.prompt(), Some(zendo::app::FilePrompt::Rename(..))), "after y, the question of the rename");
}

// FILES-28, FILES-29, FILES-36
#[tokio::test]
async fn the_worktree_recap_refreshes_every_10_minutes_when_it_changed() {
    let root = repo("recap");
    let wt = root.join(".worktrees/1");
    write(wt.join("a.md"), "# a\n\nchanged\n");
    let mut a = tree_in(&root, View::WorktreeChanged);
    a.ensure_agent();
    until(&mut a, |a| a.link.ready()).await;
    let recap = |a: &App| a.state.files.get(".worktrees/1").and_then(|f| f.recap.as_ref()).map(|r| (r.text.clone(), r.of_hash.clone()));

    // The first recap.
    until(&mut a, |a| recap(a).is_some()).await;
    let (text, first) = recap(&a).unwrap();
    assert!(text.contains("1 file"), "{text}");
    let screen = draw(&mut a, 120, 30).join("\n");
    assert!(screen.contains("RECAP ·") && screen.contains("The worktree changes"), "the recap is at the bottom of the tree: {screen}");

    // A new change: no recap before 10 minutes, a new one after.
    write(wt.join("n.txt"), "new\n");
    a.files.worktree_mut().recap_check = Some(Instant::now() - Duration::from_secs(5 * 60));
    a.tick();
    assert!(a.work.free(), "no recap at 5 minutes");
    a.files.worktree_mut().recap_check = Some(Instant::now() - Duration::from_secs(10 * 60));
    until(&mut a, |a| recap(a).is_some_and(|(_, h)| h != first)).await;
    assert!(recap(&a).unwrap().0.contains("2 files"), "{:?}", recap(&a));

    // No change: no recap.
    a.files.worktree_mut().recap_check = Some(Instant::now() - Duration::from_secs(10 * 60));
    a.tick();
    assert!(a.work.free(), "no recap without a change");
    assert!(a.threads.values().all(|t| t.items.is_empty()), "the chat has no recap");
}

// WORK-12, FILES-34: the recap of a file of a worktree without changes tells the last commit of the
// file in the branch of the worktree.
#[test]
fn the_last_commit_of_a_worktree_file_comes_from_the_worktree() {
    let root = repo("commit");
    let wt = root.join(".worktrees/1");
    write(wt.join("a.md"), "# a\n\nchanged\n");
    git(&wt, &["commit", "-q", "-am", "agent changes a"]);
    let a = app(&root);
    let commit = a.last_commit(".worktrees/1/a.md").unwrap_or_default();
    assert!(commit.contains("agent changes a"), "{commit:?}");
    assert!(a.last_commit("a.md").is_some_and(|c| c.contains("start")), "a file of the main checkout as before");
}

// ----- The document of the session (docs/zendo/files/files-worktree-note.md) ------------------

/// `repo(name)`, where `a.md` says `body main` on `main`, `body one` in `.worktrees/1` and `body two`
/// in the worktree `.worktrees/2`. Agent session 3 has no worktree.
fn two_worktrees(name: &str) -> PathBuf {
    let root = repo(name);
    write(root.join("a.md"), "# a\n\nbody main\n");
    git(&root, &["commit", "-q", "-am", "main body"]);
    git(&root, &["worktree", "add", "-q", ".worktrees/2", "-b", "agent2"]);
    write(root.join(".worktrees/1/a.md"), "# a\n\nbody one\n");
    write(root.join(".worktrees/2/a.md"), "# a\n\nbody two\n");
    root
}

fn screen(a: &mut App) -> String {
    draw(a, 120, 30).join("\n")
}

/// The file that the last question of agent session `n` attached, from the root.
fn attached(a: &App, n: usize) -> Option<String> {
    a.threads.get(&zendo::app::session_key(n))?.files.last().cloned().flatten()
}

/// Ask in the chat of the active session and wait until the agent answers.
async fn ask(a: &mut App) {
    a.focus = Focus::Claude;
    for c in "what is it".chars() {
        key(a, KeyCode::Char(c));
    }
    key(a, KeyCode::Enter);
    let k = a.key();
    until(a, |a| a.threads.get(&k).is_some_and(|t| t.busy.is_none() && t.items.iter().any(|i| matches!(i, zendo::app::Item::Agent(_))))).await;
}

// FILES-38
#[tokio::test(flavor = "multi_thread")]
async fn a_question_names_the_document_of_its_session() {
    let root = two_worktrees("ask");
    let mut a = app(&root);
    a.switch_session(0);
    a.open_file(root.join(".worktrees/1/a.md"));
    a.switch_session(1);
    ask(&mut a).await;
    assert_eq!(attached(&a, 1).as_deref(), Some(".worktrees/2/a.md"));
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-39, FILES-42
#[tokio::test]
async fn choosing_a_session_shows_its_version_of_the_document() {
    let root = two_worktrees("choose");
    let mut a = app(&root);
    a.switch_session(0);
    a.open_file(root.join("a.md"));
    assert_eq!(a.files.current_path(), Some(root.join(".worktrees/1/a.md").as_path()), "a file opens in the version of the session");
    a.focus = Focus::Note;
    assert!(screen(&mut a).contains("body one"));
    a.switch_session(1);
    assert!(screen(&mut a).contains("body two"));
    a.switch_session(2);
    assert!(screen(&mut a).contains("body main"), "no worktree: the main checkout");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-40, FILES-41, FILES-44
#[tokio::test(flavor = "multi_thread")]
async fn a_missing_document_says_so_and_comes_back() {
    let root = two_worktrees("missing");
    write(root.join(".worktrees/1/new.md"), "# new\n\nonly in one\n");
    let mut a = app(&root);
    a.switch_session(0);
    a.open_file(root.join(".worktrees/1/new.md"));
    a.switch_session(1);
    a.focus = Focus::Note;
    let s = screen(&mut a);
    assert!(s.contains("This document does not exist in this session.") && !s.contains("only in one"), "{s}");
    ask(&mut a).await;
    assert_eq!(attached(&a, 1), None, "a question without a note file");
    a.switch_session(0);
    assert!(screen(&mut a).contains("only in one"), "back in its session");
    a.switch_session(1);
    write(root.join(".worktrees/2/new.md"), "# new\n\nnow in two\n");
    a.files.look_again();
    a.tick();
    assert!(screen(&mut a).contains("now in two"), "it shows as soon as it exists");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-43
#[tokio::test]
async fn choosing_a_terminal_keeps_the_document() {
    let root = two_worktrees("terminal");
    let mut a = app(&root);
    a.cli.shell = Some("/bin/sh".into());
    a.switch_session(1);
    a.open_file(root.join("a.md"));
    a.focus = Focus::Note;
    key(&mut a, KeyCode::F(5));
    assert!(a.cli.pane.is_some(), "the terminal is in the chat pane");
    assert_eq!(a.files.current_path(), Some(root.join(".worktrees/2/a.md").as_path()));
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-45, FILES-46, FILES-47, FILES-48
#[tokio::test]
async fn holding_m_and_u_shows_main_and_the_disk_until_the_key_goes_up() {
    use ratatui::crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
    let root = two_worktrees("peek");
    write(root.join("a.md"), "# a\n\nbody on disk\n");
    write(root.join(".worktrees/1/new.md"), "# new\n\nonly in one\n");
    let mut a = app(&root);
    a.switch_session(0);
    a.open_file(root.join("a.md"));
    a.focus = Focus::Note;
    key(&mut a, KeyCode::Char('m'));
    let s = screen(&mut a);
    assert!(s.contains("body main") && s.contains("main · a.md"), "m: the last commit of main\n{s}");
    a.on_key(KeyEvent::new_with_kind(KeyCode::Char('m'), KeyModifiers::NONE, KeyEventKind::Release));
    assert!(screen(&mut a).contains("body one"), "the key went up: the version of the session");
    key(&mut a, KeyCode::Char('u'));
    let s = screen(&mut a);
    assert!(s.contains("body on disk") && s.contains("disk · a.md"), "u: the main checkout on disk\n{s}");
    a.files.peek_mut().last = Instant::now() - Duration::from_secs(5);
    a.tick();
    assert!(screen(&mut a).contains("body one"), "the auto-repeat stopped: the key is up");
    a.open_file(root.join(".worktrees/1/new.md"));
    key(&mut a, KeyCode::Char('m'));
    assert!(screen(&mut a).contains("This document does not exist in main."));
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-49
#[tokio::test]
async fn a_new_note_is_saved_in_the_checkout_of_the_session() {
    let root = two_worktrees("new-note");
    let mut a = app(&root);
    a.switch_session(0);
    a.start_write(true);
    for c in "idea\n\nwritten in one".chars() {
        key(&mut a, if c == '\n' { KeyCode::Enter } else { KeyCode::Char(c) });
    }
    a.save();
    let saved = a.files.current().cloned().unwrap();
    assert!(saved.starts_with(root.join(".worktrees/1/docs/notes")) && saved.exists(), "{}", saved.display());
    assert!(!root.join("docs/notes").exists(), "nothing in the main checkout");
    a.mode = zendo::app::Mode::Read;
    assert!(screen(&mut a).contains("written in one"));
    a.switch_session(2);
    a.start_write(true);
    for c in "other".chars() {
        key(&mut a, KeyCode::Char(c));
    }
    a.save();
    assert!(a.files.current().cloned().unwrap().starts_with(root.join("docs/notes")), "no worktree: the main checkout");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-49, NOTESREP-1: with the notes in a notes repo outside the repository, a new note goes
// there, also while the agent session has a worktree.
#[tokio::test]
async fn a_new_note_of_a_notes_repo_stays_in_the_notes_repo() {
    let root = two_worktrees("notes-repo");
    let notes = std::env::temp_dir().join(format!("zendo-worktree-notes-repo-dir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&notes);
    std::fs::create_dir_all(&notes).unwrap();
    let mut a = App::new(zendo::app::Config {
        root: root.clone(),
        notes_dir: notes.clone(),
        agent: env!("CARGO_BIN_EXE_zendo-mock-agent").to_string(),
        agent_env: Default::default(),
        offline: false,
        notes_repo: None,
    })
    .unwrap();
    a.switch_session(0);
    a.start_write(true);
    for c in "kept".chars() {
        key(&mut a, KeyCode::Char(c));
    }
    a.save();
    let saved = a.files.current().cloned().unwrap();
    assert!(saved.starts_with(&notes) && saved.exists(), "{}", saved.display());
    assert!(!root.join(".worktrees/1/docs").exists(), "nothing in the worktree");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&notes);
}
