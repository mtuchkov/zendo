use super::*;
use crate::keys::Pane;
use std::time::Duration;

/// A data folder of one test, empty: what an interrupted earlier run left is removed.
fn data(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-usage-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn files(dir: &Path) -> Vec<PathBuf> {
    let mut all: Vec<PathBuf> = std::fs::read_dir(dir.join("usage")).map(|d| d.flatten().map(|e| e.path()).collect()).unwrap_or_default();
    all.sort();
    all
}

fn read(usage: &Usage) -> toml::Table {
    toml::from_str(&std::fs::read_to_string(usage.file().expect("a usage file")).expect("the file is there")).expect("the file parses")
}

// USAGE-1, USAGE-7: a count has the keymap pane, the action name and the trigger.
#[test]
fn a_count_has_the_pane_the_action_and_the_trigger() {
    let mut u = Usage::start(&data("count"), 1);
    u.count(Pane::Read, "add_to_chat", Trigger::Key);
    u.count(Pane::Read, "add_to_chat", Trigger::Key);
    u.count(Pane::Read, "add_to_chat", Trigger::Mouse);
    u.count(Pane::Global, "next_session", Trigger::Key);
    assert_eq!(u.get(Pane::Read, "add_to_chat", Trigger::Key), 2);
    assert_eq!(u.get(Pane::Read, "add_to_chat", Trigger::Mouse), 1);
    assert_eq!(u.get(Pane::Global, "next_session", Trigger::Key), 1);
    assert_eq!(u.get(Pane::Write, "add_to_chat", Trigger::Key), 0, "the same name in another pane is another count");
}

// USAGE-8: what is not an action of the keymap is not counted.
#[test]
fn only_actions_of_the_keymap_count() {
    let mut u = Usage::start(&data("unknown"), 1);
    u.count(Pane::Read, "no_such_action", Trigger::Key);
    u.count(Pane::Delta, "copy", Trigger::Mouse);
    u.write();
    assert!(files(&data_of(&u)).is_empty(), "nothing counted, no file");
}

fn data_of(u: &Usage) -> PathBuf {
    u.file().and_then(|f| f.parent()).and_then(Path::parent).expect("the data folder").to_path_buf()
}

// USAGE-2, USAGE-4, USAGE-10: each process writes a file of its own, in `usage/` of the data folder.
#[test]
fn the_usage_file_is_local_and_of_one_process() {
    let dir = data("files");
    let (mut a, mut b) = (Usage::start(&dir, 11), Usage::start(&dir, 12));
    a.count(Pane::Read, "copy", Trigger::Key);
    b.count(Pane::Write, "paste", Trigger::Key);
    a.write();
    b.write();
    let all = files(&dir);
    assert_eq!(all.len(), 2, "{all:?}");
    let day = chrono::Local::now().format("%Y-%m-%d");
    assert_eq!(a.file().unwrap(), dir.join(format!("usage/{day}-11.toml")));
    assert_eq!(read(&a)["read"]["copy"]["key"].as_integer(), Some(1));
    assert!(!read(&a).contains_key("write"), "only the counts of this process");
    assert_eq!(read(&b)["write"]["paste"]["key"].as_integer(), Some(1));
    let top: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name()).collect();
    assert_eq!(top, ["usage"], "nothing else in the data folder");
    let _ = std::fs::remove_dir_all(&dir);
}

// USAGE-3: the counts reach the disk within 15 seconds; no change, no write.
#[test]
fn the_counts_reach_the_disk_within_15_seconds() {
    let dir = data("tick");
    let mut u = Usage::start(&dir, 1);
    let t0 = Instant::now();
    u.count(Pane::Read, "copy", Trigger::Key);
    u.tick(t0 + Duration::from_secs(5));
    assert!(files(&dir).is_empty(), "not before 15 seconds");
    u.tick(t0 + Duration::from_secs(16));
    assert_eq!(read(&u)["read"]["copy"]["key"].as_integer(), Some(1));
    u.count(Pane::Read, "copy", Trigger::Key);
    u.tick(t0 + Duration::from_secs(20));
    assert_eq!(read(&u)["read"]["copy"]["key"].as_integer(), Some(1), "15 seconds after the last write");
    u.tick(t0 + Duration::from_secs(32));
    assert_eq!(read(&u)["read"]["copy"]["key"].as_integer(), Some(2));
    std::fs::remove_file(u.file().unwrap()).unwrap();
    u.tick(t0 + Duration::from_secs(60));
    assert!(files(&dir).is_empty(), "no change of the counts, no write");
    let _ = std::fs::remove_dir_all(&dir);
}

