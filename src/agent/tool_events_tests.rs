use super::*;
use agent_client_protocol::v1::schema::{Content, Diff, ToolCallUpdateFields};

fn text(s: &str) -> ToolCallContent {
    ToolCallContent::Content(Content::new(ContentBlock::Text(TextContent::new(s))))
}

fn info(e: Event) -> (ToolState, ToolInfo) {
    let Event::Tool { state, info, .. } = e else { panic!("not a tool event") };
    (state, info)
}

// ACT-1, INTER-9: a new call carries its state, its first file and its line.
#[test]
fn a_call_has_its_state_and_file() {
    let tc = ToolCall::new("r1", "Read a.md")
        .kind(ToolKind::Read)
        .status(ToolCallStatus::InProgress)
        .locations(vec![ToolCallLocation::new("/repo/a.md").line(12), ToolCallLocation::new("/repo/b.md")]);
    let (state, i) = info(call("k".into(), tc));
    assert_eq!((state, i.place.as_deref(), i.line, i.error), (ToolState::Running, Some("/repo/a.md"), Some(12), None));
}

// ACT-8: a failed update carries the first line of its error; an update without a status says
// `Pending`, which does not move a known call back.
#[test]
fn a_failed_update_has_the_first_line_of_its_error() {
    let u = ToolCallUpdate::new("e1", ToolCallUpdateFields::new().status(ToolCallStatus::Failed).content(vec![text("\n  old_string not found\nmore")]));
    let (state, i) = info(update("k".into(), u));
    assert_eq!((state, i.error.as_deref()), (ToolState::Failed, Some("old_string not found")));

    let u = ToolCallUpdate::new("e1", ToolCallUpdateFields::new().title("Edit a.md".to_string()));
    let (state, i) = info(update("k".into(), u));
    assert_eq!((state, i.error), (ToolState::Pending, None));
}

// INTER-2: the input of a command is its command; other input is short JSON.
#[test]
fn the_input_reads_short() {
    let tc = ToolCall::new("b1", "npm test").kind(ToolKind::Execute).raw_input(serde_json::json!({"command": "npm test", "description": "tests"}));
    assert_eq!(info(call("k".into(), tc)).1.input.as_deref(), Some("npm test"));
    let tc = ToolCall::new("g1", "grep").kind(ToolKind::Search).raw_input(serde_json::json!({"pattern": "fn main"}));
    assert_eq!(info(call("k".into(), tc)).1.input.as_deref(), Some(r#"{"pattern":"fn main"}"#));
}

// INTER-2: the output of a call is its text without the code fences; a diff shows its lines.
#[test]
fn the_output_is_the_text_or_the_diff() {
    let u = ToolCallUpdate::new("r1", ToolCallUpdateFields::new().status(ToolCallStatus::Completed).content(vec![text("```\none\ntwo\n```")]));
    assert_eq!(info(update("k".into(), u)).1.output, vec!["one", "two"]);
    let d = ToolCallContent::Diff(Diff::new("/repo/a.md", "new line").old_text("old line".to_string()));
    let u = ToolCallUpdate::new("e1", ToolCallUpdateFields::new().content(vec![d]));
    assert_eq!(info(update("k".into(), u)).1.output, vec!["- old line", "+ new line"]);
}

// ACT-6: a command gives its output and its exit code in `_meta` when zendo asked for it.
#[test]
fn a_command_gives_its_output_and_exit_code() {
    let mut meta = serde_json::Map::new();
    meta.insert("terminal_output".into(), serde_json::json!({"terminal_id": "b1", "data": "1\n2\n3\n4\n5\n"}));
    meta.insert("terminal_exit".into(), serde_json::json!({"terminal_id": "b1", "exit_code": 1, "signal": null}));
    let u = ToolCallUpdate::new("b1", ToolCallUpdateFields::new().status(ToolCallStatus::Failed)).meta(meta);
    let (_, i) = info(update("k".into(), u));
    assert_eq!((i.output.len(), i.output.last().map(String::as_str), i.exit), (5, Some("5"), Some(1)));
}

// ACT-8: the text of a call that did not fail is no error.
#[test]
fn only_a_failed_call_has_an_error() {
    let u = ToolCallUpdate::new("r1", ToolCallUpdateFields::new().status(ToolCallStatus::Completed).content(vec![text("file text")]));
    assert_eq!(info(update("k".into(), u)).1.error, None);
}
