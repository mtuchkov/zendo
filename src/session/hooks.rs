//! The state of a session from the hooks of Claude Code (ADR 0005).
//!
//! zendo starts Claude Code with `--settings <file>`: hooks of that run only, which Claude Code
//! merges with the settings of the user (SESSIONS-18). Each hook writes a state word to the file
//! in `$ZENDO_STATE`; zendo reads it on each tick.

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

/// The settings with the hooks, as JSON.
pub fn settings() -> String {
    let hooks: serde_json::Map<String, serde_json::Value> = EVENTS
        .iter()
        .map(|(event, state)| {
            let hook = serde_json::json!({ "type": "command", "command": format!("printf {state} > \"$ZENDO_STATE\"") });
            let mut group = serde_json::json!({ "hooks": [hook] });
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
