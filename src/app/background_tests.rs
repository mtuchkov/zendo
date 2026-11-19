use super::*;

// BG-1, BG-7: the time of a process, as a line says it.
#[test]
fn the_time_of_a_process_reads_short() {
    assert_eq!(ran(Duration::from_secs(40)), "40 s");
    assert_eq!(ran(Duration::from_secs(14 * 60 + 5)), "14 min");
}

// BG-1: a command that a step runs now is not in the background; after the step it is.
#[test]
fn a_running_step_holds_its_command() {
    let now = Instant::now();
    let mut t = Thread::default();
    let u = ToolUse { id: "b".into(), kind: "execute".into(), state: crate::agent::ToolState::Running, started: Some(now), ..Default::default() };
    t.push(Item::Tools { uses: vec![u], started: now, took: None });
    assert_eq!(exec_step(&t, now + Duration::from_secs(1), true), Some(now));
    if let Some(Item::Tools { uses, .. }) = t.items.last_mut() {
        uses[0].state = crate::agent::ToolState::Done;
    }
    assert_eq!(exec_step(&t, now + Duration::from_secs(1), true), None);
    assert_eq!(exec_step(&t, now + Duration::from_secs(1), false), Some(now), "the owner still finds the step");
}

// BG-13, ADR 0023: zendo reads its process tree only while a command step runs, for `SCAN_AFTER`
// after the start of the last command step, or while a background process is in the line.
#[test]
fn the_tree_is_read_only_around_commands() {
    let now = Instant::now();
    let idle = Thread::default();
    assert!(!scan_wanted([&idle], true, now), "no command, no background process");
    assert!(scan_wanted([&idle], false, now), "a background process in the line");
    let mut t = Thread::default();
    let u = ToolUse { id: "b".into(), kind: "execute".into(), state: crate::agent::ToolState::Done, started: Some(now), ..Default::default() };
    t.push(Item::Tools { uses: vec![u], started: now, took: None });
    assert!(scan_wanted([&t], true, now + Duration::from_secs(1)), "just after a command");
    assert!(!scan_wanted([&t], true, now + SCAN_AFTER + Duration::from_secs(1)), "long after it");
    if let Some(Item::Tools { uses, .. }) = t.items.last_mut() {
        uses[0].state = crate::agent::ToolState::Running;
    }
    assert!(scan_wanted([&t], true, now + SCAN_AFTER + Duration::from_secs(60)), "a command that runs");
}
