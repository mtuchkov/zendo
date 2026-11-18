use super::*;

fn fresh(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("zendo-lock-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    base
}

// NOTESREP-15: one sync at a time; the holder writes its pid, and the lock goes when it drops.
#[test]
fn a_second_taker_waits_until_the_first_lets_go() {
    let base = fresh("two");
    let path = base.join("sub/notes-sync.lock");
    let first = SyncLock::try_take(&path, "/h/src/a").expect("the first takes the lock");
    assert!(SyncLock::try_take(&path, "/h/src/b").is_none(), "the lock is busy");
    let who = holder(&path).expect("someone holds the lock");
    assert!(who.contains(&format!("pid {}", std::process::id())) && who.contains("/h/src/a"), "{who}");
    drop(first);
    assert_eq!(holder(&path), None, "nobody holds the lock");
    assert!(SyncLock::try_take(&path, "/h/src/b").is_some(), "the second takes the lock now");
    let _ = std::fs::remove_dir_all(&base);
}

// NOTESREP-15: a lock that frees up in the wait is taken; a lock that stays busy is not.
#[test]
fn take_within_waits_a_bounded_time() {
    let base = fresh("within");
    let path = base.join("notes-sync.lock");
    let first = SyncLock::try_take(&path, "a").unwrap();
    let start = Instant::now();
    assert!(SyncLock::take_within(&path, "b", Duration::from_millis(200)).is_none());
    assert!(start.elapsed() < Duration::from_secs(2), "{:?}", start.elapsed());
    let t = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        drop(first);
    });
    assert!(SyncLock::take_within(&path, "b", Duration::from_secs(5)).is_some());
    t.join().unwrap();
    let _ = std::fs::remove_dir_all(&base);
}

// NOTESREP-15: a lock file that zendo cannot make does not stop the sync.
#[test]
fn a_lock_file_that_cannot_open_does_not_stop_the_sync() {
    let base = fresh("noopen");
    std::fs::write(base.join("file"), "").unwrap();
    let path = base.join("file/notes-sync.lock");
    assert!(SyncLock::try_take(&path, "a").is_some());
    assert_eq!(holder(&path), None);
    let _ = std::fs::remove_dir_all(&base);
}

// NOTESREP-18: an index.lock older than a minute, while no zendo syncs, is stale.
#[test]
fn an_old_index_lock_without_a_sync_is_stale() {
    let base = fresh("stale");
    let (repo, path) = (base.join("repo"), base.join("notes-sync.lock"));
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    assert!(!stale_index_lock(&repo, &path), "no index.lock");
    let index = repo.join(".git/index.lock");
    std::fs::write(&index, "").unwrap();
    assert!(!stale_index_lock(&repo, &path), "a new index.lock is git at work");
    let old = std::time::SystemTime::now() - Duration::from_secs(120);
    std::fs::File::options().write(true).open(&index).unwrap().set_modified(old).unwrap();
    assert!(stale_index_lock(&repo, &path));
    let held = SyncLock::try_take(&path, "a").unwrap();
    assert!(!stale_index_lock(&repo, &path), "a zendo syncs now");
    drop(held);
    let _ = std::fs::remove_dir_all(&base);
}
