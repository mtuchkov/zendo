//! The tool calls of the agent as events of zendo: the state, the first file, and the first line of
//! the error of a failed call (docs/zendo/activity.md: ACT-1, ACT-8).
use super::*;
use agent_client_protocol::v1::schema::{ToolCall, ToolCallLocation, ToolCallStatus, ToolCallUpdate};

/// The state of zendo for a status of ACP.
pub(super) fn state_of(s: ToolCallStatus) -> ToolState {
    match s {
        ToolCallStatus::InProgress => ToolState::Running,
        ToolCallStatus::Completed => ToolState::Done,
        ToolCallStatus::Failed => ToolState::Failed,
        _ => ToolState::Pending,
    }
}

fn place_of(locations: &[ToolCallLocation]) -> Option<String> {
    locations.first().map(|l| l.path.display().to_string())
}

/// The first line of the text of a failed call.
fn error_of(state: ToolState, content: &[ToolCallContent]) -> Option<String> {
    if state != ToolState::Failed {
        return None;
    }
    content.iter().find_map(|c| match c {
        ToolCallContent::Content(c) => match &c.content {
            ContentBlock::Text(t) => t.text.lines().map(str::trim).find(|l| !l.is_empty()).map(str::to_string),
            _ => None,
        },
        _ => None,
    })
}

/// A new tool call.
pub(super) fn call(key: String, tc: ToolCall) -> Event {
    let state = state_of(tc.status);
    Event::Tool {
        key,
        id: tc.tool_call_id.0.to_string(),
        title: tc.title,
        kind: kind_name(&tc.kind),
        state,
        place: place_of(&tc.locations),
        error: error_of(state, &tc.content),
    }
}

/// An update of a tool call: what it leaves out stays as it was. Without a status it says
/// `Pending`, which does not move a known call back (a call only goes forward).
pub(super) fn update(key: String, u: ToolCallUpdate) -> Event {
    let f = u.fields;
    let state = f.status.map_or(ToolState::Pending, state_of);
    Event::Tool {
        key,
        id: u.tool_call_id.0.to_string(),
        title: f.title.unwrap_or_default(),
        kind: f.kind.as_ref().map(kind_name).unwrap_or_default(),
        state,
        place: f.locations.as_deref().and_then(place_of),
        error: f.content.as_deref().and_then(|c| error_of(state, c)),
    }
}

#[cfg(test)]
#[path = "tool_events_tests.rs"]
mod tests;
