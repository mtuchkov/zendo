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
    while a.tree.view != view {
        key(&mut a, KeyCode::Char('v'));
    }
    a
}

fn files(a: &App) -> Vec<String> {
    a.tree.files.iter().map(|p| p.display().to_string()).collect()
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
    let rows: Vec<String> = a.tree.rows().iter().map(|r| r.name.clone()).collect();
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
    assert!(a.tree.deleted.contains(Path::new(".worktrees/1/b.md")), "b.md is deleted");

    // A deleted file does not open.
    a.focus = Focus::Tree;
    a.tree.reveal(Path::new(".worktrees/1/b.md"));
    key(&mut a, KeyCode::Enter);
    assert!(a.current.as_ref().is_none_or(|p| !p.ends_with("b.md")), "{:?}", a.current);

    // A new change shows without a restart.
    write(wt.join("c.md"), "# c\n\nchanged too\n");
    a.tree.scanned = None;
    a.tick();
    assert!(files(&a).contains(&".worktrees/1/c.md".to_string()), "{:?}", files(&a));
}

// FILES-31, FILES-32. Async: the chat of the other session starts the agent.
#[tokio::test]
async fn the_worktree_views_follow_the_agent_session() {
    let root = repo("session");
    let mut a = tree_in(&root, View::Worktree);
    assert_eq!(a.tree.worktree.as_deref(), Some(Path::new(".worktrees/1")));
    a.switch_session(1);
    assert_eq!(a.tree.worktree, None, "agent session 2 has no worktree");
    assert!(a.tree.files.is_empty(), "{:?}", a.tree.files);
    let screen = draw(&mut a, 120, 30).join("\n");
    assert!(screen.contains("has no") && screen.contains("worktree yet"), "{screen}");
    a.tree.view = View::Folders;
    key(&mut a, KeyCode::Char('v'));
    assert!(!a.tree.view.is_worktree(), "v skips the worktree views: {:?}", a.tree.view);
    a.switch_session(0);
    assert_eq!(a.tree.worktree.as_deref(), Some(Path::new(".worktrees/1")));
}

// FILES-34
#[test]
fn a_file_of_the_worktree_opens_with_the_worktree_in_its_head() {
    let root = repo("open");
    let mut a = tree_in(&root, View::Worktree);
    a.focus = Focus::Tree;
    a.tree.reveal(Path::new(".worktrees/1/a.md"));
    key(&mut a, KeyCode::Enter);
    assert_eq!(a.current.as_deref(), Some(root.join(".worktrees/1/a.md").as_path()));
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
    a.tree.reveal(Path::new(".worktrees/1/a.md"));

    // The agent is idle: the usual question.
    key(&mut a, KeyCode::Delete);
    assert!(a.file_prompt.is_some());
    key(&mut a, KeyCode::Esc);
    assert!(a.file_prompt.is_none());

    // The agent of the session answers: a popup asks first; a key other than y cancels.
    a.threads.entry("(chat)".into()).or_default().busy = Some(Instant::now());
    key(&mut a, KeyCode::Delete);
    assert!(a.file_prompt.is_none() && a.worktree.confirm.is_some());
    let screen = draw(&mut a, 120, 30).join("\n");
    assert!(screen.contains("works in this worktree"), "{screen}");
    key(&mut a, KeyCode::Char('n'));
    assert!(a.worktree.confirm.is_none() && a.file_prompt.is_none());
    key(&mut a, KeyCode::F(2));
    key(&mut a, KeyCode::Char('y'));
    assert!(matches!(a.file_prompt, Some(zendo::app::FilePrompt::Rename(..))), "after y, the question of the rename");
}

// FILES-28, FILES-29, FILES-36
#[tokio::test]
async fn the_worktree_recap_refreshes_every_10_minutes_when_it_changed() {
    let root = repo("recap");
    let wt = root.join(".worktrees/1");
    write(wt.join("a.md"), "# a\n\nchanged\n");
    let mut a = tree_in(&root, View::WorktreeChanged);
    a.ensure_agent();
    until(&mut a, |a| a.agent_name.is_some()).await;
    let recap = |a: &App| a.state.files.get(".worktrees/1").and_then(|f| f.recap.as_ref()).map(|r| (r.text.clone(), r.of_hash.clone()));

    // The first recap.
    until(&mut a, |a| recap(a).is_some()).await;
    let (text, first) = recap(&a).unwrap();
    assert!(text.contains("1 file"), "{text}");
    let screen = draw(&mut a, 120, 30).join("\n");
    assert!(screen.contains("RECAP ·") && screen.contains("The worktree changes"), "the recap is at the bottom of the tree: {screen}");

    // A new change: no recap before 10 minutes, a new one after.
    write(wt.join("n.txt"), "new\n");
    a.worktree.recap_check = Some(Instant::now() - Duration::from_secs(5 * 60));
    a.tick();
    assert!(a.recap_job.is_none(), "no recap at 5 minutes");
    a.worktree.recap_check = Some(Instant::now() - Duration::from_secs(10 * 60));
    until(&mut a, |a| recap(a).is_some_and(|(_, h)| h != first)).await;
    assert!(recap(&a).unwrap().0.contains("2 files"), "{:?}", recap(&a));

    // No change: no recap.
    a.worktree.recap_check = Some(Instant::now() - Duration::from_secs(10 * 60));
    a.tick();
    assert!(a.recap_job.is_none(), "no recap without a change");
    assert!(a.threads.values().all(|t| t.items.is_empty()), "the chat has no recap");
}
