use super::*;

/// A folder of one test, empty: what an interrupted earlier run left is removed.
fn data(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-stats-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

// USAGE-13, RESP-7: a write replaces the file whole, and the folder comes when it is missing.
#[test]
fn a_write_replaces_the_file_whole() {
    let dir = data("whole");
    let file = dir.join("usage/a.toml");
    write(&file, "old = 1\n").unwrap();
    write(&file, "new = 2\n").unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "new = 2\n");
    let names: Vec<_> = std::fs::read_dir(dir.join("usage")).unwrap().flatten().map(|e| e.file_name()).collect();
    assert_eq!(names, ["a.toml"], "no temporary file stays");
    let _ = std::fs::remove_dir_all(&dir);
}

// USAGE-13, RESP-7: a write that fails leaves no temporary file.
#[test]
fn a_failed_write_leaves_no_temporary_file() {
    let dir = data("failed");
    let file = dir.join("a.toml");
    std::fs::create_dir_all(file.join("inside")).unwrap();
    assert!(write(&file, "x = 1\n").is_err(), "a folder is where the file goes");
    let names: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name()).collect();
    assert_eq!(names, ["a.toml"], "only the folder: {names:?}");
    let _ = std::fs::remove_dir_all(&dir);
}
