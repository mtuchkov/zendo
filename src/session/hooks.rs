//! The state of a session from the hooks of Claude Code (ADR 0005).
//!
//! zendo starts Claude Code with `--settings <file>`: hooks of that run only, which Claude Code
//! merges with the settings of the user (SESSIONS-18). Each hook writes a state word to the file
//! in `$ZENDO_STATE`; zendo reads it on each tick. The hooks of "prompt sent" and "stop" also append
//! their input to the file in `$ZENDO_MESSAGES`, for the chat (CLIMSG-2).

use super::State;
use std::path::Path;

/// The hook events and the state that each one reports.
const EVENTS: [(&str, &str); 6] = [
    ("UserPromptSubmit", "works"),
    ("PreToolUse", "works"),
    ("PostToolUse", "works"),
    ("PermissionRequest", "waits"),
    ("Notification", "waits"),
    ("Stop", "idle"),
];

/// Append the input of the hook to the file of the messages as one line, and print nothing:
/// Claude Code adds the output of a "prompt sent" hook to the context (CLIMSG-2, CLIMSG-5). JSON has
/// no raw new line in a string, so the new lines that `tr` takes out are only between tokens.
const MESSAGE_HOOK: &str = r#"{ tr -d '
'; echo; } >> "$ZENDO_MESSAGES""#;

/// The settings with the hooks, as JSON.
pub fn settings() -> String {
    let hooks: serde_json::Map<String, serde_json::Value> = EVENTS
        .iter()
        .map(|(event, state)| {
            let hook = serde_json::json!({ "type": "command", "command": format!("printf {state} > \"$ZENDO_STATE\"") });
            let message = serde_json::json!({ "type": "command", "command": MESSAGE_HOOK });
            let reports = matches!(*event, "UserPromptSubmit" | "Stop");
            let mut group = serde_json::json!({ "hooks": if reports { vec![hook, message] } else { vec![hook] } });
            match *event {
                "PreToolUse" | "PostToolUse" | "PermissionRequest" => group["matcher"] = "*".into(),
                "Notification" => group["matcher"] = "permission_prompt".into(),
                _ => {}
            }
            (event.to_string(), serde_json::json!([group]))
        })
        .collect();
    serde_json::json!({ "hooks": hooks }).to_string()
}

/// The state in the file of a session, if a hook wrote one.
pub fn read(path: &Path) -> Option<State> {
    match std::fs::read_to_string(path).ok()?.trim() {
        "works" => Some(State::Works),
        "waits" => Some(State::Waits),
        "idle" => Some(State::Idle),
        _ => None,
    }
}

#[cfg(test)]
#[path = "hooks_tests.rs"]
mod tests;
