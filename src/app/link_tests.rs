//! The link without an app and without an agent process (ADR 0008).
use super::*;

// ADR 0008: before the agent starts, the link has no handle, and a command goes nowhere.
#[test]
fn a_link_without_an_agent_sends_nothing() {
    let mut l = AgentLink::with_name_model(Some("light".into()));
    assert!(!l.running() && !l.ready() && l.handle().is_none() && l.events_mut().is_none());
    l.send(Cmd::Cancel { key: "k".into() });
    assert_eq!(l.name_model().as_deref(), Some("light"));
    l.set_name_model(None);
    assert_eq!(l.name_model(), None);
}

// ADR 0008: the agent is ready from the time it tells its name; when it goes down, the reason stays.
#[test]
fn the_agent_is_ready_with_its_name_and_leaves_a_reason_when_it_goes() {
    let mut l = AgentLink::default();
    l.set_name("claude".into());
    l.set_images(true);
    assert!(l.ready() && l.images() && l.down().is_none());
    assert_eq!(l.name(), Some("claude"));
    l.disconnect("the agent ended".into());
    assert_eq!(l.down(), Some("the agent ended"));
    assert!(!l.running());
}

// ADR 0008, NAMES-10, NAMES-14: each answer that keeps the name counts, up to a limit, and the
// next question keeps the count.
#[test]
fn the_answers_that_keep_a_name_are_counted() {
    let mut l = AgentLink::default();
    l.name_kept("id", 3);
    assert!(l.asked("id").is_none(), "no question: nothing to count");
    l.ask_name("id", 10);
    assert_eq!(l.asked("id").map(|a| (a.size, a.keeps)), Some((10, 0)));
    (0..5).for_each(|_| l.name_kept("id", 3));
    l.ask_name("id", 40);
    assert_eq!(l.asked("id").map(|a| (a.size, a.keeps)), Some((40, 3)));
}

// ADR 0008, CLAUDE-37: the answer for the name of an image comes in parts, and goes with the turn.
#[test]
fn the_name_of_an_image_comes_in_parts() {
    let mut l = AgentLink::default();
    assert!(l.no_image_waits());
    l.name_image("k".into(), "/tmp/a.png".into());
    l.image_text("k", "sun");
    l.image_text("k", "set");
    l.image_text("other", "lost");
    assert!(!l.no_image_waits());
    assert_eq!(l.take_image("k"), Some(("/tmp/a.png".into(), "sunset".to_string())));
    assert!(l.take_image("k").is_none() && l.no_image_waits());
}
