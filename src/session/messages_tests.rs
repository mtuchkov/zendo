use super::*;
use std::io::Write;

fn inbox(name: &str) -> (std::path::PathBuf, Inbox) {
    let path = std::env::temp_dir().join(format!("zendo-inbox-{name}-{}.messages", std::process::id()));
    let _ = std::fs::remove_file(&path);
    (path.clone(), Inbox::new(path))
}

fn append(path: &std::path::Path, text: &str) {
    std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap().write_all(text.as_bytes()).unwrap();
}

// CLIMSG-1: a prompt and an answer, each once, in their order.
#[test]
fn takes_the_new_prompts_and_answers_once() {
    let (path, mut inbox) = inbox("once");
    assert!(inbox.take().is_empty(), "no file yet");
    append(&path, "{\"hook_event_name\":\"UserPromptSubmit\",\"prompt\":\"hi\"}\n{\"hook_event_name\":\"Stop\",\"last_assistant_message\":\"hello\"}\n");
    assert_eq!(inbox.take(), vec![Message::Prompt("hi".into()), Message::Answer("hello".into())]);
    assert!(inbox.take().is_empty(), "each message once");
    let _ = std::fs::remove_file(&path);
}

// A line that a hook still writes waits until it is whole.
#[test]
fn a_line_without_its_end_waits() {
    let (path, mut inbox) = inbox("partial");
    append(&path, "{\"hook_event_name\":\"UserPromptSubmit\",");
    assert!(inbox.take().is_empty());
    append(&path, "\"prompt\":\"hi\"}\n");
    assert_eq!(inbox.take(), vec![Message::Prompt("hi".into())]);
    let _ = std::fs::remove_file(&path);
}

// CLIMSG-6: a stop without an answer, another event, or a broken line gives no message.
#[test]
fn a_stop_without_an_answer_gives_nothing() {
    let (path, mut inbox) = inbox("none");
    append(&path, "{\"hook_event_name\":\"Stop\"}\n{\"hook_event_name\":\"Stop\",\"last_assistant_message\":\"\"}\n{\"hook_event_name\":\"PreToolUse\"}\nnot json\n");
    assert!(inbox.take().is_empty());
    let _ = std::fs::remove_file(&path);
}

// CLIMSG-4: a pasted question comes back between the marks of pasted content; it is the same text.
#[test]
fn the_marks_of_pasted_content_are_not_part_of_the_text() {
    assert_eq!(plain("<pasted_content id=\"1\">\none\ntwo\n</pasted_content id=\"1\">\n"), "one\ntwo");
    assert_eq!(plain("  hi \n"), "hi");
}
