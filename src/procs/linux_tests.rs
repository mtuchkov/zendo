use super::*;
use std::fs;
use std::path::PathBuf;

/// A fake `/proc` in a folder of its own for `name`, with the leftovers of an earlier run removed.
fn fake_proc(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("zendo-procs-linux-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::write(mk(&base, "uptime"), "1000.50 4000.00\n").unwrap();
    base
}

fn mk(base: &std::path::Path, rel: &str) -> PathBuf {
    let p = base.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    p
}

/// A process: its id, parent, group, state and start (clock ticks after boot).
struct P(u32, u32, u32, char, u64);

/// The process `p` with its command line and children.
fn process(base: &std::path::Path, P(pid, ppid, pgid, state, start): P, args: &[&str], children: &[u32]) {
    let rest: Vec<String> = (0..16).map(|_| "0".to_string()).collect();
    let stat = format!("{pid} (a (b) c) {state} {ppid} {pgid} {} {start} 0\n", rest.join(" "));
    fs::write(mk(base, &format!("{pid}/stat")), stat).unwrap();
    fs::write(mk(base, &format!("{pid}/cmdline")), args.iter().map(|a| format!("{a}\0")).collect::<String>()).unwrap();
    let kids: Vec<String> = children.iter().map(u32::to_string).collect();
    fs::write(mk(base, &format!("{pid}/task/{pid}/children")), kids.join(" ")).unwrap();
}

// BG-2, ADR 0016: zendo reads only its own tree: the root and the processes under it, with the
// group, the age and the command line of each; not the other processes of the computer.
#[test]
fn the_tree_of_zendo_comes_from_proc() {
    let base = fake_proc("tree");
    process(&base, P(100, 1, 100, 'S', 50_000), &["zendo"], &[200]);
    process(&base, P(200, 100, 200, 'S', 60_000), &["node", "claude-agent-acp"], &[230]);
    process(&base, P(230, 200, 230, 'S', 90_050), &["/bin/bash", "-c", "npm run dev"], &[]);
    process(&base, P(999, 1, 999, 'S', 1_000), &["another", "program"], &[]);
    let mut t = tree(&base, 100).unwrap();
    t.sort_by_key(|p| p.pid);
    assert_eq!(t.iter().map(|p| (p.pid, p.ppid, p.pgid)).collect::<Vec<_>>(), vec![(100, 1, 100), (200, 100, 200), (230, 200, 230)]);
    assert_eq!((t[2].secs, t[2].args.as_str()), (100, "/bin/bash -c npm run dev"));
    assert_eq!(t[1].secs, 400, "the whole seconds of the exact age: 1000.50 - 600.00");
    let _ = fs::remove_dir_all(&base);
}

// BG-2: a zombie runs no more and a process that ended while zendo reads does not count.
#[test]
fn a_zombie_and_an_ended_process_do_not_count() {
    let base = fake_proc("zombie");
    process(&base, P(100, 1, 100, 'S', 0), &["zendo"], &[200, 300]);
    process(&base, P(200, 100, 200, 'Z', 0), &[], &[]);
    let t = tree(&base, 100).unwrap();
    assert_eq!(t.iter().map(|p| p.pid).collect::<Vec<_>>(), vec![100]);
    assert!(zombie(&base, 200) && !zombie(&base, 100) && !zombie(&base, 300));
    let _ = fs::remove_dir_all(&base);
}

// ADR 0016: a kernel without the children files gives no tree, so zendo uses `ps`.
#[test]
fn no_children_files_give_no_tree() {
    let base = fake_proc("nochildren");
    process(&base, P(100, 1, 100, 'S', 0), &["zendo"], &[]);
    fs::remove_file(base.join("100/task/100/children")).unwrap();
    assert_eq!(tree(&base, 100), None);
    let _ = fs::remove_dir_all(&base);
}
