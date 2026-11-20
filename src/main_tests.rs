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

// ENV-7: the start has a message for the problems of the keys, for each problem of the agent
// environment, and for the notes outside the notes repo; zendo shows them one after another.
#[test]
fn the_start_has_a_message_for_each_problem() {
    let said = super::start_notices(2, &["/r/.envrc: zendo skipped line 5".to_string()], 3);
    assert_eq!(said.len(), 3, "{said:?}");
    assert!(said[0].contains("[keys] 2 problem(s)") && said[1].contains("skipped line 5") && said[2].contains("3 note(s)"), "{said:?}");
    assert!(super::start_notices(0, &[], 0).is_empty());
}
