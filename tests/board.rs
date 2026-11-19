//! The message board between zendo sessions (docs/zendo/board.md, board-posts.md, board-leases.md,
//! board-watch.md): the MCP server of one session writes, the zendo app delivers to another.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use zendo::app::{App, BoardLink, Focus, Item, session_key};
use zendo::board::tools::{Ctx, call};

/// The folder of the board of a test: an interrupted earlier run leaves nothing behind.
fn board_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-board-it-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// An app on the repository `name` with the board in `dir`.
fn app_on(root: &Path, dir: &Path) -> App {
    let mut a = app(root);
    a.board = BoardLink::open(dir, root);
    a
}

/// The MCP server of session `s` of the repository of `a`.
fn server(a: &App, s: usize) -> Ctx {
    Ctx { journal: a.board.journal.clone().unwrap(), repo: a.board.repo.clone(), session: s, exe: "zendo".into(), choose_for: Duration::from_secs(5) }
}

fn tool(c: &Ctx, name: &str, args: Value) -> Value {
    call(c, name, &args).unwrap().unwrap()
}

/// The questions in the thread of session `n` that came from the board.
fn from_board(a: &App, n: usize) -> Vec<String> {
    a.threads.get(&session_key(n)).map_or(Vec::new(), |t| t.items.iter().filter_map(|i| if let Item::User(u) = i { Some(u.clone()) } else { None }).filter(|u| u.starts_with("from ")).collect())
}

/// Tick the apps and feed their agent events until `done` holds.
async fn drive(apps: &mut [&mut App], secs: u64, done: impl Fn(&[&mut App]) -> bool) {
    let end = Instant::now() + Duration::from_secs(secs);
    while !done(apps) {
        assert!(Instant::now() < end, "timeout");
        for a in apps.iter_mut() {
            a.tick();
            if let Some(rx) = a.link.events_mut()
                && let Ok(Some(e)) = tokio::time::timeout(Duration::from_millis(20), rx.recv()).await
            {
                a.on_agent(e);
            }
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

// BOARD-1, BOARD-2, BOARD-11: a task to the inbox of session 2 goes to its thread as one question
// with the label of the sender, once, also with two zendo processes on the repository; the turn
// that took it lands it (BOARD-9).
#[tokio::test]
async fn a_task_reaches_the_inbox_once() {
    let root = repo("board-inbox", &[("README.txt", "x\n")]);
    let dir = board_dir("inbox");
    let (mut a, mut b) = (app_on(&root, &dir), app_on(&root, &dir));
    tool(&server(&a, 3), "assign", json!({ "to": "session:2", "text": "rebase feat/oci onto main" }));
    drive(&mut [&mut a, &mut b], 20, |x| x.iter().map(|a| from_board(a, 2).len()).sum::<usize>() == 1 && x.iter().all(|a| a.threads.get(&session_key(2)).is_none_or(|t| t.busy.is_none()))).await;
    let got: Vec<String> = [&a, &b].iter().flat_map(|a| from_board(a, 2)).collect();
    assert_eq!(got, ["from 3 · task: rebase feat/oci onto main"]);
    tokio::time::sleep(Duration::from_millis(400)).await;
    drive(&mut [&mut a, &mut b], 5, |x| x[0].board.state.taken(&x[0].board.repo, 2).is_empty()).await;
    assert_eq!([&a, &b].iter().map(|a| from_board(a, 2).len()).sum::<usize>(), 1, "never twice");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&dir);
}

// BOARD-3: a question to `all` goes only to the sessions whose agent runs.
#[tokio::test]
async fn all_skips_sessions_without_an_agent() {
    let root = repo("board-all", &[("README.txt", "x\n")]);
    let dir = board_dir("all");
    let mut a = app_on(&root, &dir);
    a.thread(&session_key(2)).session = Some("s2".into());
    tool(&server(&a, 3), "ask", json!({ "to": "all", "text": "who uses port 5220?" }));
    drive(&mut [&mut a], 20, |x| from_board(x[0], 2).len() == 1).await;
    assert!(from_board(&a, 4).is_empty() && from_board(&a, 1).is_empty());
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&dir);
}

// BOARD-3, BOARD-6: while session 2 answers, its messages wait; at the end of the turn they go as
// one question.
#[tokio::test]
async fn messages_wait_for_the_turn_and_go_as_one() {
    let root = repo("board-wait", &[("README.txt", "x\n")]);
    let dir = board_dir("wait");
    let mut a = app_on(&root, &dir);
    a.switch_session(2);
    a.focus = Focus::Claude;
    typ(&mut a, "slow work");
    key(&mut a, KeyCode::Enter);
    let s3 = server(&a, 3);
    tool(&s3, "ask", json!({ "to": "session:2", "text": "one" }));
    tool(&s3, "ask", json!({ "to": "session:2", "text": "two" }));
    let t0 = Instant::now();
    drive(&mut [&mut a], 3, |_| t0.elapsed() > Duration::from_secs(1)).await;
    assert!(from_board(&a, 2).is_empty(), "nothing goes while the agent answers");
    drive(&mut [&mut a], 20, |x| !from_board(x[0], 2).is_empty()).await;
    assert_eq!(from_board(&a, 2), ["from 3 · question: one\nfrom 3 · question: two"]);
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&dir);
}

