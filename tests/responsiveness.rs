//! Responsiveness of actions (docs/zendo/responsiveness.md): zendo measures the time of each action
//! of the keymap at the place where it counts the action, and the code of an action holds no
//! measurement code.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use zendo::app::App;
use zendo::keys::Pane;
use zendo::responsiveness::Responsiveness;

const NOTE: &str = "docs/notes/2026-11-19-a.md";

/// The key `alt-c` read at `t`, and the draw of its screen 10 ms later.
fn alt_in_10_ms(a: &mut App, c: char, t: Instant) {
    a.responsiveness.input(t);
    a.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT));
    draw(a, 140, 30);
    a.responsiveness.drawn(t + Duration::from_millis(10));
}

// RESP-1: each action of the keymap gets the time from its key to the draw of its screen, under
// its keymap pane and its name.
#[tokio::test]
async fn every_action_gets_a_time() {
    let root = repo("responsiveness-time", &[(NOTE, "# A\n\nBefore the table there is text.\n")]);
    let data = std::env::temp_dir().join(format!("zendo-responsiveness-data-time-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    let mut a = app(&root);
    a.keys = zendo::keys::Keymap::defaults();
    a.responsiveness = Responsiveness::start(&data, std::process::id());
    draw(&mut a, 140, 30);
    let t0 = Instant::now();
    alt_in_10_ms(&mut a, 'a', t0);
    key(&mut a, KeyCode::Esc);
    draw(&mut a, 140, 30);
    alt_in_10_ms(&mut a, 'a', t0 + Duration::from_secs(1));
    key(&mut a, KeyCode::Esc);
    alt_in_10_ms(&mut a, 'b', t0 + Duration::from_secs(2));
    let chat = a.responsiveness.times(Pane::Read, "add_to_chat").expect("the times of add_to_chat");
    assert_eq!(chat.runs(), 2);
    assert_eq!(chat.within(16) - chat.within(8), 2, "each run took 10 ms");
    let tree = a.responsiveness.times(Pane::Global, "toggle_tree").expect("the times of the global action");
    assert_eq!(tree.runs(), 1);
    assert_eq!(tree.within(16) - tree.within(8), 1);
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&data);
}

/// The files of `src`, outside the module of the measurements, that use it, with their reason.
const HOOKS: [(&str, &str); 6] = [
    ("src/lib.rs", "declares the module"),
    ("src/main.rs", "turns the measurements on at the start and writes them at the exit"),
    ("src/app/mod.rs", "the field of the app"),
    ("src/app/count.rs", "the one place where the actions of the keymap are counted and timed"),
    ("src/app/files.rs", "the tick that writes the measurements"),
    ("src/tui/mod.rs", "the loop: the read of an input event and the end of the draw"),
];

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
        .filter(|f| std::fs::read_to_string(root.join(f)).unwrap().contains("responsiveness"))
        .collect();
    let other: Vec<&String> = users.iter().filter(|f| !HOOKS.iter().any(|(h, _)| h == f)).collect();
    assert!(other.is_empty(), "these files measure time outside the one place (RESP-2); measure in App::did:\n{other:#?}");
    let count = std::fs::read_to_string(root.join("src/app/count.rs")).unwrap();
    assert_eq!(count.matches("self.responsiveness.ran(").count(), 1, "one call, in the hook that every action passes");
}
