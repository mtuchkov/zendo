use super::repo_root;
use std::path::Path;

fn scratch(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-root-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("a/b")).unwrap();
    d
}

// README: in a repo, zendo uses the repo root.
#[test]
fn repo_root_finds_the_nearest_git_folder() {
    let d = scratch("git");
    std::fs::create_dir(d.join("a/.git")).unwrap();
    assert_eq!(repo_root(&d.join("a/b"), Some(Path::new("/home/u"))), d.join("a"));
    let _ = std::fs::remove_dir_all(&d);
}

// README: outside a repo, zendo uses the home folder as a scratchpad.
#[test]
fn repo_root_outside_a_repo_is_the_home_folder() {
    let d = scratch("nogit");
    let start = d.join("a/b");
    if repo_root(&start, None) == start {
        assert_eq!(repo_root(&start, Some(Path::new("/home/u"))), Path::new("/home/u"));
    }
    let _ = std::fs::remove_dir_all(&d);
}
