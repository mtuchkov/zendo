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

// NAMES-4, NAMES-7: the start of a session gives its session ID, and whether `/clear` started it.
#[test]
fn a_session_start_gives_the_session_id() {
    let (path, mut inbox) = inbox("start");
    append(&path, "{\"hook_event_name\":\"SessionStart\",\"session_id\":\"s2\",\"source\":\"clear\"}\n{\"hook_event_name\":\"SessionStart\",\"session_id\":\"s3\",\"source\":\"resume\"}\n{\"hook_event_name\":\"SessionStart\",\"source\":\"startup\"}\n");
    assert_eq!(inbox.take(), vec![Message::Session { id: "s2".into(), clear: true }, Message::Session { id: "s3".into(), clear: false }]);
    let _ = std::fs::remove_file(&path);
}

/// A transcript file of Claude Code beside the inbox `name`, with `history` in it.
fn transcript(name: &str, history: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("zendo-inbox-{name}-{}.jsonl", std::process::id()));
    std::fs::write(&path, history).unwrap();
    path
}

fn hook(event: &str, transcript: &std::path::Path, rest: &str) -> String {
    format!("{{\"hook_event_name\":\"{event}\",\"transcript_path\":\"{}\"{rest}}}\n", transcript.display())
}

fn said(text: &str) -> String {
    format!("{{\"type\":\"assistant\",\"isSidechain\":false,\"message\":{{\"role\":\"assistant\",\"content\":[{{\"type\":\"text\",\"text\":\"{text}\"}}]}}}}\n")
}

const BASH: &str = "{\"type\":\"assistant\",\"isSidechain\":false,\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"id\":\"t1\",\"name\":\"Bash\",\"input\":{\"command\":\"ls -la\",\"description\":\"List the files\"}}]}}\n";

// CLIMSG-8, CLIMSG-9: the text between the tool calls and each tool call come from the transcript
// that the hooks name, in their order; the history of a resumed session, the thoughts, the tool
// results and the work of a subagent do not.
#[test]
fn the_text_and_the_tool_calls_of_a_turn_come_from_the_transcript() {
    let (path, mut inbox) = inbox("transcript");
    let t = transcript("transcript", &said("old history"));
    append(&path, &hook("SessionStart", &t, ",\"session_id\":\"s1\",\"source\":\"resume\""));
    assert_eq!(inbox.take(), vec![Message::Session { id: "s1".into(), clear: false }], "no history");
    append(&path, &hook("UserPromptSubmit", &t, ",\"prompt\":\"go\""));
    append(&t, &said("I look first."));
    append(&t, BASH);
    append(&t, "{\"type\":\"assistant\",\"isSidechain\":false,\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"thinking\",\"thinking\":\"hm\"}]}}\n");
    append(&t, "{\"type\":\"user\",\"isSidechain\":false,\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"tool_result\",\"tool_use_id\":\"t1\",\"content\":\"a b\"}]}}\n");
    append(&t, &said("from a subagent").replace("\"isSidechain\":false", "\"isSidechain\":true"));
    let tool = Message::Tool { id: "t1".into(), title: "Bash: List the files".into(), kind: "execute".into() };
    assert_eq!(inbox.take(), vec![Message::Prompt("go".into()), Message::Text("I look first.".into()), tool]);
    append(&t, "{\"type\":\"assistant\",");
    assert!(inbox.take().is_empty(), "a line of the transcript without its end waits");
    let _ = (std::fs::remove_file(&path), std::fs::remove_file(&t));
}

// CLIMSG-1, CLIMSG-8: the final answer shows once, whether the transcript or the "stop" hook has
// it first.
#[test]
fn the_final_answer_shows_once() {
    let (path, mut inbox) = inbox("final");
    let t = transcript("final", "");
    append(&path, &hook("SessionStart", &t, ",\"session_id\":\"s1\",\"source\":\"resume\""));
    inbox.take();
    append(&path, &hook("UserPromptSubmit", &t, ",\"prompt\":\"one\""));
    append(&t, &said("Done one."));
    append(&path, &hook("Stop", &t, ",\"last_assistant_message\":\"Done one.\""));
    assert_eq!(inbox.take(), vec![Message::Prompt("one".into()), Message::Text("Done one.".into())]);
    append(&path, &hook("UserPromptSubmit", &t, ",\"prompt\":\"two\""));
    append(&path, &hook("Stop", &t, ",\"last_assistant_message\":\"Done two.\""));
    assert_eq!(inbox.take(), vec![Message::Prompt("two".into()), Message::Answer("Done two.".into())]);
    append(&t, &said("Done two."));
    assert!(inbox.take().is_empty(), "the transcript has the answer later: {:?}", inbox);
    let _ = (std::fs::remove_file(&path), std::fs::remove_file(&t));
}

// CLIMSG-10: after `/clear`, Claude Code writes a new transcript; zendo reads that one from its
// start, and no more of the old one.
#[test]
fn a_clear_moves_to_the_new_transcript() {
    let (path, mut inbox) = inbox("clear");
    let (old, new) = (transcript("clear-old", ""), transcript("clear-new", ""));
    append(&path, &hook("SessionStart", &old, ",\"session_id\":\"s1\",\"source\":\"resume\""));
    inbox.take();
    append(&new, &said("In the new conversation."));
    append(&path, &hook("SessionStart", &new, ",\"session_id\":\"s2\",\"source\":\"clear\""));
    assert_eq!(inbox.take(), vec![Message::Session { id: "s2".into(), clear: true }, Message::Text("In the new conversation.".into())]);
    append(&old, &said("late, in the old one"));
    append(&new, &said("More."));
    assert_eq!(inbox.take(), vec![Message::Text("More.".into())]);
    let _ = (std::fs::remove_file(&path), std::fs::remove_file(&old), std::fs::remove_file(&new));
}

// CLIMSG-11: a line of the transcript in a format that zendo does not know gives nothing, and the
// prompt and the final answer of the hooks still show.
#[test]
fn an_unknown_transcript_line_gives_nothing() {
    let (path, mut inbox) = inbox("unknown");
    let t = transcript("unknown", "");
    append(&path, &hook("SessionStart", &t, ",\"session_id\":\"s1\",\"source\":\"resume\""));
    inbox.take();
    append(&t, "not json at all\n{\"type\":\"assistant\",\"message\":\"a string\"}\n{\"type\":\"assistant\",\"message\":{\"content\":[{\"type\":\"new_kind\"}]}}\n");
    append(&path, &hook("UserPromptSubmit", &t, ",\"prompt\":\"go\""));
    append(&path, &hook("Stop", &t, ",\"last_assistant_message\":\"Done.\""));
    assert_eq!(inbox.take(), vec![Message::Prompt("go".into()), Message::Answer("Done.".into())]);
    let _ = (std::fs::remove_file(&path), std::fs::remove_file(&t));
}
