use super::super::tests::{fresh, other};
use super::*;
use crate::board::How;
use serde_json::json;

fn run(c: &Ctx, tool: &str, a: Value) -> Out {
    call(c, tool, &a).expect("a board tool")
}

fn id(v: Out) -> String {
    v.unwrap()["id"].as_str().unwrap().to_string()
}

fn pending(c: &Ctx, r: usize) -> Vec<String> {
    c.journal.state().pending(&c.repo, r, &|_| true).iter().map(|m| m.text.clone()).collect()
}

// BOARD-1, BOARD-2: a task to an inbox waits for that session, and goes to it one time.
#[test]
fn a_task_reaches_the_inbox_once() {
    let c3 = fresh("inbox", 3);
    let m = id(run(&c3, "assign", json!({ "to": "session:2", "text": "rebase feat/x" })));
    assert_eq!(pending(&c3, 2), ["rebase feat/x"]);
    assert!(pending(&c3, 4).is_empty(), "only the inbox of 2");
    assert!(c3.journal.claim(&m, 2, How::Delivered));
    assert!(!c3.journal.claim(&m, 2, How::Watched), "a second claim loses");
    assert!(pending(&c3, 2).is_empty());
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-3: `all` reaches only the sessions whose agent runs, never the sender.
#[test]
fn all_skips_sessions_without_an_agent() {
    let c3 = fresh("all", 3);
    run(&c3, "ask", json!({ "to": "all", "text": "who owns port 5220?" })).unwrap();
    let st = c3.journal.state();
    assert_eq!(st.recipients(&st.msgs[0].1, &|n| n == 2 || n == 3), [2]);
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-4, BOARD-7: a channel of another session is not found for a session outside it.
#[test]
fn a_foreign_channel_is_not_found() {
    let c3 = fresh("foreign", 3);
    run(&c3, "ask", json!({ "to": "session:2", "text": "q", "reply_to": "review" })).unwrap();
    let c4 = other(&c3, 4);
    assert_eq!(run(&c4, "ask", json!({ "to": "3/review", "text": "hi" })).unwrap_err(), "not found");
    assert_eq!(run(&c4, "invite", json!({ "channel": "3/review", "session": 1 })).unwrap_err(), "not found");
    assert_eq!(run(&c4, "leave", json!({ "channel": "3/review" })).unwrap_err(), "not found");
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-8, BOARD-7: `reply_to` with a channel of the sender takes the recipient in; the channel
// closes with the thread of its owner.
#[test]
fn reply_to_invites_and_the_channel_closes_with_its_owner() {
    let c3 = fresh("invite", 3);
    let m = id(run(&c3, "ask", json!({ "to": "session:2", "text": "q", "reply_to": "oci" })));
    let c2 = other(&c3, 2);
    assert!(c3.journal.claim(&m, 2, How::Delivered));
    run(&c2, "ask", json!({ "to": "oci", "text": "to the channel" })).unwrap();
    assert_eq!(c3.journal.state().channels_of("/r", 2), ["3/oci"]);
    c3.journal.append(&[Event::ThreadEnd { repo: "/r".into(), session: 3 }]).unwrap();
    assert_eq!(run(&c2, "ask", json!({ "to": "3/oci", "text": "again" })).unwrap_err(), "not found");
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-5: one answer to a question; no response to a response.
#[test]
fn no_response_to_a_response() {
    let c3 = fresh("response", 3);
    let c2 = other(&c3, 2);
    let q = id(run(&c3, "ask", json!({ "to": "session:2", "text": "which commit?" })));
    assert_eq!(run(&c2, "answer", json!({ "in_reply_to": q, "text": "a1" })).unwrap_err(), "not found", "2 did not take it yet");
    c3.journal.claim(&q, 2, How::Delivered);
    let a = id(run(&c2, "answer", json!({ "in_reply_to": q, "text": "235403111a" })));
    assert!(run(&c2, "answer", json!({ "in_reply_to": q, "text": "again" })).is_err(), "one answer");
    assert_eq!(pending(&c3, 3), ["235403111a"], "the answer goes to the inbox of the sender");
    c3.journal.claim(&a, 3, How::Delivered);
    assert!(run(&c3, "answer", json!({ "in_reply_to": a, "text": "thanks" })).is_err());
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-5: a task goes accepted or rejected, then done or failed, each once.
#[test]
fn a_task_status_follows_its_order() {
    let c3 = fresh("status", 3);
    let c2 = other(&c3, 2);
    let t = id(run(&c3, "assign", json!({ "to": "session:2", "text": "fix" })));
    c3.journal.claim(&t, 2, How::Delivered);
    let st = |s: &str| run(&c2, "status", json!({ "in_reply_to": t, "status": s }));
    assert!(st("done").is_err(), "accepted first");
    assert!(st("accepted").is_ok());
    assert!(st("accepted").is_err(), "once");
    assert!(st("done").is_ok());
    assert!(st("failed").is_err(), "done is the last");
    assert!(run(&c2, "answer", json!({ "in_reply_to": t, "text": "x" })).is_err(), "a task gets a status");
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-6, BOARD-9: the waiting questions of a sender go together; while they are in flight, the
// next one waits, and it goes when they land.
#[test]
fn initiations_wait_while_one_is_in_flight() {
    let c3 = fresh("held", 3);
    let one = id(run(&c3, "ask", json!({ "to": "session:2", "text": "one" })));
    let two = id(run(&c3, "ask", json!({ "to": "session:2", "text": "two" })));
    assert_eq!(pending(&c3, 2), ["one", "two"], "zendo squashes these into one question");
    c3.journal.claim(&one, 2, How::Delivered);
    c3.journal.claim(&two, 2, How::Delivered);
    run(&c3, "ask", json!({ "to": "session:2", "text": "three" })).unwrap();
    assert!(pending(&c3, 2).is_empty(), "held while the first ones are in flight");
    assert_eq!(c3.journal.state().taken("/r", 2), [one.clone(), two.clone()]);
    c3.journal.land(&[one], 2);
    assert!(pending(&c3, 2).is_empty(), "two is still in flight");
    c3.journal.land(&[two], 2);
    assert_eq!(pending(&c3, 2), ["three"]);
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-11, BOARD-36: no message to itself, to a session that does not exist, and no task to a
// channel.
#[test]
fn bad_targets_are_refused() {
    let c3 = fresh("targets", 3);
    assert!(run(&c3, "ask", json!({ "to": "session:3", "text": "me" })).is_err());
    assert_eq!(run(&c3, "ask", json!({ "to": "session:9", "text": "x" })).unwrap_err(), "not found");
    assert!(run(&c3, "assign", json!({ "to": "all", "text": "x" })).unwrap_err().contains("one session"));
    assert!(c3.journal.state().msgs.is_empty());
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-10, BOARD-12: `assign` without a target waits for the choice of the user; `esc` cancels.
#[test]
fn assign_without_a_target_waits_for_the_choice() {
    let c3 = fresh("choice", 3);
    let chooser = |to: Option<usize>| {
        let j = c3.journal.clone();
        std::thread::spawn(move || {
            for _ in 0..50 {
                std::thread::sleep(std::time::Duration::from_millis(20));
                let st = j.state();
                if let Some(m) = st.msgs.iter().map(|(_, m)| m).rfind(|m| m.to.is_none() && st.choice(&m.id).is_none()) {
                    j.append(&[Event::Choice { msg: m.id.clone(), to }]).unwrap();
                    return;
                }
            }
        })
    };
    let h = chooser(Some(4));
    let v = run(&c3, "assign", json!({ "text": "rebase" })).unwrap();
    h.join().unwrap();
    assert_eq!(v["to"], "session:4");
    assert_eq!(pending(&c3, 4), ["rebase"]);
    let h = chooser(None);
    assert!(run(&c3, "assign", json!({ "text": "again" })).unwrap_err().contains("did not choose"));
    h.join().unwrap();
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-26: an initiation returns its id and the command that waits for its response.
#[test]
fn an_initiation_returns_the_wait_command() {
    let c3 = fresh("wait", 3);
    let v = run(&c3, "ask", json!({ "to": "session:2", "text": "q" })).unwrap();
    let id = v["id"].as_str().unwrap();
    assert_eq!(v["wait"], format!("zendo board watch --session 3 --for {id}"));
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}
