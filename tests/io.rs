//! Input and output off the UI thread (docs/zendo/io.md, ADR 0024): keys and ticks answer while the
//! disk or a program does not.

mod common;
use common::*;
use ratatui::crossterm::event::KeyCode;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use zendo::app::{BoardLink, Clipboard, Facts, Focus};
use zendo::text::clipboard::Clip;
use zendo::worker::{LIMIT, Sync, Writer};

/// A git that takes 3 s to show a file at HEAD.
fn slow_head(_: &(PathBuf, String)) -> Option<String> {
    std::thread::sleep(Duration::from_secs(3));
    Some("# A\n".into())
}

// IO-1, IO-2, IO-6: while git does not answer, a frame with the delta pane takes at most 100 ms;
// after 350 ms it says that it reads the file at HEAD.
#[test]
fn a_frame_answers_while_git_does_not() {
    let root = repo("io-delta-git", &[("docs/notes/2026-11-19-a.md", "# A\n\n- new line\n")]);
    let mut a = app(&root);
    a.facts = Facts::with_head(slow_head);
    // A new note has no saved visit: the delta compares with git HEAD.
    a.open_file(root.join("docs/notes/2026-11-19-a.md"));
    a.toggle_delta();
    let mut screen = Vec::new();
    within_limit("a frame", || screen = draw_now(&mut a, 160, 30));
    assert!(!screen.join("\n").contains("readin"), "a short wait shows nothing:\n{}", screen.join("\n"));
    std::thread::sleep(Duration::from_millis(400));
    within_limit("a later frame", || screen = draw_now(&mut a, 160, 30));
    assert!(screen.join("\n").contains("readin"), "the delta says that it reads:\n{}", screen.join("\n"));
    let _ = std::fs::remove_dir_all(&root);
}

/// The folder of a test: an interrupted earlier run leaves nothing behind.
fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-io-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

type Got = Arc<Mutex<Vec<(PathBuf, Vec<u8>)>>>;

/// A disk that holds each write until the test lets it go, and keeps what it got.
fn gated() -> (Writer, Got, mpsc::Sender<()>) {
    let got: Got = Arc::new(Mutex::new(Vec::new()));
    let (go, wait) = mpsc::channel::<()>();
    let wait = Mutex::new(wait);
    let seen = got.clone();
    let w = Writer::with(move |p: &Path, c: &[u8], _: Sync| {
        let _ = wait.lock().unwrap().recv();
        seen.lock().unwrap().push((p.to_path_buf(), c.to_vec()));
        Ok(())
    });
    (w, got, go)
}

fn within_limit(what: &str, f: impl FnOnce()) {
    let t = Instant::now();
    f();
    assert!(t.elapsed() < LIMIT, "{what} took {:?}", t.elapsed());
}

// IO-1, IO-2, IO-4: while the disk holds the heartbeat of the board, each tick and each key returns
// within 100 ms; then the file gets the newest heartbeat.
#[test]
fn keys_answer_while_a_write_waits() {
    let root = repo("io-write-waits", &[("docs/notes/2026-11-19-a.md", "# A\n\n- one\n- two\n")]);
    let board = scratch("board");
    let mut a = app(&root);
    a.board = BoardLink::open(&board, &root);
    let (disk, got, go) = gated();
    a.disk = disk;
    for _ in 0..3 {
        within_limit("a tick", || a.tick());
        within_limit("a key", || key(&mut a, KeyCode::Down));
        std::thread::sleep(Duration::from_millis(1050));
    }
    assert!(got.lock().unwrap().is_empty(), "the disk held the first write");
    for _ in 0..10 {
        let _ = go.send(());
    }
    assert!(a.disk.flush(Duration::from_secs(5)), "the writes ended");
    let beats: Vec<Vec<u8>> = got.lock().unwrap().iter().filter(|(p, _)| p.starts_with(&board)).map(|(_, c)| c.clone()).collect();
    // Three heartbeats came while the disk waited; only the newest of those that waited went.
    assert!((1..3).contains(&beats.len()), "the heartbeats that waited became one: {}", beats.len());
    let at = |b: &[u8]| serde_json::from_slice::<serde_json::Value>(b).unwrap()["at"].as_u64().unwrap();
    assert!(beats.windows(2).all(|w| at(&w[1]) > at(&w[0])), "the newest heartbeat comes last");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&board);
}

/// Tick until `done` holds, at most 5 s; each tick returns within the limit (IO-2).
fn until(a: &mut zendo::app::App, what: &str, done: impl Fn(&zendo::app::App) -> bool) {
    let end = Instant::now() + Duration::from_secs(5);
    while !done(a) {
        assert!(Instant::now() < end, "no {what}: {:?}", a.flash);
        within_limit("a tick", || a.tick());
        std::thread::sleep(Duration::from_millis(20));
    }
}

// IO-1, IO-2, IO-5: the paste key returns at once while the clipboard does not answer; after the
// time limit the user learns that the clipboard does not answer.
#[tokio::test(flavor = "multi_thread")]
async fn paste_answers_while_the_clipboard_does_not() {
    let root = repo("io-paste-hangs", &[("docs/notes/2026-11-19-a.md", "# A\n")]);
    let mut a = app(&root);
    a.clipboard = Clipboard::with(
        || {
            std::thread::sleep(Duration::from_secs(30));
            Clip::Empty
        },
        |_| {},
    );
    a.open_panel();
    a.focus = Focus::Claude;
    within_limit("the paste key", || ctrl(&mut a, 'v'));
    until(&mut a, "failure", |a| a.flash.as_ref().is_some_and(|(s, _)| s.contains("the clipboard does not answer")));
    let _ = std::fs::remove_dir_all(&root);
}

// IO-6: the text of the clipboard lands in the chat input, where the user pasted.
#[tokio::test(flavor = "multi_thread")]
async fn a_pasted_text_lands_in_the_chat() {
    let root = repo("io-paste-text", &[("docs/notes/2026-11-19-a.md", "# A\n")]);
    let mut a = app(&root);
    a.clipboard = Clipboard::with(|| Clip::Text("hello from the clipboard".into()), |_| {});
    a.open_panel();
    a.focus = Focus::Claude;
    within_limit("the paste key", || ctrl(&mut a, 'v'));
    until(&mut a, "paste", |a| a.composer().input().text().contains("hello from the clipboard"));
    let _ = std::fs::remove_dir_all(&root);
}
