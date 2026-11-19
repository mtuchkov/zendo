use super::*;
use crate::keys::Pane;
use std::collections::BTreeSet;

/// Nothing runs in the background.
fn idle() -> BTreeSet<Work> {
    BTreeSet::new()
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn works<const N: usize>(w: [Work; N]) -> BTreeSet<Work> {
    BTreeSet::from(w)
}

/// The answer to question `n`.
fn answer(n: usize) -> Work {
    Work::Answer(n)
}

/// The events that are ready: the pane, the action and the wait in milliseconds.
fn ready(r: &mut Responsiveness) -> Vec<(Pane, &'static str, Option<u128>)> {
    r.take().into_iter().map(|e| (e.pane, e.action, e.wait.map(|w| w.as_millis()))).collect()
}

/// One run of `name` of `pane` that took `n` ms from the input to the draw, with nothing in the
/// background.
fn run(r: &mut Responsiveness, pane: Pane, name: &str, t0: Instant, n: u64) {
    r.input(t0, &idle());
    r.ran(pane, name, Trigger::Key);
    r.drawn(t0 + ms(n), &idle());
}

// RESP-1: each action gets the time from its input to the draw of its screen.
#[test]
fn a_run_gets_the_time_from_its_input_to_its_draw() {
    let mut r = Responsiveness::on();
    let t0 = Instant::now();
    run(&mut r, Pane::Read, "add_to_chat", t0, 10);
    run(&mut r, Pane::Global, "toggle_tree", t0, 300);
    assert_eq!(ready(&mut r), [(Pane::Read, "add_to_chat", Some(10)), (Pane::Global, "toggle_tree", Some(300))]);
    assert!(ready(&mut r).is_empty(), "each event comes once");
}

// USAGE-7: the event has the keymap pane and the trigger of the action; the same name in another
// pane is another action.
#[test]
fn a_count_has_the_pane_the_action_and_the_trigger() {
    let mut r = Responsiveness::on();
    let t0 = Instant::now();
    r.input(t0, &idle());
    r.ran(Pane::Read, "add_to_chat", Trigger::Mouse);
    r.ran(Pane::Write, "add_to_chat", Trigger::Key);
    r.drawn(t0 + ms(4), &idle());
    let e: Vec<(Pane, Trigger)> = r.take().into_iter().map(|e| (e.pane, e.trigger)).collect();
    assert_eq!(e, [(Pane::Read, Trigger::Mouse), (Pane::Write, Trigger::Key)]);
}

// USAGE-8: a name that is not an action of the keymap gets no event; RESP-1: an action without an
// input gets its event at once, with no time.
#[test]
fn only_actions_of_the_keymap_count() {
    let mut r = Responsiveness::on();
    let t0 = Instant::now();
    run(&mut r, Pane::Read, "no_such_action", t0, 5);
    assert!(ready(&mut r).is_empty(), "not an action of the keymap");
    r.ran(Pane::Read, "copy", Trigger::Key);
    assert_eq!(ready(&mut r), [(Pane::Read, "copy", None)], "no input, no time");
}

// RESP-1: an action that starts work in the background waits for the end of that work: the time
// ends at the first draw after it, not at the first screen.
#[test]
fn an_action_waits_for_the_end_of_its_work() {
    let mut r = Responsiveness::on();
    let t0 = Instant::now();
    let search = works([Work::Search(1)]);
    r.drawn(t0, &idle());
    r.input(t0, &idle());
    r.ran(Pane::Global, "find", Trigger::Key);
    r.drawn(t0 + ms(5), &search);
    r.drawn(t0 + ms(300), &search);
    assert!(ready(&mut r).is_empty(), "the search still runs");
    r.drawn(t0 + ms(700), &idle());
    assert_eq!(ready(&mut r), [(Pane::Global, "find", Some(700))]);
}

// RESP-1: work that ran before the key, such as a turn of the agent, does not hold the action.
#[test]
fn work_that_ran_before_the_key_does_not_hold_it() {
    let mut r = Responsiveness::on();
    let t0 = Instant::now();
    let turn = works([answer(1)]);
    r.drawn(t0, &turn);
    r.input(t0 + ms(10), &turn);
    r.ran(Pane::Global, "toggle_tree", Trigger::Key);
    r.drawn(t0 + ms(12), &turn);
    assert_eq!(ready(&mut r), [(Pane::Global, "toggle_tree", Some(2))], "2 ms, not the turn");
}

// RESP-1: a question that waits in the queue of a busy thread waits for its own answer; the
// question before it waits only for the answer before.
#[test]
fn a_queued_question_waits_for_its_own_answer() {
    let mut r = Responsiveness::on();
    let t0 = Instant::now();
    r.drawn(t0, &idle());
    r.input(t0, &idle());
    r.ran(Pane::Chat, "send", Trigger::Key);
    r.drawn(t0 + ms(5), &works([answer(1)]));
    r.input(t0 + ms(100), &works([answer(1)]));
    r.ran(Pane::Chat, "send", Trigger::Key);
    r.drawn(t0 + ms(105), &works([answer(1), answer(2)]));
    // The first turn ends and the queued question goes to the agent in the same event.
    r.drawn(t0 + ms(1000), &works([answer(2)]));
    assert_eq!(ready(&mut r), [(Pane::Chat, "send", Some(1000))], "1 s, not both answers");
    r.drawn(t0 + ms(3100), &idle());
    assert_eq!(ready(&mut r), [(Pane::Chat, "send", Some(3000))], "the queued question waited 3 s for its answer");
}

// RESP-1: two inputs before one draw: each action waits only for the work that its own input
// started.
#[test]
fn each_input_of_one_frame_waits_for_its_own_work() {
    let mut r = Responsiveness::on();
    let t0 = Instant::now();
    r.drawn(t0, &idle());
    r.input(t0, &idle());
    r.ran(Pane::Global, "toggle_tree", Trigger::Key);
    r.input(t0 + ms(1), &idle());
    r.ran(Pane::Chat, "send", Trigger::Key);
    r.drawn(t0 + ms(3), &works([answer(1)]));
    assert_eq!(ready(&mut r), [(Pane::Global, "toggle_tree", Some(3))], "the tree did not wait for the answer; the question waits");
}

// RESP-1: a new search of the same panel ends the wait for the old one: the typing of the user
// after the action is not in its time.
#[test]
fn a_new_search_ends_the_wait_for_the_old_one() {
    let mut r = Responsiveness::on();
    let t0 = Instant::now();
    r.drawn(t0, &idle());
    r.input(t0, &idle());
    r.ran(Pane::Global, "find", Trigger::Key);
    r.drawn(t0 + ms(5), &works([Work::Search(1)]));
    // The user types: no action of the keymap, and a new search replaces the old one.
    r.input(t0 + ms(40), &works([Work::Search(1)]));
    r.drawn(t0 + ms(45), &works([Work::Search(2)]));
    assert_eq!(ready(&mut r), [(Pane::Global, "find", Some(45))]);
}

// RESP-9: a result that does not come within 512 seconds: the event comes with no time, once.
#[test]
fn a_result_that_does_not_come_has_no_time() {
    let mut r = Responsiveness::on();
    let t0 = Instant::now();
    let turn = works([answer(1)]);
    r.drawn(t0, &idle());
    r.input(t0, &idle());
    r.ran(Pane::Chat, "send", Trigger::Key);
    r.drawn(t0 + ms(5), &turn);
    r.drawn(t0 + Duration::from_secs(600), &turn);
    assert_eq!(ready(&mut r), [(Pane::Chat, "send", None)]);
    r.drawn(t0 + Duration::from_secs(700), &idle());
    assert!(ready(&mut r).is_empty(), "counted once");
}

// RESP-9: an action whose result has not come at the exit gets its event with no time (User,
// 2026-11-19), so that its count stays.
#[test]
fn an_action_without_its_result_at_the_exit_has_no_time() {
    let mut r = Responsiveness::on();
    let t0 = Instant::now();
    r.drawn(t0, &idle());
    r.input(t0, &idle());
    r.ran(Pane::Chat, "send", Trigger::Key);
    r.drawn(t0 + ms(5), &works([answer(1)]));
    r.input(t0 + ms(9), &works([answer(1)]));
    r.ran(Pane::Read, "copy", Trigger::Key);
    r.end();
    assert_eq!(ready(&mut r), [(Pane::Chat, "send", None), (Pane::Read, "copy", None)]);
}

// TELEM-12: off (`stats = false`): no event.
#[test]
fn off_measures_nothing() {
    let mut r = Responsiveness::default();
    run(&mut r, Pane::Read, "copy", Instant::now(), 2);
    r.ran(Pane::Read, "copy", Trigger::Key);
    r.end();
    assert!(ready(&mut r).is_empty());
}
