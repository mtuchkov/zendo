use super::*;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The writes that a disk got: the file, the content and the sync.
type Got = Arc<Mutex<Vec<(PathBuf, Vec<u8>, Sync)>>>;

/// A disk that holds each write until the test lets it go, and keeps what it got.
struct Gate {
    got: Got,
    go: mpsc::Sender<()>,
}

fn gated() -> (Writer, Gate) {
    let got: Got = Arc::new(Mutex::new(Vec::new()));
    let (go, wait) = mpsc::channel::<()>();
    let wait = Mutex::new(wait);
    let seen = got.clone();
    let w = Writer::with(move |p: &Path, c: &[u8], s: Sync| {
        let _ = wait.lock().unwrap().recv();
        seen.lock().unwrap().push((p.to_path_buf(), c.to_vec(), s));
        Ok(())
    });
    (w, Gate { got, go })
}

fn settle(w: &Writer) {
    let end = Instant::now() + Duration::from_secs(5);
    while !w.idle() {
        assert!(Instant::now() < end, "the writer did not finish");
        std::thread::sleep(Duration::from_millis(2));
    }
}

// IO-1, IO-2: a write returns at once while the disk holds the write before it.
#[test]
fn a_write_returns_at_once_while_the_disk_waits() {
    let (w, gate) = gated();
    w.write(Path::new("a"), b"1".to_vec(), Sync::Own);
    std::thread::sleep(Duration::from_millis(20));
    let t = Instant::now();
    for i in 0..100 {
        w.write(Path::new("b"), vec![i], Sync::Own);
    }
    assert!(t.elapsed() < Duration::from_millis(10), "the writes waited: {:?}", t.elapsed());
    for _ in 0..2 {
        gate.go.send(()).unwrap();
    }
    settle(&w);
}

// IO-4: while a write of a file runs, only the newest content of that file waits.
#[test]
fn only_the_newest_content_waits_while_a_write_runs() {
    let (w, gate) = gated();
    w.write(Path::new("beat"), b"1".to_vec(), Sync::Own);
    std::thread::sleep(Duration::from_millis(20));
    w.write(Path::new("beat"), b"2".to_vec(), Sync::Own);
    w.write(Path::new("beat"), b"3".to_vec(), Sync::Own);
    for _ in 0..2 {
        gate.go.send(()).unwrap();
    }
    settle(&w);
    let got: Vec<Vec<u8>> = gate.got.lock().unwrap().iter().map(|(_, c, _)| c.clone()).collect();
    assert_eq!(got, [b"1".to_vec(), b"3".to_vec()]);
}

// IO-3: the writer keeps the sync of each write as the caller asks.
#[test]
fn each_write_keeps_its_sync() {
    let (w, gate) = gated();
    w.write(Path::new("note.md"), b"text".to_vec(), Sync::User);
    w.write(Path::new("beat"), b"1".to_vec(), Sync::Own);
    for _ in 0..2 {
        gate.go.send(()).unwrap();
    }
    settle(&w);
    let mut got: Vec<(PathBuf, Sync)> = gate.got.lock().unwrap().iter().map(|(p, _, s)| (p.clone(), *s)).collect();
    got.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(got, [(PathBuf::from("beat"), Sync::Own), (PathBuf::from("note.md"), Sync::User)]);
}

// IO-8: appends to a file while a write of it runs are all kept, in order.
#[test]
fn appends_while_a_write_runs_are_kept_in_order() {
    let (w, gate) = gated();
    w.append(Path::new("log"), b"a\n".to_vec());
    std::thread::sleep(Duration::from_millis(20));
    w.append(Path::new("log"), b"b\n".to_vec());
    w.append(Path::new("log"), b"c\n".to_vec());
    for _ in 0..2 {
        gate.go.send(()).unwrap();
    }
    settle(&w);
    let got: Vec<Vec<u8>> = gate.got.lock().unwrap().iter().map(|(_, c, _)| c.clone()).collect();
    assert_eq!(got, [b"a\n".to_vec(), b"b\nc\n".to_vec()]);
    assert!(gate.got.lock().unwrap().iter().all(|(_, _, s)| *s == Sync::Append));
}

// FAIL-1: a failed write comes back once, with its file.
#[test]
fn a_failed_write_comes_back_once() {
    let mut w = Writer::with(|_: &Path, _: &[u8], _: Sync| Err(std::io::Error::other("disk full")));
    w.write(Path::new("db.json"), b"{}".to_vec(), Sync::Own);
    settle(&w);
    let f = w.failures();
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].0, PathBuf::from("db.json"));
    assert!(f[0].1.contains("disk full"), "{}", f[0].1);
    assert!(w.failures().is_empty(), "once");
}

// IO-3: the real disk gets the content of an own file and of user data, and the end of a log.
#[test]
fn the_disk_gets_each_kind_of_write() {
    let dir = std::env::temp_dir().join(format!("zendo-writer-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let w = Writer::new();
    w.write(&dir.join("own.json"), b"{}".to_vec(), Sync::Own);
    w.write(&dir.join("note.md"), b"# A\n".to_vec(), Sync::User);
    w.append(&dir.join("log"), b"one\n".to_vec());
    settle(&w);
    w.append(&dir.join("log"), b"two\n".to_vec());
    settle(&w);
    assert_eq!(std::fs::read_to_string(dir.join("own.json")).unwrap(), "{}");
    assert_eq!(std::fs::read_to_string(dir.join("note.md")).unwrap(), "# A\n");
    assert_eq!(std::fs::read_to_string(dir.join("log")).unwrap(), "one\ntwo\n");
    let _ = std::fs::remove_dir_all(&dir);
}

// UNDO-6, IO-1: a checked write writes only when the disk has what zendo saw last; else it answers
// with the text of the disk and writes nothing. A new file is written only when it is not there.
#[test]
fn a_checked_write_does_not_write_over_a_change_from_outside() {
    let dir = std::env::temp_dir().join(format!("zendo-writer-checked-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let note = dir.join("note.md");
    std::fs::write(&note, "# A\n").unwrap();
    let mut w = Writer::new();
    let first = w.write_checked(&note, b"# A\n\nmine\n".to_vec(), Some("# A".into()));
    settle(&w);
    assert_eq!(w.checked(), [(first, Checked::Written)]);
    std::fs::write(&note, "# A\n\ntheirs\n").unwrap();
    let second = w.write_checked(&note, b"# A\n\nmine again\n".to_vec(), Some("# A\n\nmine".into()));
    settle(&w);
    assert_eq!(w.checked(), [(second, Checked::Changed("# A\n\ntheirs".into()))]);
    assert_eq!(std::fs::read_to_string(&note).unwrap(), "# A\n\ntheirs\n", "nothing written over");
    let taken = w.write_checked(&note, b"# B\n".to_vec(), None);
    settle(&w);
    assert_eq!(w.checked(), [(taken, Checked::Taken)]);
    let _ = std::fs::remove_dir_all(&dir);
}
