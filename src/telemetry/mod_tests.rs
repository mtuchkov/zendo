use super::*;
use crate::keys::Pane;
use std::sync::mpsc;

/// A data folder of one test, empty: what an interrupted earlier run left is removed.
fn data(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-telemetry-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// An event of `action` of the read pane, with a wait of `ms`.
fn event(action: &'static str, ms: Option<u64>) -> Event {
    Event { at: Utc::now(), pane: Pane::Read, action, trigger: Trigger::Key, wait: ms.map(Duration::from_millis) }
}

fn names(dir: &Path) -> Vec<String> {
    let mut n: Vec<String> = std::fs::read_dir(dir).map(|d| d.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect()).unwrap_or_default();
    n.sort();
    n
}

// TELEM-2: the log goes to `telemetry/` of the data folder, and nowhere else.
#[test]
fn the_log_stays_in_the_data_folder() {
    let dir = data("local");
    let mut t = Telemetry::start(&dir, 7);
    t.record(event("copy", Some(3)));
    t.record(event("add_to_chat", Some(12)));
    t.finish();
    assert_eq!(names(&dir), ["telemetry"], "nothing else in the data folder");
    assert_eq!(t.dir(), Some(dir.join("telemetry").as_path()));
    assert_eq!(read(&dir).len(), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

// TELEM-5: the events stay in memory and a thread of their own writes them: a write that does not
// return does not delay the next event, the tick or the exit.
#[test]
fn a_slow_write_does_not_delay_the_screen() {
    let (hold_tx, hold) = mpsc::channel::<()>();
    let mut t = Telemetry::writing(&data("slow"), 1, move |_, _| {
        let _ = hold.recv();
        Ok(())
    });
    let t0 = Instant::now();
    let begin = Instant::now();
    t.record(event("copy", Some(2)));
    t.tick(t0 + EVERY);
    t.record(event("copy", Some(2)));
    assert_eq!(t.waiting().len(), 1, "the next event is in memory");
    t.tick(t0 + EVERY * 2);
    t.finish();
    assert!(begin.elapsed() < Duration::from_secs(3), "the writes did not hold the events: {:?}", begin.elapsed());
    drop(hold_tx);
}

// TELEM-6, TELEM-7: one line of JSON for each event, with the time, the version, the system and the
// process; nothing of the user; no time for an event without one.
#[test]
fn an_event_is_one_line_of_json() {
    let line = event("add_to_chat", Some(12)).line(4711);
    assert!(!line.contains('\n'), "{line}");
    let v: serde_json::Value = serde_json::from_str(&line).expect("JSON");
    let mut keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["action", "at", "os", "pane", "pid", "trigger", "wait_ms", "zendo"], "{line}");
    assert_eq!((v["pane"].as_str(), v["action"].as_str(), v["trigger"].as_str(), v["wait_ms"].as_u64(), v["pid"].as_u64()), (Some("read"), Some("add_to_chat"), Some("key"), Some(12), Some(4711)));
    assert_eq!(v["zendo"].as_str(), Some(env!("CARGO_PKG_VERSION")));
    assert!(!line.contains('/') && !line.contains(&std::env::var("USER").unwrap_or_else(|_| "\u{0}".into())), "no path and no user name: {line}");
    let none: serde_json::Value = serde_json::from_str(&event("send", None).line(1)).unwrap();
    assert!(none.get("wait_ms").is_none(), "no time: {none}");
}

// TELEM-8: the events reach the disk every 15 seconds; no event, no write.
#[test]
fn the_events_reach_the_disk_within_15_seconds() {
    let dir = data("tick");
    let mut t = Telemetry::start(&dir, 1);
    let t0 = Instant::now();
    t.record(event("copy", Some(2)));
    t.tick(t0 + Duration::from_secs(5));
    assert_eq!(t.waiting().len(), 1, "not before 15 seconds");
    t.tick(t0 + Duration::from_secs(16));
    assert!(t.waiting().is_empty(), "the events went to the writer");
    t.finish();
    assert_eq!(read(&dir).len(), 1);
    t.tick(t0 + Duration::from_secs(40));
    t.finish();
    assert_eq!(names(&dir.join("telemetry")).len(), 1, "no event, no new file");
    let _ = std::fs::remove_dir_all(&dir);
}

// TELEM-9: the exit writes the waiting events.
#[test]
fn the_exit_writes_the_events() {
    let dir = data("exit");
    let mut t = Telemetry::start(&dir, 1);
    t.record(event("open", None));
    t.finish();
    assert_eq!(read(&dir).len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

// TELEM-10: each process writes only files of its own.
#[test]
fn two_processes_write_two_files() {
    let dir = data("two");
    let (mut a, mut b) = (Telemetry::start(&dir, 11), Telemetry::start(&dir, 12));
    a.record(event("copy", Some(1)));
    b.record(event("copy", Some(1)));
    a.finish();
    b.finish();
    let files = names(&dir.join("telemetry"));
    assert_eq!(files.len(), 2, "{files:?}");
    for f in &files {
        let pid: u64 = f.split('-').nth(1).and_then(|p| p.parse().ok()).expect("the process id in the name");
        let text = std::fs::read_to_string(dir.join("telemetry").join(f)).unwrap();
        let v: serde_json::Value = serde_json::from_str(text.lines().next().unwrap()).unwrap();
        assert_eq!(v["pid"].as_u64(), Some(pid), "{f}: the events of its process");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// TELEM-11: a write that fails says nothing and keeps the events for the next write.
#[test]
fn a_failed_write_keeps_the_events() {
    let dir = data("fail");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("telemetry"), "a file where the folder goes").unwrap();
    let mut t = Telemetry::start(&dir, 1);
    let t0 = Instant::now();
    t.record(event("copy", Some(2)));
    t.tick(t0 + EVERY);
    t.finish();
    assert!(read(&dir).is_empty(), "the write failed");
    std::fs::remove_file(dir.join("telemetry")).unwrap();
    t.record(event("copy", Some(3)));
    t.finish();
    assert_eq!(read(&dir).len(), 2, "the next write has the kept events");
    let _ = std::fs::remove_dir_all(&dir);
}

// TELEM-12: off (`stats = false`): no event and no file.
#[test]
fn off_records_nothing_and_writes_nothing() {
    let mut t = Telemetry::default();
    t.record(event("copy", Some(2)));
    t.tick(Instant::now() + EVERY);
    t.finish();
    assert!(t.waiting().is_empty());
    assert!(t.dir().is_none());
}

// TELEM-14: the folders of the summaries of earlier versions go at the start.
#[test]
fn the_old_summaries_go_at_the_start() {
    let dir = data("old");
    for f in ["usage/2026-11-14-1.toml", "responsiveness/2026-11-19-1.toml", "fonts-named.txt"] {
        std::fs::create_dir_all(dir.join(f).parent().unwrap()).unwrap();
        std::fs::write(dir.join(f), "x").unwrap();
    }
    let _t = Telemetry::start(&dir, 1);
    assert_eq!(names(&dir), ["fonts-named.txt"], "only the summaries go");
    let _ = std::fs::remove_dir_all(&dir);
}
