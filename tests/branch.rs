//! The branch of the context in the top line and in the float (FILES-59 to FILES-65), in a real git
//! repository on `main` with a worktree on the branch `agent` for agent session 1.

mod common;
use common::{app, draw, key};
use ratatui::crossterm::event::KeyCode;
use std::path::{Path, PathBuf};
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

/// A git repository in its own folder on `branch`, with `a.md`. A run that stopped before leaves
/// nothing behind.
fn git_repo(name: &str, branch: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-branch-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    git(&d, &["init", "-q", "-b", branch]);
    write(d.join("a.md"), "# a\n\nbody main\n");
    write(d.join(".gitignore"), "/.worktrees/\n/.zendo/\n");
    git(&d, &["add", "."]);
    git(&d, &["commit", "-q", "-m", "start"]);
    d
}

/// The repository on `main`, and `.worktrees/1` on the branch `agent`.
fn repo(name: &str) -> PathBuf {
    let d = git_repo(name, "main");
    git(&d, &["worktree", "add", "-q", ".worktrees/1", "-b", "agent"]);
    write(d.join(".worktrees/1/a.md"), "# a\n\nbody one\n");
    d
}

/// The top line of the screen.
fn top(a: &mut App) -> String {
    draw(a, 120, 30)[0].clone()
}

/// The words of the top line.
fn words(a: &mut App) -> Vec<String> {
    top(a).split_whitespace().map(str::to_string).collect()
}

fn names(a: &mut App, branch: &str) -> bool {
    words(a).iter().any(|w| w == branch)
}

// FILES-59, FILES-65
#[tokio::test]
async fn the_top_line_names_the_branch_of_the_session() {
    let root = repo("session");
    let mut a = app(&root);
    a.switch_session(0);
    a.focus = Focus::Claude;
    assert!(names(&mut a, "agent") && !names(&mut a, "main"), "session 1: its worktree\n{}", top(&mut a));
    a.switch_session(1);
    assert!(names(&mut a, "main") && !names(&mut a, "agent"), "session 2 has no worktree: the main checkout\n{}", top(&mut a));
    git(&root, &["checkout", "-q", "-b", "trunk"]);
    assert!(names(&mut a, "trunk"), "a new branch shows without a restart\n{}", top(&mut a));
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-60
#[tokio::test]
async fn the_top_line_names_the_branch_of_the_document() {
    let root = repo("doc");
    let notes = git_repo("doc-notes", "notes");
    let mut a = app(&root);
    a.switch_session(1);
    a.open_file(notes.join("a.md"));
    a.focus = Focus::Note;
    assert!(names(&mut a, "notes"), "the note pane: the branch of the document\n{}", top(&mut a));
    a.focus = Focus::Claude;
    assert!(names(&mut a, "main") && !names(&mut a, "notes"), "the chat: the branch of the session\n{}", top(&mut a));
    a.switch_session(0);
    a.open_file(root.join("a.md"));
    a.focus = Focus::Note;
    assert!(names(&mut a, "agent"), "the version of session 1 is in its worktree\n{}", top(&mut a));
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&notes);
}

/// The tree open in `view`, with the focus.
fn tree_in(a: &mut App, view: View) {
    if !a.state.tree {
        a.toggle_tree();
    }
    while a.files.tree().view != view {
        key(a, KeyCode::Char('v'));
    }
    a.focus = Focus::Tree;
}

// FILES-61
#[tokio::test]
async fn the_top_line_names_the_branch_of_the_view() {
    let root = repo("view");
    let mut a = app(&root);
    a.switch_session(0);
    a.focus = Focus::Tree;
    tree_in(&mut a, View::Text);
    assert!(names(&mut a, "main") && !names(&mut a, "agent"), "the text view: the main checkout\n{}", top(&mut a));
    tree_in(&mut a, View::Worktree);
    assert!(names(&mut a, "agent") && !names(&mut a, "main"), "the worktree view: the worktree\n{}", top(&mut a));
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-64
#[tokio::test]
async fn a_worktree_view_without_a_worktree_names_no_branch() {
    let root = repo("no-worktree");
    let mut a = app(&root);
    a.switch_session(0);
    tree_in(&mut a, View::Worktree);
    a.switch_session(1);
    a.focus = Focus::Tree;
    assert!(a.files.tree().view.is_worktree(), "the view stays");
    assert!(!names(&mut a, "main") && !names(&mut a, "agent"), "no branch\n{}", top(&mut a));
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-63
#[tokio::test]
async fn holding_m_and_u_names_their_branch() {
    let root = repo("peek");
    git(&root, &["checkout", "-q", "-b", "trunk"]);
    let mut a = app(&root);
    a.switch_session(0);
    a.open_file(root.join("a.md"));
    a.focus = Focus::Note;
    assert!(names(&mut a, "agent"), "{}", top(&mut a));
    key(&mut a, KeyCode::Char('m'));
    assert!(names(&mut a, "main") && !names(&mut a, "agent"), "m: main\n{}", top(&mut a));
    a.files.peek_mut().on = None;
    key(&mut a, KeyCode::Char('u'));
    assert!(names(&mut a, "trunk") && !names(&mut a, "agent"), "u: the main checkout\n{}", top(&mut a));
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-62
#[tokio::test]
async fn the_float_names_the_branch_of_the_document() {
    let root = repo("float");
    let notes = git_repo("float-notes", "notes");
    let mut a = app(&root);
    a.switch_session(0);
    a.open_file(notes.join("a.md"));
    a.focus = Focus::Claude;
    a.open_float();
    let screen = draw(&mut a, 120, 40);
    let line = screen.iter().find(|l| l.contains("close ]")).unwrap_or_else(|| panic!("no float:\n{}", screen.join("\n")));
    assert!(line.contains("agent"), "a new note goes to the checkout of the session (FILES-49)\n{line}");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&notes);
}

// FILES-80: the branch stands at the left end of the top line, before the path of the document, and
// the buttons stay at the right end.
#[tokio::test]
async fn the_branch_stands_left_of_the_path() {
    let root = repo("left");
    let mut a = app(&root);
    a.switch_session(0);
    a.open_file(root.join(".worktrees/1/a.md"));
    a.focus = Focus::Note;
    let line = top(&mut a);
    assert!(line.trim_start().starts_with("agent · a.md"), "{line}");
    assert!(line.trim_end().ends_with("[sound]"), "{line}");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}
