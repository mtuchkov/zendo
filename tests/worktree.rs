//! The worktree views of the file tree and the recap of the worktree (FILES-26 to FILES-37), in a
//! real git repository with a worktree for agent session 1; agent session 0 works in the main
//! checkout (SESSIONS-23).

mod common;
use common::{app, draw, draw_now, key, until};
use ratatui::crossterm::event::KeyCode;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use zendo::app::{App, Focus};
use zendo::docs::tree::View;

/// Run git in `dir`, as a test user.
fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git").args(["-c", "user.name=t", "-c", "user.email=t@t"]).args(args).current_dir(dir).output().unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).into_owned()
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
    // SESSIONS-23: agent session 1 has the worktree `.worktrees/1`; session 0 works in the main checkout.
    a.state.session = 1;
    a.toggle_tree();
    // As a click on its letter: `v` would show the hint over the tree (FILES-73, FILES-74).
    a.show_tree_view(view);
    // FILES-79: the text views walk the files on a thread.
    a.settle_tree();
    a
}

/// Tick until the worktree changed view has the answer of git (ADR 0009).
fn settle(a: &mut App) {
    let end = Instant::now() + Duration::from_secs(20);
    while a.files.tree().scanning() {
        assert!(Instant::now() < end, "timeout");
        std::thread::sleep(Duration::from_millis(10));
        a.tick();
    }
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
    settle(&mut a);
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
    settle(&mut a);
    assert!(files(&a).contains(&".worktrees/1/c.md".to_string()), "{:?}", files(&a));
}

// ADR 0009, FILES-32: a switch of the session does not wait for git. The view says that it reads
// the changes, and the rows come on a later tick.
#[tokio::test]
async fn the_changed_view_reads_git_off_the_ui_thread() {
    let root = repo("thread");
    write(root.join(".worktrees/1/a.md"), "# a\n\nchanged\n");
    let mut a = tree_in(&root, View::WorktreeChanged);
    a.switch_session(2);
    a.switch_session(1);
    assert!(a.files.tree().scanning(), "git runs on its own thread");
    assert!(a.files.tree().files.is_empty(), "{:?}", a.files.tree().files);
    // A frame now, while git still runs: draw_now does not wait for the workers.
    let screen = draw_now(&mut a, 120, 30).join("\n");
    assert!(screen.contains("Reading the changes"), "{screen}");
    settle(&mut a);
    assert_eq!(files(&a), [".worktrees/1/a.md"]);
}

// ADR 0009, FILES-27: when the answer of git comes, the selection goes to the open file, as it did
// when git ran on the UI thread.
#[test]
fn the_changed_view_selects_the_open_file_when_git_answers() {
    let root = repo("select");
    for f in ["a", "b", "c"] {
        write(root.join(format!(".worktrees/1/{f}.md")), &format!("# {f}\n\nchanged\n"));
    }
    let mut a = tree_in(&root, View::Worktree);
    a.open_file(root.join(".worktrees/1/c.md"));
    key(&mut a, KeyCode::Char('v'));
    a.settle_tree();
    assert_eq!(a.files.tree().view, View::WorktreeChanged);
    settle(&mut a);
    let t = a.files.tree();
    assert_eq!(t.rows().get(t.sel).map(|r| r.rel.clone()), Some(PathBuf::from(".worktrees/1/c.md")));
}

// ADR 0009, FILES-27: a reveal while git reads selects the file in the rows of the answer, also when
// the view kept the rows of the last answer meanwhile.
#[test]
fn a_reveal_while_git_reads_selects_the_file_in_the_new_rows() {
    let root = repo("reveal");
    write(root.join(".worktrees/1/c.md"), "# c\n\nchanged\n");
    let mut a = tree_in(&root, View::WorktreeChanged);
    a.open_file(root.join(".worktrees/1/c.md"));
    settle(&mut a);
    a.toggle_tree();
    write(root.join(".worktrees/1/a.md"), "# a\n\nchanged\n");
    write(root.join(".worktrees/1/b.md"), "# b\n\nchanged\n");
    a.toggle_tree();
    settle(&mut a);
    let t = a.files.tree();
    assert_eq!(t.rows().get(t.sel).map(|r| r.rel.clone()), Some(PathBuf::from(".worktrees/1/c.md")));
}

