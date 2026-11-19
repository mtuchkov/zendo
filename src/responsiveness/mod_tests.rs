use super::*;
use crate::keys::Pane;
use std::collections::BTreeSet;
use std::sync::mpsc;

/// A data folder of one test, empty: what an interrupted earlier run left is removed.
fn data(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-responsiveness-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn read(r: &Responsiveness) -> toml::Table {
    toml::from_str(&std::fs::read_to_string(r.file().expect("a file")).expect("the file is there")).expect("the file parses")
}

/// Nothing runs in the background.
fn idle() -> BTreeSet<Work> {
    BTreeSet::new()
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// One run of `name` of `pane` that took `ms` from the input to the draw.
fn run(r: &mut Responsiveness, pane: Pane, name: &str, t0: Instant, n: u64) {
    r.input(t0);
    r.ran(pane, name);
    r.drawn(t0 + ms(n), &idle());
}

// RESP-1, RESP-5: each action gets the time from its input to the draw, in a bucket of its pane
// and name.
#[test]
fn a_run_goes_into_the_bucket_of_its_time() {
    let mut r = Responsiveness::start(&data("bucket"), 1);
    let t0 = Instant::now();
    run(&mut r, Pane::Read, "add_to_chat", t0, 10);
    run(&mut r, Pane::Read, "add_to_chat", t0, 300);
    run(&mut r, Pane::Global, "toggle_tree", t0, 1);
    let t = r.times(Pane::Read, "add_to_chat").expect("times");
    assert_eq!(t.runs(), 2);
    assert_eq!(t.within(16) - t.within(8), 1, "10 ms is in the bucket up to 16 ms");
    assert_eq!(t.within(512) - t.within(256), 1, "300 ms is in the bucket up to 512 ms");
    assert_eq!(r.times(Pane::Global, "toggle_tree").expect("times").within(1), 1);
    assert!(r.times(Pane::Write, "add_to_chat").is_none(), "the same name in another pane is another entry");
}

// RESP-1: an action without an input, and a name that is not an action of the keymap, get no time.
#[test]
fn only_actions_after_an_input_get_a_time() {
    let mut r = Responsiveness::start(&data("input"), 1);
    let t0 = Instant::now();
    r.ran(Pane::Read, "copy");
    r.drawn(t0, &idle());
    run(&mut r, Pane::Read, "no_such_action", t0, 5);
    assert!(r.times(Pane::Read, "copy").is_none(), "no input, no time");
    assert!(r.times(Pane::Read, "no_such_action").is_none());
    // The draw ends the input: the next action without a new input gets no time.
    run(&mut r, Pane::Read, "copy", t0, 5);
    r.ran(Pane::Read, "copy");
    r.drawn(t0 + ms(900), &idle());
    assert_eq!(r.times(Pane::Read, "copy").expect("times").runs(), 1);
}

// RESP-1: an action that starts work in the background waits for the end of that work: the time
// ends at the first draw after it, not at the first screen.
#[test]
fn an_action_waits_for_the_end_of_its_work() {
    let mut r = Responsiveness::start(&data("wait"), 1);
    let t0 = Instant::now();
    let search = BTreeSet::from([Work::Find]);
    r.drawn(t0, &idle());
    r.input(t0);
    r.ran(Pane::Global, "find");
    r.drawn(t0 + ms(5), &search);
    r.drawn(t0 + ms(300), &search);
    assert!(r.times(Pane::Global, "find").is_none(), "the search still runs");
    r.drawn(t0 + ms(700), &idle());
    let t = r.times(Pane::Global, "find").expect("the time after the search");
    assert_eq!((t.runs(), t.within(1000) - t.within(512)), (1, 1), "700 ms: the bucket up to 1 s");
}

// RESP-1: work that ran before the key, such as a turn of the agent, does not hold the action.
#[test]
fn work_that_ran_before_the_key_does_not_hold_it() {
    let mut r = Responsiveness::start(&data("before"), 1);
    let t0 = Instant::now();
    let turn = BTreeSet::from([Work::Turn("chat".into())]);
    r.drawn(t0, &turn);
    r.input(t0 + ms(10));
    r.ran(Pane::Global, "toggle_tree");
    r.drawn(t0 + ms(12), &turn);
    assert_eq!(r.times(Pane::Global, "toggle_tree").expect("times").within(2), 1, "2 ms, not the turn");
}

// RESP-1: a result that does not come goes into the last bucket, and the action waits no more.
#[test]
fn a_result_that_does_not_come_goes_into_the_last_bucket() {
    let mut r = Responsiveness::start(&data("never"), 1);
    let t0 = Instant::now();
    let turn = BTreeSet::from([Work::Turn("chat".into())]);
    r.drawn(t0, &idle());
    r.input(t0);
    r.ran(Pane::Chat, "send");
    r.drawn(t0 + ms(5), &turn);
    r.drawn(t0 + Duration::from_secs(600), &turn);
    let t = r.times(Pane::Chat, "send").expect("the time of the last bucket");
    assert_eq!((t.runs(), t.within(512_000)), (1, 0), "over the last bound");
    r.drawn(t0 + Duration::from_secs(700), &idle());
    assert_eq!(r.times(Pane::Chat, "send").expect("times").runs(), 1, "counted once");
}

// RESP-3: the measurements go to a file of this process in the data folder, and nowhere else.
#[test]
fn the_measurements_stay_in_the_data_folder() {
    let dir = data("local");
    let mut r = Responsiveness::start(&dir, 7);
    run(&mut r, Pane::Read, "copy", Instant::now(), 3);
    r.finish();
    let day = chrono::Local::now().format("%Y-%m-%d");
    assert_eq!(r.file().unwrap(), dir.join(format!("responsiveness/{day}-7.toml")));
    assert_eq!(read(&r)["read"]["copy"]["runs"].as_integer(), Some(1));
    let top: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name()).collect();
    assert_eq!(top, ["responsiveness"], "nothing else in the data folder");
    let inner: Vec<_> = std::fs::read_dir(dir.join("responsiveness")).unwrap().flatten().map(|e| e.file_name()).collect();
    assert_eq!(inner.len(), 1, "no temporary file stays: {inner:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

// RESP-4: a write that does not return does not delay the next action, the draw or the tick.
#[test]
fn a_slow_write_does_not_delay_the_screen() {
    let (hold_tx, hold) = mpsc::channel::<()>();
    let mut r = Responsiveness::writing(&data("slow"), 1, move |_, _| {
        let _ = hold.recv();
        Ok(())
    });
    let t0 = Instant::now();
    let begin = Instant::now();
    run(&mut r, Pane::Read, "copy", t0, 2);
    r.tick(t0 + EVERY);
    run(&mut r, Pane::Read, "copy", t0, 2);
    r.tick(t0 + EVERY * 2);
    r.finish();
    assert!(begin.elapsed() < Duration::from_secs(3), "the writes did not hold the actions: {:?}", begin.elapsed());
    assert_eq!(r.times(Pane::Read, "copy").expect("times").runs(), 2, "the next action works");
    drop(hold_tx);
}

// The write every 15 seconds while the measurements changed, as for the counts (USAGE-3); no
// change, no write.
#[test]
fn the_measurements_reach_the_disk_within_15_seconds() {
    let dir = data("tick");
    let mut r = Responsiveness::start(&dir, 1);
    let t0 = Instant::now();
    run(&mut r, Pane::Read, "copy", t0, 2);
    r.tick(t0 + EVERY);
    r.finish();
    assert_eq!(read(&r)["read"]["copy"]["runs"].as_integer(), Some(1));
    std::fs::remove_file(r.file().unwrap()).unwrap();
    r.tick(t0 + EVERY * 2);
    r.finish();
    assert!(!r.file().unwrap().exists(), "no change, no write");
    let _ = std::fs::remove_dir_all(&dir);
}

// RESP-6: the file holds the head and the buckets, no text, no path and no time of one run.
#[test]
fn the_file_has_a_head_and_only_buckets() {
    let mut r = Responsiveness::start(&data("text"), 1);
    run(&mut r, Pane::Read, "add_to_chat", Instant::now(), 10);
    let text = r.text(chrono::Utc::now());
    let t: toml::Table = toml::from_str(&text).expect("the file parses");
    assert_eq!(t["zendo"].as_str(), Some(env!("CARGO_PKG_VERSION")));
    for key in ["os", "started", "written"] {
        assert!(t.contains_key(key), "{key}: {text}");
    }
    let entry = t["read"]["add_to_chat"].as_table().expect("an entry");
    let keys: Vec<&str> = entry.keys().map(String::as_str).collect();
    assert_eq!(keys, ["runs", "upto_16ms"], "{text}");
    assert!(!text.contains('/') && !text.contains("10ms"), "no path and no time of one run: {text}");
}

// RESP-7: a write that fails says nothing; the measurements stay for the next write.
#[test]
fn a_failed_write_keeps_the_measurements() {
    let dir = data("fail");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("responsiveness"), "a file where the folder goes").unwrap();
    let mut r = Responsiveness::start(&dir, 1);
    let t0 = Instant::now();
    run(&mut r, Pane::Read, "copy", t0, 2);
    r.tick(t0 + EVERY);
    r.finish();
    assert_eq!(r.times(Pane::Read, "copy").expect("times").runs(), 1);
    std::fs::remove_file(dir.join("responsiveness")).unwrap();
    run(&mut r, Pane::Read, "copy", t0, 2);
    r.finish();
    assert_eq!(read(&r)["read"]["copy"]["runs"].as_integer(), Some(2), "the next write has all runs");
    let _ = std::fs::remove_dir_all(&dir);
}

// Off (`usage = false`): no time and no file.
#[test]
fn off_measures_nothing_and_writes_nothing() {
    let mut r = Responsiveness::default();
    run(&mut r, Pane::Read, "copy", Instant::now(), 2);
    r.finish();
    assert!(r.file().is_none());
    assert!(r.times(Pane::Read, "copy").is_none());
}
