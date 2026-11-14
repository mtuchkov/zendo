use super::*;

#[test]
fn origins_from_every_url_form_match() {
    for u in ["git@github.com:Mtuchkov/zendo.git", "https://github.com/mtuchkov/zendo", "ssh://git@github.com/mtuchkov/zendo.git", "https://github.com/mtuchkov/zendo.git/"] {
        assert_eq!(normalize_origin(u), "github.com/mtuchkov/zendo", "{u}");
    }
}

#[test]
fn location_is_machine_then_path_from_home() {
    let repo = Path::new("/n");
    let home = Some(Path::new("/Users/m"));
    assert_eq!(location_dir(repo, "mbp", Path::new("/Users/m/src/zendo"), home), Path::new("/n/mbp/src/zendo"));
    assert_eq!(location_dir(repo, "mbp", Path::new("/Users/m"), home), Path::new("/n/mbp/~"));
    assert_eq!(location_dir(repo, "mbp", Path::new("/opt/work"), home), Path::new("/n/mbp/_/opt/work"));
    assert_eq!(clean_name("Mikhails-MacBook-Pro.local"), "mikhails-macbook-pro");
    assert_eq!(clean_name("  "), "machine");
}

// NOTESREP-1: a home behind a symlink (as /var → /private/var on macOS) is still the home, so a
// repository under it keeps its folder of this machine and root.
#[cfg(unix)]
#[test]
fn a_home_behind_a_symlink_is_still_the_home() {
    let base = std::env::temp_dir().join(format!("zendo-sync-link-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("real/src/proj")).unwrap();
    std::os::unix::fs::symlink(base.join("real"), base.join("home")).unwrap();
    let root = base.join("real/src/proj").canonicalize().unwrap();
    let got = location_dir(Path::new("/n"), "m1", &root, Some(&base.join("home")));
    assert_eq!(got, Path::new("/n/m1/src/proj"));
    let _ = std::fs::remove_dir_all(&base);
}

fn sh(dir: &Path, args: &[&str]) {
    let ok = Command::new("git").arg("-C").arg(dir).args(args).stdout(Stdio::null()).stderr(Stdio::null()).status().unwrap().success();
    assert!(ok, "git {args:?}");
}

/// A bare remote and two clones: the notes repo on two machines.
fn two_machines(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let base = std::env::temp_dir().join(format!("zendo-sync-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    sh(&base, &["init", "-q", "--bare", "-b", "main", "remote.git"]);
    for m in ["a", "b"] {
        sh(&base, &["clone", "-q", "remote.git", m]);
        sh(&base.join(m), &["config", "user.email", "t@example.com"]);
        sh(&base.join(m), &["config", "user.name", "t"]);
        sh(&base.join(m), &["checkout", "-q", "-B", "main"]);
    }
    (base.clone(), base.join("a"), base.join("b"))
}

#[test]
fn two_machines_sync_their_own_folders() {
    let (base, a, b) = two_machines("ok");
    let (da, db) = (a.join("a/src/x"), b.join("b/src/x"));
    for (repo, dir, m) in [(&a, &da, "a"), (&b, &db, "b")] {
        prepare(dir, &Location { machine: m.into(), path: "/h/src/x".into(), origin: Some("github.com/o/x".into()) }).unwrap();
        std::fs::write(dir.join("idea.md"), format!("# idea from {m}\n")).unwrap();
        assert_eq!(commit(repo, dir, "notes"), Ok(true));
        assert_eq!(commit(repo, dir, "notes"), Ok(false), "nothing new");
        assert_eq!(pull_push(repo), State::Synced);
    }
    assert_eq!(pull_push(&a), State::Synced);
    assert!(a.join("b/src/x/idea.md").exists(), "a got the note of b");
    let locs = locations(&a);
    assert_eq!(locs.len(), 2);
    assert!(locs.iter().all(|(_, l)| l.origin.as_deref() == Some("github.com/o/x")));
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn a_new_empty_notes_repo_is_in_sync() {
    let (base, a, _) = two_machines("empty");
    assert_eq!(pull_push(&a), State::Synced);
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn a_conflict_stops_and_keeps_the_files() {
    let (base, a, b) = two_machines("conflict");
    for (repo, text) in [(&a, "from a\n"), (&b, "from b\n")] {
        std::fs::create_dir_all(repo.join("m/~")).unwrap();
        std::fs::write(repo.join("m/~/same.md"), text).unwrap();
        commit(repo, &repo.join("m"), "notes").unwrap();
    }
    assert_eq!(pull_push(&a), State::Synced);
    assert!(matches!(pull_push(&b), State::Conflict(_)));
    assert_eq!(std::fs::read_to_string(b.join("m/~/same.md")).unwrap(), "from b\n");
    assert!(!b.join(".git/rebase-merge").exists() && !b.join(".git/rebase-apply").exists());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn syncer_commits_on_its_own_and_at_exit() {
    let (base, a, _) = two_machines("syncer");
    let dir = a.join("a/~");
    prepare(&dir, &Location { machine: "a".into(), path: "/h".into(), origin: None }).unwrap();
    let s = Syncer::start(a.clone(), dir.clone(), "a", Duration::from_millis(300));
    std::fs::write(dir.join("one.md"), "# one\n").unwrap();
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(s.state(), State::Synced);
    std::fs::write(dir.join("two.md"), "# two\n").unwrap();
    assert_eq!(s.finish(), State::Synced);
    let log = git(&base.join("remote.git"), &["log", "--format=%s", "main"], Duration::from_secs(5)).unwrap();
    assert!(log.lines().count() >= 2 && log.lines().all(|l| l == "notes: a ~"), "{log}");
    let _ = std::fs::remove_dir_all(&base);
}