// ADR 0009: when git fails, the worktree changed view shows the error, not an empty list.
#[test]
fn a_git_error_shows_in_the_changed_view() {
    let root = repo("error");
    git(&root, &["branch", "-m", "main", "trunk"]);
    let mut a = tree_in(&root, View::WorktreeChanged);
    settle(&mut a);
    assert!(a.files.tree().files.is_empty());
    assert!(a.files.tree().error().is_some_and(|e| e.contains("main")), "{:?}", a.files.tree().error());
    let screen = draw(&mut a, 120, 30).join("\n");
    assert!(screen.contains("Cannot read the changes"), "{screen}");
    assert!(!screen.contains("No changes against main"), "{screen}");
}

// FILES-31, FILES-32. Async: the chat of the other session starts the agent.
#[tokio::test]
async fn the_worktree_views_follow_the_agent_session() {
    let root = repo("session");
    let mut a = tree_in(&root, View::Worktree);
    assert_eq!(a.files.tree().worktree.as_deref(), Some(Path::new(".worktrees/1")));
    a.switch_session(2);
    a.settle_tree();
    assert_eq!(a.files.tree().worktree, None, "agent session 3 has no worktree");
    assert!(a.files.tree().files.is_empty(), "{:?}", a.files.tree().files);
    let screen = draw(&mut a, 120, 30).join("\n");
    assert!(screen.contains("has no") && screen.contains("worktree yet"), "{screen}");
    a.files.set_view(View::Text);
    key(&mut a, KeyCode::Char('v'));
    a.settle_tree();
    assert!(!a.files.tree().view.is_worktree(), "v skips the worktree views: {:?}", a.files.tree().view);
    a.switch_session(1);
    a.settle_tree();
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
    assert!(screen[0].contains(" a.md") && !screen[0].contains("worktree 1"), "the top line has the path in the worktree: {}", screen[0]);
    let title = screen.iter().find(|r| r.contains("worktree 1")).expect("the worktree in the head of the note");
    assert!(title.trim_end().ends_with("worktree 1") && title.contains('a'), "at the right end of the title: {title}");
    // The delta compares with the HEAD of the worktree, not of the main checkout; a worker reads
    // it (IO-1).
    let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while a.git_head(".worktrees/1/a.md").is_none() && std::time::Instant::now() < end {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(a.git_head(".worktrees/1/a.md").flatten().as_deref(), Some("# a\n"));
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
    a.threads.entry(&zendo::app::session_key(1)).busy = Some(Instant::now());
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
    let commit = zendo::agent::work::recap::last_commit(&a.cfg.root, ".worktrees/1/a.md").unwrap_or_default();
    assert!(commit.contains("agent changes a"), "{commit:?}");
    assert!(zendo::agent::work::recap::last_commit(&a.cfg.root, "a.md").is_some_and(|c| c.contains("start")), "a file of the main checkout as before");
}

// ----- The document of the session (docs/zendo/files/files-worktree-note.md) ------------------

/// `repo(name)`, where `a.md` says `body main` on `main`, `body one` in `.worktrees/1` and `body two`
/// in the worktree `.worktrees/2`. Agent session 4 has no worktree; agent session 1 works in the main
/// checkout.
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
    a.switch_session(1);
    a.settle_tree();
    a.open_file(root.join(".worktrees/1/a.md"));
    a.switch_session(2);
    a.settle_tree();
    ask(&mut a).await;
    assert_eq!(attached(&a, 2).as_deref(), Some(".worktrees/2/a.md"));
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-39, FILES-42
#[tokio::test]
async fn choosing_a_session_shows_its_version_of_the_document() {
    let root = two_worktrees("choose");
    let mut a = app(&root);
    a.switch_session(1);
    a.settle_tree();
    a.open_file(root.join("a.md"));
    assert_eq!(a.files.current_path(), Some(root.join(".worktrees/1/a.md").as_path()), "a file opens in the version of the session");
    a.focus = Focus::Note;
    assert!(screen(&mut a).contains("body one"));
    a.switch_session(2);
    a.settle_tree();
    assert!(screen(&mut a).contains("body two"));
    a.switch_session(3);
    a.settle_tree();
    assert!(screen(&mut a).contains("body main"), "no worktree: the main checkout");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-40, FILES-41, FILES-44
#[tokio::test(flavor = "multi_thread")]
async fn a_missing_document_says_so_and_comes_back() {
    let root = two_worktrees("missing");
    write(root.join(".worktrees/1/new.md"), "# new\n\nonly in one\n");
    let mut a = app(&root);
    a.switch_session(1);
    a.settle_tree();
    a.open_file(root.join(".worktrees/1/new.md"));
    a.switch_session(2);
    a.settle_tree();
    a.focus = Focus::Note;
    let s = screen(&mut a);
    assert!(s.contains("This document does not exist in this session.") && !s.contains("only in one"), "{s}");
    ask(&mut a).await;
    assert_eq!(attached(&a, 2), None, "a question without a note file");
    a.switch_session(1);
    a.settle_tree();
    assert!(screen(&mut a).contains("only in one"), "back in its session");
    a.switch_session(2);
    a.settle_tree();
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
    a.switch_session(2);
    a.settle_tree();
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
    a.switch_session(1);
    a.settle_tree();
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
    a.switch_session(1);
    a.settle_tree();
    a.start_write(true);
    for c in "idea\n\nwritten in one".chars() {
        key(&mut a, if c == '\n' { KeyCode::Enter } else { KeyCode::Char(c) });
    }
    a.save();
    a.settle_saves();
    let saved = a.files.current().cloned().unwrap();
    assert!(saved.starts_with(root.join(".worktrees/1/docs/notes")) && saved.exists(), "{}", saved.display());
    assert!(!root.join("docs/notes").exists(), "nothing in the main checkout");
    a.mode = zendo::app::Mode::Read;
    assert!(screen(&mut a).contains("written in one"));
    a.switch_session(3);
    a.settle_tree();
    a.start_write(true);
    for c in "other".chars() {
        key(&mut a, KeyCode::Char(c));
    }
    a.save();
    a.settle_saves();
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
        user_config: root.join("zendo-config.toml"),
    })
    .unwrap();
    a.switch_session(1);
    a.settle_tree();
    a.start_write(true);
    for c in "kept".chars() {
        key(&mut a, KeyCode::Char(c));
    }
    a.save();
    a.settle_saves();
    let saved = a.files.current().cloned().unwrap();
    assert!(saved.starts_with(&notes) && saved.exists(), "{}", saved.display());
    assert!(!root.join(".worktrees/1/docs").exists(), "nothing in the worktree");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&notes);
}