// BOARD-10, BOARD-12: a task without a target asks the user in the chat of the sender; the choice
// goes back to the tool.
#[tokio::test(flavor = "multi_thread")]
async fn assign_without_a_target_asks_the_user() {
    let root = repo("board-choose", &[("README.txt", "x\n")]);
    let dir = board_dir("choose");
    let mut a = app_on(&root, &dir);
    let s3 = server(&a, 3);
    let h = std::thread::spawn(move || call(&s3, "assign", &json!({ "text": "take the OCI fix" })).unwrap());
    drive(&mut [&mut a], 10, |x| x[0].threads.get(&session_key(3)).is_some_and(|t| t.asking().is_some())).await;
    a.switch_session(3);
    a.focus = Focus::Claude;
    key(&mut a, KeyCode::Down);
    key(&mut a, KeyCode::Enter);
    let v = h.join().unwrap().unwrap();
    assert_eq!(v["to"], "session:1", "the second choice: sessions 0, 1, 2, 4");
    drive(&mut [&mut a], 20, |x| from_board(x[0], 1).len() == 1).await;
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Tick until the screen of Claude Code session 0 shows `text`. The screen scrolls, so the test
/// looks at each tick.
async fn screen(a: &mut App, text: &str) {
    let end = Instant::now() + Duration::from_secs(10);
    while !a.cli.text(0).contains(text) {
        assert!(Instant::now() < end, "no {text:?} on the screen: {}", a.cli.text(0));
        a.tick();
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

// BOARD-8: while Claude Code runs for session 0, the message goes to Claude Code.
#[tokio::test(flavor = "multi_thread")]
async fn a_message_reaches_claude_code() {
    let root = repo("board-cli", &[("README.txt", "x\n")]);
    let dir = board_dir("cli");
    let mut a = app_on(&root, &dir);
    a.cli.command = Some(zendo::session::Command { program: env!("CARGO_BIN_EXE_zendo-mock-cli").into(), args: vec!["--resume".into()] });
    a.thread(&session_key(0)).session = Some("s1".into());
    a.open_panel();
    a.switch_session(0);
    a.focus = Focus::Claude;
    ctrl(&mut a, 'o');
    screen(&mut a, "tick 1").await;
    a.on_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT));
    // The mock reads raw keys and quits on `q`, so the test sends a task: no `q` in its tag.
    tool(&server(&a, 3), "assign", json!({ "to": "session:0", "text": "rebase the branch" }));
    screen(&mut a, "rebase the branch").await;
    assert!(a.threads[&session_key(0)].busy.is_none(), "no prompt went to the agent");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&dir);
}

// BOARD-17: the next question of a session carries the posts that it did not see, one time.
#[tokio::test]
async fn a_question_carries_unseen_posts_once() {
    let root = repo("board-posts", &[("README.txt", "x\n")]);
    let dir = board_dir("posts");
    let mut a = app_on(&root, &dir);
    let mut other = server(&a, 2);
    other.repo = "/another/repo".into();
    tool(&other, "post", json!({ "resource": "port:5220", "text": "dev server" }));
    drive(&mut [&mut a], 5, |x| x[0].board.state.live_posts().count() == 1).await;
    let first = a.board_posts_context(1);
    assert!(first.contains("port:5220") && first.contains("/another/repo"), "{first}");
    assert_eq!(a.board_posts_context(1), "", "one time");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&dir);
}

