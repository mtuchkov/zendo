use super::*;
use std::time::{Duration, Instant};

fn taken(names: &[&str]) -> Vec<String> {
    names.iter().map(|s| s.to_string()).collect()
}

// NAMES-1, NAMES-2: the first free word of one to 12 letters, in lower case.
#[test]
fn pick_takes_the_first_free_word() {
    assert_eq!(pick("Parser, lexer, tokens", &[]).as_deref(), Some("parser"));
    assert_eq!(pick("parser lexer tokens", &taken(&["parser"])).as_deref(), Some("lexer"));
    assert_eq!(pick("Here are the names:\nhooks, x, layout", &[]).as_deref(), Some("hooks"), "the last line");
    assert_eq!(pick("averyveryverylongword short", &[]).as_deref(), Some("short"));
    assert_eq!(pick("mock", &taken(&["mock"])), None, "no free word");
    assert_eq!(pick("", &[]), None);
}

fn tools(titles: &[&str]) -> Item {
    let uses = titles.iter().map(|t| crate::app::ToolUse { id: t.to_string(), title: t.to_string(), kind: "read".into() }).collect();
    Item::Tools { uses, started: Instant::now(), took: None }
}

// NAMES-1: a question and one complete agent message, also while the turn runs; a message that is
// still streaming at the end of a running turn does not count yet.
#[test]
fn told_needs_a_question_and_a_complete_agent_message() {
    let q = Item::User("fix the parser".into());
    let m = Item::Agent("I look at the lexer.".into());
    assert_eq!(told(std::slice::from_ref(&q), true), None, "no agent message");
    assert_eq!(told(std::slice::from_ref(&m), false), None, "no question");
    assert_eq!(told(&[q.clone(), m.clone()], true), None, "the message still streams");
    assert_eq!(told(&[q.clone(), m.clone()], false), Some(34), "the turn ended");
    assert_eq!(told(&[q.clone(), m.clone(), tools(&["Read a.rs"])], true), Some(43), "a tool call ends the message; its title counts");
}

// NAMES-1, NAMES-10, NAMES-14: the first question comes at once; the next one after 200 more
// characters or 30 s more of a running turn with something new, both doubled by each kept name.
#[test]
fn due_after_the_thresholds_doubled_by_each_kept_name() {
    let t0 = Instant::now();
    let s = |n| Duration::from_secs(n);
    assert!(due(10, None, None, t0), "first question");
    let last = Asked { size: 100, at: t0, keeps: 0 };
    assert!(!due(299, None, Some(&last), t0 + s(99)), "199 more characters, no turn");
    assert!(due(300, None, Some(&last), t0), "200 more characters");
    assert!(!due(101, Some(t0), Some(&last), t0 + s(29)), "29 s of the turn");
    assert!(due(101, Some(t0), Some(&last), t0 + s(30)), "30 s of the turn");
    assert!(!due(100, Some(t0), Some(&last), t0 + s(60)), "nothing new");
    assert!(!due(101, Some(t0 + s(20)), Some(&last), t0 + s(40)), "the turn runs 20 s only");
    let kept = Asked { keeps: 2, ..last };
    assert!(!due(500, None, Some(&kept), t0), "two kept names: 800 more characters");
    assert!(due(900, None, Some(&kept), t0));
    assert!(!due(101, Some(t0), Some(&kept), t0 + s(119)), "two kept names: 120 s");
    assert!(due(101, Some(t0), Some(&kept), t0 + s(120)));
}

// NAMES-14: the question for a name has the first message and the last ones, with tool calls.
#[test]
fn the_conversation_has_the_first_and_the_last_messages() {
    let mut items = vec![Item::User("first".into())];
    items.extend((0..20).map(|i| Item::Agent(format!("a{i}"))));
    items.push(tools(&["Read a.rs", "Edit b.rs"]));
    let text = conversation(&items);
    assert!(text.starts_with("user: first\n\n"), "{text}");
    assert!(!text.contains("agent: a13\n") && text.contains("agent: a14\n") && text.ends_with("tools: Read a.rs; Edit b.rs"), "{text}");
    assert_eq!(text.matches("\n\n").count(), MESSAGES - 1);
}