/// Open a file as the user does in the active session, and let zendo see it.
fn open_in(a: &mut App, root: &Path, rel: &str) {
    a.focus = Focus::Note;
    a.open_file(root.join(rel));
    a.tick();
}

fn current(a: &App, root: &Path) -> String {
    a.files.current_path().and_then(|p| p.strip_prefix(root).ok()).map(|p| p.display().to_string()).unwrap_or_default()
}

// FILES-50, FILES-51
#[tokio::test]
async fn a_session_opens_its_last_document_again() {
    let root = two_worktrees("last-doc");
    let mut a = app(&root);
    a.switch_session(1);
    a.settle_tree();
    open_in(&mut a, &root, "a.md");
    a.switch_session(2);
    a.settle_tree();
    open_in(&mut a, &root, "b.md");
    a.switch_session(1);
    a.settle_tree();
    assert_eq!(current(&a, &root), ".worktrees/1/a.md", "back in session 2: its last document");
    a.focus = Focus::Note;
    assert!(screen(&mut a).contains("body one"));
    a.switch_session(2);
    a.settle_tree();
    assert_eq!(current(&a, &root), ".worktrees/2/b.md", "back in session 3: its last document");
    a.switch_session(3);
    a.settle_tree();
    assert_eq!(current(&a, &root), "b.md", "no last document: the open document stays");
    std::fs::remove_file(root.join(".worktrees/1/a.md")).unwrap();
    a.switch_session(1);
    a.settle_tree();
    assert_eq!(current(&a, &root), ".worktrees/1/b.md", "the last document is gone: the open document stays");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-52
#[tokio::test]
async fn the_last_document_of_a_session_stays_after_a_restart() {
    let root = two_worktrees("last-doc-restart");
    let mut a = app(&root);
    a.switch_session(2);
    a.settle_tree();
    open_in(&mut a, &root, "c.md");
    a.switch_session(3);
    a.settle_tree();
    open_in(&mut a, &root, "b.md");
    drop(a);
    let mut a = app(&root);
    a.switch_session(2);
    a.settle_tree();
    assert_eq!(current(&a, &root), ".worktrees/2/c.md");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-53
#[tokio::test]
async fn the_last_document_of_a_session_follows_a_rename() {
    let root = two_worktrees("last-doc-rename");
    let mut a = app(&root);
    a.switch_session(3);
    a.settle_tree();
    open_in(&mut a, &root, "b.md");
    a.switch_session(4);
    a.settle_tree();
    open_in(&mut a, &root, "c.md");
    a.files.ask(zendo::app::FilePrompt::Rename(PathBuf::from("b.md"), "b.md".into()));
    for _ in 0.."b.md".len() {
        key(&mut a, KeyCode::Backspace);
    }
    for c in "beta.md".chars() {
        key(&mut a, KeyCode::Char(c));
    }
    key(&mut a, KeyCode::Enter);
    assert!(root.join("beta.md").exists());
    a.switch_session(3);
    a.settle_tree();
    assert_eq!(current(&a, &root), "beta.md");
    let _ = std::fs::remove_dir_all(&root);
}

/// A long document in `.worktrees/1`: the view must scroll to show its end.
fn long_doc(root: &Path) {
    let body: String = (1..=200).map(|i| format!("line {i}\n\n")).collect();
    write(root.join(".worktrees/1/long.md"), &format!("# long\n\n{body}"));
}

/// The place of the read cursor and the view: (scroll, row).
fn place(a: &App) -> (usize, usize) {
    (a.reader.scroll(), a.reader.row())
}

/// Down in read mode `n` times, with a frame before and after, as the user does.
fn scroll_down(a: &mut App, n: usize) {
    a.focus = Focus::Note;
    screen(a);
    for _ in 0..n {
        key(a, KeyCode::Down);
    }
    screen(a);
    a.tick();
}

// FILES-54
#[tokio::test]
async fn a_session_goes_back_to_the_place_in_its_last_document() {
    let root = two_worktrees("last-place");
    long_doc(&root);
    let mut a = app(&root);
    a.switch_session(1);
    a.settle_tree();
    open_in(&mut a, &root, ".worktrees/1/long.md");
    scroll_down(&mut a, 80);
    let left = place(&a);
    assert!(left.0 > 0, "the view scrolled: {left:?}");
    a.switch_session(2);
    a.settle_tree();
    open_in(&mut a, &root, "b.md");
    assert_eq!(place(&a), (0, 0), "another document starts at the top");
    a.switch_session(1);
    a.settle_tree();
    screen(&mut a);
    assert_eq!(place(&a), left, "back where the user left it");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-52, FILES-55
#[tokio::test]
async fn zendo_starts_on_the_last_document_of_the_active_session() {
    let root = two_worktrees("last-start");
    long_doc(&root);
    let mut a = app(&root);
    a.switch_session(2);
    a.settle_tree();
    open_in(&mut a, &root, "c.md");
    a.switch_session(1);
    a.settle_tree();
    open_in(&mut a, &root, ".worktrees/1/long.md");
    scroll_down(&mut a, 80);
    let left = place(&a);
    a.quit = true;
    assert!(a.settle_quit());
    drop(a);
    let mut a = app(&root);
    assert_eq!(current(&a, &root), ".worktrees/1/long.md");
    a.focus = Focus::Note;
    screen(&mut a);
    assert_eq!(place(&a), left);
    let _ = std::fs::remove_dir_all(&root);
}

// SESSIONS-23, FILES-81: agent session 0 has the changed view of the main checkout against
// `origin/main`: commits that origin does not have, and changes that are not committed.
#[test]
fn session_0_shows_the_changes_against_origin_main() {
    let root = repo("session-0-changed");
    let start = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["update-ref", "refs/remotes/origin/main", start.trim()]);
    write(root.join("b.md"), "# b\n\ncommitted\n");
    git(&root, &["commit", "-q", "-am", "local"]);
    write(root.join("c.md"), "# c\n\nnot committed\n");
    let mut a = app(&root);
    a.state.session = 0;
    a.toggle_tree();
    a.show_tree_view(View::WorktreeChanged);
    settle(&mut a);
    let rows: Vec<String> = a.files.tree().rows().iter().map(|r| r.rel.display().to_string()).collect();
    assert_eq!(rows, vec!["b.md", "c.md"], "{rows:?}");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-82: agent session 0 shows `W` in the strip, but the worktree view cannot be chosen: a click
// and `v` pass it.
#[test]
fn session_0_cannot_choose_the_worktree_view() {
    let root = repo("session-0-no-w");
    let mut a = app(&root);
    a.state.session = 0;
    a.toggle_tree();
    let screen = draw(&mut a, 120, 30);
    assert!(screen.iter().any(|l| l.starts_with('W')), "W shows: {}", screen.join("\n"));
    assert!(!a.frame.view_buttons().iter().any(|(_, v)| *v == View::Worktree), "W takes no click");
    a.show_tree_view(View::Text);
    a.focus = Focus::Tree;
    key(&mut a, KeyCode::Char('v'));
    assert_eq!(a.files.tree().view, View::WorktreeChanged, "v passes W");
    let _ = std::fs::remove_dir_all(&root);
}
