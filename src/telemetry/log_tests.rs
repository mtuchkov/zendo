use super::*;

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-telemetry-log-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn files(d: &Path) -> Vec<(String, u64)> {
    let mut f: Vec<(String, u64)> = std::fs::read_dir(d).unwrap().flatten().map(|e| (e.file_name().to_string_lossy().into_owned(), e.metadata().unwrap().len())).collect();
    f.sort();
    f
}

// TELEM-3, TELEM-4: a file grows to at most 1 MB, then a new one comes; at most 50 files stay, of
// all processes together, and the oldest go first.
#[test]
fn the_log_keeps_at_most_50_files_of_1_mb() {
    let d = dir("roll");
    std::fs::create_dir_all(&d).unwrap();
    for i in 0..FILES {
        std::fs::write(d.join(format!("20000101T000000.000Z-9-{i:02}.jsonl")), "{}\n").unwrap();
    }
    let mut roll = Roll::new(d.clone(), 1, Box::new(append));
    let line = format!("{{\"x\":\"{}\"}}", "a".repeat(990));
    let len = line.len();
    assert!(roll.write(vec![line; 1100]), "the write worked");
    let all = files(&d);
    assert_eq!(all.len(), FILES, "{:?}", all.iter().map(|f| &f.0).collect::<Vec<_>>());
    assert!(all.iter().all(|(_, n)| *n <= MAX_FILE), "no file over 1 MB");
    assert!(!all.iter().any(|(n, _)| n.ends_with("-9-00.jsonl") || n.ends_with("-9-01.jsonl")), "the two oldest went");
    assert_eq!(all.iter().filter(|(n, _)| n.contains("-1-")).count(), 2, "two new files of this process");
    let text: String = all.iter().filter(|(n, _)| n.contains("-1-")).map(|(n, _)| std::fs::read_to_string(d.join(n)).unwrap()).collect();
    assert_eq!((text.lines().count(), text.lines().all(|l| l.len() == len)), (1100, true), "all lines, whole");
    let _ = std::fs::remove_dir_all(&d);
}

// TELEM-11: the lines of a failed write stay for the next one, at most 1 MB of them, the newest.
#[test]
fn the_kept_lines_stay_under_1_mb() {
    let d = dir("keep");
    let mut roll = Roll::new(d.clone(), 1, Box::new(|_: &Path, _: &str| Err(std::io::Error::other("full"))));
    let line = "b".repeat(999);
    assert!(!roll.write(vec![line.clone(); 1500]), "the write failed");
    assert!(roll.kept() <= MAX_FILE, "{} bytes kept", roll.kept());
    assert!(roll.kept() > MAX_FILE - 2_000, "the newest lines stay");
    let _ = std::fs::remove_dir_all(&d);
}
