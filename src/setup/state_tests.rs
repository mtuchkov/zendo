use super::*;

// CLAUDE-9: the default prompt does not tell the agent to write into the note of the user; the
// agent edits a note only when the user asks for it.
#[test]
fn the_default_prompt_does_not_ask_the_agent_to_edit_the_note() {
    assert!(!DEFAULT_PROMPT.contains("edit the note"), "{DEFAULT_PROMPT}");
    assert!(DEFAULT_PROMPT.contains("Ask one question when something is unclear."), "the rest stays");
}

// PROFILE-10: a rename and a delete of a file reach its thread in every agent and every profile.
#[test]
fn a_rename_and_a_delete_reach_the_threads_of_every_profile() {
    let mut s = State::default();
    s.set_session("claude", "docs/a.md", "plain".into());
    s.set_session("claude --profile work", "docs/a.md", "of-work".into());
    assert!(s.move_note("docs/a.md", "docs/b.md"));
    let at = |s: &State, note: &str| (s.session("claude", note), s.session("claude --profile work", note));
    assert_eq!(at(&s, "docs/b.md"), (Some("plain".into()), Some("of-work".into())));
    assert_eq!(at(&s, "docs/a.md"), (None, None));
    assert!(!s.move_note("docs/x.md", "docs/y.md"), "a file without a thread");
    s.forget_note("docs/b.md");
    assert_eq!(at(&s, "docs/b.md"), (None, None));
}