// USAGE-12: the exit writes the counts at once.
#[test]
fn the_exit_writes_the_counts() {
    let dir = data("exit");
    let mut u = Usage::start(&dir, 1);
    u.count(Pane::Tree, "open", Trigger::Key);
    u.write();
    assert_eq!(read(&u)["tree"]["open"]["key"].as_integer(), Some(1));
    let _ = std::fs::remove_dir_all(&dir);
}

// USAGE-5: a write that fails changes nothing: the counts stay, and the next write comes 15 seconds later.
#[test]
fn a_failed_write_keeps_the_counts() {
    let dir = data("fail");
    // A file where the folder must be: the folder cannot be made.
    std::fs::write(&dir, "").unwrap();
    let mut u = Usage::start(&dir, 1);
    let t0 = Instant::now();
    u.count(Pane::Read, "copy", Trigger::Key);
    u.tick(t0 + Duration::from_secs(16));
    assert_eq!(u.get(Pane::Read, "copy", Trigger::Key), 1);
    std::fs::remove_file(&dir).unwrap();
    u.tick(t0 + Duration::from_secs(20));
    assert!(files(&dir).is_empty(), "the next try is 15 seconds after the last");
    u.tick(t0 + Duration::from_secs(32));
    assert_eq!(read(&u)["read"]["copy"]["key"].as_integer(), Some(1));
    let _ = std::fs::remove_dir_all(&dir);
}

// USAGE-9, USAGE-11, USAGE-13: the head of the file, nothing of the user in it, no temporary file left.
#[test]
fn the_file_has_a_head_and_only_counts() {
    let dir = data("head");
    let mut u = Usage::start(&dir, 1);
    u.count(Pane::Selected, "cut", Trigger::Key);
    u.write();
    let t = read(&u);
    assert_eq!(t["zendo"].as_str(), Some(env!("CARGO_PKG_VERSION")));
    assert_eq!(t["os"].as_str(), Some(std::env::consts::OS));
    let time = |k: &str| chrono::DateTime::parse_from_rfc3339(t[k].as_str().unwrap()).unwrap_or_else(|e| panic!("{k}: {e}"));
    assert!(time("started") <= time("written"));
    let keys: Vec<&str> = t.keys().map(String::as_str).collect();
    assert_eq!(keys, ["os", "selection", "started", "written", "zendo"], "the head and the panes with counts");
    assert_eq!(t["selection"].as_table().unwrap().len(), 1);
    assert_eq!(files(&dir).len(), 1, "no temporary file stays");
    let _ = std::fs::remove_dir_all(&dir);
}

// USAGE-14: with `usage = false`, nothing is counted and nothing is written.
#[test]
fn off_counts_nothing_and_writes_nothing() {
    let mut u = Usage::for_process(false);
    u.count(Pane::Read, "copy", Trigger::Key);
    u.tick(Instant::now() + Duration::from_secs(60));
    u.write();
    assert_eq!(u.get(Pane::Read, "copy", Trigger::Key), 0);
    assert_eq!(u.file(), None);
    assert_eq!(Usage::default().file(), None, "an app in a test counts nothing until the test turns it on");
}

// USAGE-14: `usage = false` in the config; without the key, the counting is on.
#[test]
fn the_config_turns_the_counting_off() {
    use crate::setup::config::UserConfig;
    let on: UserConfig = toml::from_str("").unwrap();
    let off: UserConfig = toml::from_str("usage = false").unwrap();
    assert!(on.usage.unwrap_or(true));
    assert_eq!(off.usage, Some(false));
}
