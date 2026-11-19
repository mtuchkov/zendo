//! Telemetry log (docs/zendo/telemetry.md): each action of the keymap becomes one event of the log
//! in the data folder, and nothing sends the log out of the machine.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::Path;
use std::time::{Duration, Instant};
use zendo::responsiveness::Responsiveness;
use zendo::telemetry::Telemetry;

const NOTE: &str = "docs/notes/2026-11-19-a.md";

// TELEM-1: an action of the keymap becomes one event, with its pane, its name, its trigger and the
// time that the user waited for its result.
#[tokio::test]
async fn an_action_becomes_one_event() {
    let root = repo("telemetry-event", &[(NOTE, "# A\n\nBefore the table there is text.\n")]);
    let data = std::env::temp_dir().join(format!("zendo-telemetry-data-event-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    let mut a = app(&root);
    a.keys = zendo::keys::Keymap::defaults();
    (a.telemetry, a.responsiveness) = (Telemetry::start(&data, std::process::id()), Responsiveness::on());
    draw(&mut a, 140, 30);
    let t0 = Instant::now();
    a.drawn_at(t0);
    a.input_at(t0);
    a.on_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT));
    draw(&mut a, 140, 30);
    a.drawn_at(t0 + Duration::from_millis(10));
    a.end_telemetry();
    let events = zendo::telemetry::read(&data);
    assert_eq!(events.len(), 1, "{events:?}");
    let e = &events[0];
    assert_eq!((e["pane"].as_str(), e["action"].as_str(), e["trigger"].as_str(), e["wait_ms"].as_u64()), (Some("read"), Some("add_to_chat"), Some("key"), Some(10)), "{e}");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&data);
}

/// The calls that open a connection to another machine.
const SENDS: [&str; 6] = ["TcpStream", "UdpSocket", "reqwest", "ureq", "hyper", "http://"];

fn sources(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

// TELEM-13: until the user agrees to share the statistics, nothing sends the log: the code of the
// log opens no connection, and the log stays in the data folder.
#[test]
fn nothing_leaves_the_machine_without_the_agreement() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    sources(&root.join("src/telemetry"), &mut files);
    assert!(!files.is_empty(), "the code of the log");
    let sends: Vec<String> = files
        .iter()
        .flat_map(|f| {
            let text = std::fs::read_to_string(f).unwrap();
            SENDS.iter().filter(|w| text.contains(*w)).map(|w| format!("{}: {w}", f.display())).collect::<Vec<_>>()
        })
        .collect();
    assert!(sends.is_empty(), "the log goes out only after the agreement of the user (TELEM-13):\n{sends:#?}");
}
