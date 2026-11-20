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

fn out(q: &str, n: usize) -> Outgoing {
    Outgoing { question: q.into(), text: q.into(), attach: Vec::new(), sent: None, command: false, n, fresh: false }
}

// RESP-1: the questions that wait for an answer: the one of the turn, then the queue, each with its
// own number, which stays when a queued question goes to the agent.
#[test]
fn each_question_without_an_answer_has_its_own_number() {
    let mut t = Thread::default();
    assert_eq!(t.waiting().collect::<Vec<_>>(), Vec::<usize>::new());
    (t.busy, t.turn) = (Some(Instant::now()), Some(1));
    t.queue = vec![out("b", 2), out("c", 3)];
    assert_eq!(t.waiting().collect::<Vec<_>>(), [1, 2, 3]);
    // The turn ends; the next question goes to the agent.
    t.turn = Some(t.queue.remove(0).n);
    assert_eq!(t.waiting().collect::<Vec<_>>(), [2, 3]);
    // The agent exits: the turn ends, the queue still waits.
    t.busy = None;
    assert_eq!(t.waiting().collect::<Vec<_>>(), [3]);
}

/// A question of the queue: `/new` with or without text, or a plain question.
fn queued(text: &str, fresh: bool, sent: Option<&str>) -> Outgoing {
    Outgoing { question: text.into(), text: text.into(), attach: Vec::new(), sent: sent.map(str::to_string), command: false, n: 0, fresh }
}

// HISTORY-10: the next question goes to the thread as the queue leaves it: behind `/new` with no
// text it is the first question of a new thread, and it knows only the files that went behind
// that `/new`.
#[test]
fn the_next_question_sees_the_thread_behind_a_queued_new() {
    let mut t = Thread { primed: true, ..Thread::default() };
    t.files.push(Some("a.md".into()));
    assert!(t.primed_for_next() && t.sent_lately_for_next("a.md"), "no `/new`: this thread");
    t.queue.push(queued("one more", false, None));
    assert!(t.primed_for_next() && t.sent_lately_for_next("a.md"), "a question in the queue: this thread");
    t.queue.push(queued("", true, None));
    assert!(!t.primed_for_next() && !t.sent_lately_for_next("a.md"), "behind `/new`: a new thread");
    t.queue.push(queued("first", false, Some("b.md")));
    assert!(t.primed_for_next() && t.sent_lately_for_next("b.md") && !t.sent_lately_for_next("a.md"), "behind its first question");
    t.queue.push(queued("a task", true, None));
    assert!(t.primed_for_next() && !t.sent_lately_for_next("b.md"), "`/new <text>` starts the next thread itself");
}

// ATTN-5, HISTORY-9: `/new` with no text is no question that waits.
#[test]
fn new_alone_is_no_question_that_waits() {
    let mut t = Thread::default();
    assert!(!t.question_waits());
    t.queue.push(queued("", true, None));
    assert!(!t.question_waits(), "`/new` alone sends nothing");
    t.queue.push(queued("next", false, None));
    assert!(t.question_waits());
}
