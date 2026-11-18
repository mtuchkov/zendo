use super::*;
use agent_client_protocol::v1::schema::{Content, ToolCallUpdateFields};

fn text(s: &str) -> ToolCallContent {
    ToolCallContent::Content(Content::new(ContentBlock::Text(TextContent::new(s))))
}

// ACT-1: a new call carries its state and its first file.
#[test]
fn a_call_has_its_state_and_file() {
    let tc = ToolCall::new("r1", "Read a.md")
        .kind(ToolKind::Read)
        .status(ToolCallStatus::InProgress)
        .locations(vec![ToolCallLocation::new("/repo/a.md"), ToolCallLocation::new("/repo/b.md")]);
    let Event::Tool { state, place, error, kind, .. } = call("k".into(), tc) else { panic!() };
    assert_eq!((state, place.as_deref(), error, kind.as_str()), (ToolState::Running, Some("/repo/a.md"), None, "read"));
}

// ACT-8: a failed update carries the first line of its error; an update without a status says
// `Pending`, which does not move a known call back.
#[test]
fn a_failed_update_has_the_first_line_of_its_error() {
    let u = ToolCallUpdate::new("e1", ToolCallUpdateFields::new().status(ToolCallStatus::Failed).content(vec![text("\n  old_string not found\nmore")]));
    let Event::Tool { state, error, .. } = update("k".into(), u) else { panic!() };
    assert_eq!((state, error.as_deref()), (ToolState::Failed, Some("old_string not found")));

    let u = ToolCallUpdate::new("e1", ToolCallUpdateFields::new().title("Edit a.md".to_string()));
    let Event::Tool { state, error, .. } = update("k".into(), u) else { panic!() };
    assert_eq!((state, error), (ToolState::Pending, None));
}

// ACT-8: the text of a call that did not fail is no error.
#[test]
fn only_a_failed_call_has_an_error() {
    let u = ToolCallUpdate::new("r1", ToolCallUpdateFields::new().status(ToolCallStatus::Completed).content(vec![text("file text")]));
    let Event::Tool { error, .. } = update("k".into(), u) else { panic!() };
    assert_eq!(error, None);
}