// BOARD-31, BOARD-30: the lease of a post with a process expires when the process ends.
#[tokio::test]
async fn a_lease_ends_with_its_process() {
    let root = repo("board-lease", &[("README.txt", "x\n")]);
    let dir = board_dir("lease");
    let mut a = app_on(&root, &dir);
    let mut child = std::process::Command::new("sleep").arg("30").spawn().unwrap();
    tool(&server(&a, 2), "post", json!({ "resource": "port:7000", "text": "server", "pid": child.id() }));
    drive(&mut [&mut a], 5, |x| x[0].board.state.live_posts().count() == 1).await;
    tokio::time::sleep(Duration::from_millis(1200)).await;
    drive(&mut [&mut a], 3, |x| x[0].board.state.live_posts().all(|p| !p.expired())).await;
    child.kill().unwrap();
    child.wait().unwrap();
    drive(&mut [&mut a], 5, |x| x[0].board.state.live_posts().all(|p| p.expired())).await;
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&dir);
}

// BOARD-37: after the fuse, the messages wait, and the chat says so once; a question of the user
// lets them go.
#[tokio::test]
async fn the_fuse_holds_a_chain_until_the_user_asks() {
    let root = repo("board-fuse", &[("README.txt", "x\n")]);
    let dir = board_dir("fuse");
    let mut a = app_on(&root, &dir);
    a.board.turns = zendo::board::FUSE;
    tool(&server(&a, 3), "ask", json!({ "to": "session:2", "text": "again" }));
    let t0 = Instant::now();
    drive(&mut [&mut a], 3, |_| t0.elapsed() > Duration::from_millis(800)).await;
    assert!(from_board(&a, 2).is_empty(), "held");
    let infos = a.threads[&session_key(2)].items.iter().filter(|i| matches!(i, Item::Info(t) if t.contains("in a row"))).count();
    assert_eq!(infos, 1);
    a.switch_session(1);
    a.focus = Focus::Claude;
    typ(&mut a, "hello");
    key(&mut a, KeyCode::Enter);
    drive(&mut [&mut a], 20, |x| from_board(x[0], 2).len() == 1).await;
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&dir);
}

// BOARD-21: a message that a watch printed shows in the chat of its session as watched, and does
// not go to the agent again.
#[tokio::test]
async fn a_watched_message_shows_as_watched() {
    let root = repo("board-watched", &[("README.txt", "x\n")]);
    let dir = board_dir("watched");
    let mut a = app_on(&root, &dir);
    let id = tool(&server(&a, 3), "ask", json!({ "to": "session:2", "text": "port?" }))["id"].as_str().unwrap().to_string();
    assert!(a.board.journal.as_ref().unwrap().claim(&id, 2, zendo::board::How::Watched));
    drive(&mut [&mut a], 5, |x| from_board(x[0], 2).len() == 1).await;
    assert_eq!(from_board(&a, 2), ["from 3 · question: port? · watched"]);
    assert!(a.threads[&session_key(2)].busy.is_none(), "no turn for it");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&dir);
}

// BOARD-35: the author of a tombstoned post hears it with its next question, once.
#[tokio::test]
async fn the_author_hears_of_a_tombstone_once() {
    let root = repo("board-tomb", &[("README.txt", "x\n")]);
    let dir = board_dir("tomb");
    let mut a = app_on(&root, &dir);
    let p = tool(&server(&a, 2), "post", json!({ "resource": "docker:foo", "text": "kind" }))["id"].as_str().unwrap().to_string();
    let j = a.board.journal.clone().unwrap();
    j.append(&[zendo::board::Event::Expired { post: p.clone() }, zendo::board::Event::Tombstone { post: p, repo: "/other".into(), session: 1 }]).unwrap();
    drive(&mut [&mut a], 5, |x| x[0].board.state.history().count() == 1).await;
    let text = a.board_posts_context(2);
    assert!(text.contains("tombstoned by session 1 of /other"), "{text}");
    assert_eq!(a.board_posts_context(2), "");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&dir);
}
