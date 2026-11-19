//! Responsiveness of actions (docs/zendo/responsiveness.md): zendo measures the time of each action
//! of the keymap at the place where it counts the action, puts it into the event of the action
//! (TELEM-1), and the code of an action holds no measurement code.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use zendo::app::App;
use zendo::keys::Pane;
use zendo::responsiveness::Responsiveness;
use zendo::telemetry::Telemetry;

const NOTE: &str = "docs/notes/2026-11-19-a.md";

/// An app of `root` that records into the data folder `data`.
fn timing(root: &Path, data: &Path) -> App {
    let _ = std::fs::remove_dir_all(data);
    let mut a = app(root);
    a.keys = zendo::keys::Keymap::defaults();
    (a.telemetry, a.responsiveness) = (Telemetry::start(data, std::process::id()), Responsiveness::on());
    a
}

/// The waits of the events of `name` of `pane`, in milliseconds; `None` for an event without a time.
fn waits(a: &App, pane: Pane, name: &str) -> Vec<Option<u128>> {
    a.telemetry.waiting().iter().filter(|e| e.pane == pane && e.action == name).map(|e| e.wait.map(|w| w.as_millis())).collect()
}

/// The key `alt-c` read at `t`, and the draw of its screen 10 ms later.
fn alt_in_10_ms(a: &mut App, c: char, t: Instant) {
    a.input_at(t);
    a.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT));
    draw(a, 140, 30);
    a.drawn_at(t + Duration::from_millis(10));
}

// RESP-1: each action of the keymap gets the time from its key to the draw of its screen, under
// its keymap pane and its name.
#[tokio::test]
async fn every_action_gets_a_time() {
    let root = repo("responsiveness-time", &[(NOTE, "# A\n\nBefore the table there is text.\n")]);
    let data = std::env::temp_dir().join(format!("zendo-responsiveness-data-time-{}", std::process::id()));
    let mut a = timing(&root, &data);
    draw(&mut a, 140, 30);
    let t0 = Instant::now();
    alt_in_10_ms(&mut a, 'a', t0);
    key(&mut a, KeyCode::Esc);
    draw(&mut a, 140, 30);
    alt_in_10_ms(&mut a, 'a', t0 + Duration::from_secs(1));
    key(&mut a, KeyCode::Esc);
    alt_in_10_ms(&mut a, 'b', t0 + Duration::from_secs(2));
    assert_eq!(waits(&a, Pane::Read, "add_to_chat"), [Some(10), Some(10)], "each run took 10 ms");
    assert_eq!(waits(&a, Pane::Global, "toggle_tree"), [Some(10)], "the global action");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&data);
}

// RESP-1: an action that starts a turn of the agent waits for the end of the turn: its time spans
// the wait for the answer, not only the first screen.
#[tokio::test]
async fn an_action_waits_for_the_work_that_it_started() {
    let root = repo("responsiveness-wait", &[(NOTE, "# A\n\nSome text.\n")]);
    let data = std::env::temp_dir().join(format!("zendo-responsiveness-data-wait-{}", std::process::id()));
    let mut a = timing(&root, &data);
    draw(&mut a, 140, 30);
    let t0 = Instant::now();
    a.drawn_at(t0);
    a.input_at(t0);
    a.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT));
    // The action started a turn: the thread of the chat answers.
    let key = a.key();
    let t = a.thread(&key);
    (t.busy, t.turn) = (Some(t0), Some(1));
    draw(&mut a, 140, 30);
    a.drawn_at(t0 + Duration::from_millis(10));
    assert!(waits(&a, Pane::Global, "toggle_tree").is_empty(), "the screen shows the turn, the answer is not there");
    // The note moves while the agent answers: the thread moves to the new key, the answer is the same.
    let key = format!("{key}-moved");
    a.threads.rename(&a.key(), key.clone());
    a.drawn_at(t0 + Duration::from_millis(20));
    assert!(waits(&a, Pane::Global, "toggle_tree").is_empty(), "the move does not end the wait");
    a.thread(&key).busy = None;
    a.drawn_at(t0 + Duration::from_millis(1500));
    assert_eq!(waits(&a, Pane::Global, "toggle_tree"), [Some(1500)], "the time after the answer");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&data);
}

/// The files of `src`, outside the module of the measurements, that use it, with their reason.
const HOOKS: [(&str, &str); 7] = [
    ("src/lib.rs", "declares the module"),
    ("src/main.rs", "turns the measurements and the log on at the start"),
    ("src/app/mod.rs", "the field of the app"),
    ("src/app/count.rs", "the one place where the actions of the keymap are counted and timed, with the input, the draw and the work in the background"),
    ("src/app/files.rs", "the tick that writes the log"),
    ("src/telemetry/mod.rs", "deletes the old folder `responsiveness/` of the data folder (TELEM-14)"),
    ("src/tui/mod.rs", "the terminal loop: the read of each input event and the end of each draw"),
];

/// The words that use the measurements: the module, and the marks of the input and of the draw.
const USES: [&str; 3] = ["responsiveness", ".input_at(", ".drawn_at("];

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

// RESP-2: the measurement is in one place; no action calls it.
#[test]
fn actions_hold_no_measurement_code() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    sources(&root.join("src"), &mut files);
    let users: Vec<String> = files
        .iter()
        .map(|p| p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"))
        .filter(|f| !f.starts_with("src/responsiveness/"))
        .filter(|f| {
            let text = std::fs::read_to_string(root.join(f)).unwrap();
            USES.iter().any(|w| text.contains(w))
        })
        .collect();
    let other: Vec<&String> = users.iter().filter(|f| !HOOKS.iter().any(|(h, _)| h == f)).collect();
    assert!(other.is_empty(), "these files measure time outside the one place (RESP-2); measure in App::did:\n{other:#?}");
    let count = std::fs::read_to_string(root.join("src/app/count.rs")).unwrap();
    assert_eq!(count.matches("self.responsiveness.ran(").count(), 1, "one call, in the hook that every action passes");
    let tui = std::fs::read_to_string(root.join("src/tui/mod.rs")).unwrap();
    assert_eq!((tui.matches(".input_at(").count(), tui.matches(".drawn_at(").count()), (1, 1), "the loop marks each input and each draw once");
}

// RESP-1: the time starts when the input thread reads the event, not when the loop takes it from
// the queue: a busy loop is in the time.
#[tokio::test]
async fn the_time_starts_at_the_read_of_the_event() {
    let root = repo("responsiveness-read", &[(NOTE, "# A\n\nSome text.\n")]);
    let data = std::env::temp_dir().join(format!("zendo-responsiveness-data-read-{}", std::process::id()));
    let mut a = timing(&root, &data);
    draw(&mut a, 140, 30);
    let read = Instant::now();
    a.drawn_at(read);
    // The loop takes the key 400 ms after the input thread read it.
    let key = ratatui::crossterm::event::Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT));
    zendo::tui::on_input(&mut a, zendo::tui::Input { event: key, read });
    draw(&mut a, 140, 30);
    a.drawn_at(read + Duration::from_millis(410));
    assert_eq!(waits(&a, Pane::Global, "toggle_tree"), [Some(410)], "410 ms from the read");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&data);
}
