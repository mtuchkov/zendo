//! The tool calls of the agent as events of zendo: the state, the first file and line, the first
//! line of the error, the input, the output, and the exit code of a command
//! (docs/zendo/activity.md: ACT-1, ACT-6, ACT-8; docs/zendo/interactivity.md: INTER-2).
use super::*;
use agent_client_protocol::v1::schema::{Meta, ToolCall, ToolCallLocation, ToolCallStatus, ToolCallUpdate};

/// zendo keeps at most this many lines of the output of a call.
const KEEP: usize = 200;
/// The input of a call shows at most this many characters.
const INPUT: usize = 300;

/// The state of zendo for a status of ACP.
pub(super) fn state_of(s: ToolCallStatus) -> ToolState {
    match s {
        ToolCallStatus::InProgress => ToolState::Running,
        ToolCallStatus::Completed => ToolState::Done,
        ToolCallStatus::Failed => ToolState::Failed,
        _ => ToolState::Pending,
    }
}

fn texts(content: &[ToolCallContent]) -> impl Iterator<Item = &str> {
    content.iter().filter_map(|c| match c {
        ToolCallContent::Content(c) => match &c.content {
            ContentBlock::Text(t) => Some(t.text.as_str()),
            _ => None,
        },
        _ => None,
    })
}

/// The first line of the text of a failed call.
fn error_of(state: ToolState, content: &[ToolCallContent]) -> Option<String> {
    if state != ToolState::Failed {
        return None;
    }
    texts(content).flat_map(str::lines).map(str::trim).find(|l| !l.is_empty()).map(str::to_string)
}

/// The last lines of `lines`, at most `KEEP`.
fn last(mut lines: Vec<String>) -> Vec<String> {
    lines.drain(..lines.len().saturating_sub(KEEP));
    lines
}

/// The lines of the text and of the diffs of a call, without code fences.
fn output_of(content: &[ToolCallContent]) -> Vec<String> {
    let text = texts(content).flat_map(str::lines).filter(|l| !l.trim_start().starts_with("```")).map(str::to_string);
    let diffs = content.iter().filter_map(|c| if let ToolCallContent::Diff(d) = c { Some(d) } else { None }).flat_map(|d| {
        let old = d.old_text.iter().flat_map(|o| o.lines()).map(|l| format!("- {l}"));
        old.chain(d.new_text.lines().map(|l| format!("+ {l}"))).collect::<Vec<_>>()
    });
    last(text.chain(diffs).collect())
}

/// The input of a call: the command of a shell call, else its JSON, short.
fn input_of(raw: Option<&serde_json::Value>) -> Option<String> {
    let raw = raw?;
    let s = raw.get("command").and_then(|c| c.as_str()).map_or_else(|| raw.to_string(), str::to_string);
    let s: String = s.chars().take(INPUT).collect();
    (!s.is_empty() && s != "{}").then_some(s)
}

/// The output and the exit code of a command, when zendo asked for them (ADR 0014).
fn terminal_of(meta: Option<&Meta>) -> (Vec<String>, Option<i32>) {
    let Some(m) = meta else { return (Vec::new(), None) };
    let data = m.get("terminal_output").and_then(|o| o.get("data")).and_then(|d| d.as_str()).unwrap_or("");
    let exit = m.get("terminal_exit").and_then(|e| e.get("exit_code")).and_then(serde_json::Value::as_i64).and_then(|c| i32::try_from(c).ok());
    (last(data.lines().map(str::to_string).collect()), exit)
}

/// What a call or an update tells.
struct Parts<'a> {
    state: ToolState,
    locations: Option<&'a [ToolCallLocation]>,
    content: Option<&'a [ToolCallContent]>,
    raw: Option<&'a serde_json::Value>,
    meta: Option<&'a Meta>,
}

fn info_of(p: Parts) -> ToolInfo {
    let first = p.locations.and_then(<[ToolCallLocation]>::first);
    let (term, exit) = terminal_of(p.meta);
    let output = if term.is_empty() { p.content.map(output_of).unwrap_or_default() } else { term };
    ToolInfo {
        place: first.map(|l| l.path.display().to_string()),
        line: first.and_then(|l| l.line),
        error: p.content.and_then(|c| error_of(p.state, c)),
        input: input_of(p.raw),
        output,
        exit,
    }
}

/// A new tool call.
pub(super) fn call(key: String, tc: ToolCall) -> Event {
    let state = state_of(tc.status);
    let info = info_of(Parts { state, locations: Some(&tc.locations), content: Some(&tc.content), raw: tc.raw_input.as_ref(), meta: tc.meta.as_ref() });
    Event::Tool { key, id: tc.tool_call_id.0.to_string(), title: tc.title, kind: kind_name(&tc.kind), state, info }
}

/// An update of a tool call: what it leaves out stays as it was. Without a status it says
/// `Pending`, which does not move a known call back (a call only goes forward).
pub(super) fn update(key: String, u: ToolCallUpdate) -> Event {
    let f = u.fields;
    let state = f.status.map_or(ToolState::Pending, state_of);
    let info = info_of(Parts { state, locations: f.locations.as_deref(), content: f.content.as_deref(), raw: f.raw_input.as_ref(), meta: u.meta.as_ref() });
    Event::Tool {
        key,
        id: u.tool_call_id.0.to_string(),
        title: f.title.unwrap_or_default(),
        kind: f.kind.as_ref().map(kind_name).unwrap_or_default(),
        state,
        info,
    }
}

#[cfg(test)]
#[path = "tool_events_tests.rs"]
mod tests;
