use super::*;
use crate::agent::Attachment;
use crate::app::{App, Outgoing, session_key};
use std::path::PathBuf;
use std::time::{Duration, Instant};

// CLAUDE-45: the drafts of the sessions go into the state and come back from it, with the chips,
// their contexts and their files.
#[test]
fn drafts_go_to_the_state_and_come_back() {
    let mut p = ChatPane::default();
    p.composer_mut(2).input_mut().type_str("why ");
    p.composer_mut(2).add_chip("sel".into(), Some("> text".into()), None, false);
    p.composer_mut(2).add_chip("otes/b.md".into(), None, Some(Attachment::File("/r/notes/b.md".into())), false);
    p.composer_mut(2).set_context("Changes".into());
    let drafts = p.drafts();
    assert_eq!(drafts.keys().copied().collect::<Vec<_>>(), [2], "only a session with something in it");
    let mut q = ChatPane::default();
    q.load(&drafts);
    let c = q.composer(2);
    assert_eq!(c.input().text(), "why ⟦sel⟧ ⟦otes/b.md⟧ ");
    assert_eq!(c.chips()[0].context.as_deref(), Some("> text"));
    assert!(matches!(&c.chips()[1].attach, Some(Attachment::File(f)) if f.ends_with("b.md")));
    assert!(c.has_context());
    assert_eq!(q.drafts(), drafts, "a load and a save give the same drafts");
}

// ADR 0011: the drafts go to the state 2 s after the last change, so a crash loses at most 2 s.
#[test]
fn the_drafts_are_due_two_seconds_after_the_last_change() {
    let mut p = ChatPane::default();
    let t0 = Instant::now();
    assert!(!p.due(t0), "nothing changed");
    p.composer_mut(0).input_mut().type_str("a");
    assert!(!p.due(t0));
    p.composer_mut(0).input_mut().type_str("b");
    assert!(!p.due(t0 + Duration::from_secs(1)), "a change starts the 2 s again");
    assert!(p.due(t0 + Duration::from_secs(3)));
    p.mark_saved();
    assert!(!p.due(t0 + Duration::from_secs(9)), "saved");
}

// CLAUDE-47: the queue of a thread goes back into the input of its session, and the name of a
// pasted image labels its chip in the session where the user pasted it, while another session is
// active.
#[test]
fn background_changes_go_to_the_input_of_their_session() {
    let mut a = App::default();
    let img = PathBuf::from("/tmp/zendo-test-paste.png");
    a.chat.composer_mut(0).add_chip("Image #1".into(), None, Some(Attachment::Image(img.clone())), false);
    a.thread(&session_key(0)).queue.push(Outgoing { question: "queued".into(), text: String::new(), attach: Vec::new(), sent: None, command: false });
    a.state.session = 1;
    a.queue_to_input(&session_key(0));
    a.label_image(&img, "red car");
    assert_eq!(a.composer().input().text(), "", "the active session does not change");
    let first = a.chat.composer(0).input().text();
    assert!(first.contains("queued") && first.contains("⟦Image: red car⟧"), "{first}");
}
