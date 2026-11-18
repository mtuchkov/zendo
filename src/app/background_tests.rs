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
