use super::*;
use std::time::Duration;

fn tools(started: Instant) -> Item {
    Item::Tools { uses: Vec::new(), started, took: None }
}

fn took(it: &Item) -> Option<Duration> {
    match it {
        Item::Tools { took, .. } => *took,
        _ => None,
    }
}

// CLAUDE-33: a tool line shows how long its run of tools took; the time stops when the agent moves on,
// not at the end of the turn.
#[test]
fn the_time_of_a_tool_run_stops_when_the_next_item_comes() {
    let mut t = Thread::default();
    t.push(tools(Instant::now() - Duration::from_secs(5)));
    t.push(Item::Agent("next".into()));
    let frozen = took(&t.items[0]).expect("the time of the tool run stops");
    assert!(frozen >= Duration::from_secs(5) && frozen < Duration::from_secs(6), "{frozen:?}");
}

// CLAUDE-33: an earlier, stopped time does not change.
#[test]
fn a_stopped_time_does_not_change() {
    let mut t = Thread::default();
    t.items.push(Item::Tools { uses: Vec::new(), started: Instant::now(), took: Some(Duration::from_secs(2)) });
    t.push(Item::Agent("next".into()));
    assert_eq!(took(&t.items[0]), Some(Duration::from_secs(2)));
}

// CLAUDE-33: the run of tools that is the last item still runs.
#[test]
fn the_last_tool_run_still_runs() {
    let mut t = Thread::default();
    t.push(Item::Agent("a".into()));
    t.push(tools(Instant::now()));
    assert_eq!(took(&t.items[1]), None);
}
