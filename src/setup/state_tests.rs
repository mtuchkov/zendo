use super::*;

// CLAUDE-9: the default prompt does not tell the agent to write into the note of the user; the
// agent edits a note only when the user asks for it.
#[test]
fn the_default_prompt_does_not_ask_the_agent_to_edit_the_note() {
    assert!(!DEFAULT_PROMPT.contains("edit the note"), "{DEFAULT_PROMPT}");
    assert!(DEFAULT_PROMPT.contains("Ask one question when something is unclear."), "the rest stays");
}
