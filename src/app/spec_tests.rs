//! The task of note to spec (docs/zendo/note-to-spec.md).
use super::*;

// SPEC-2, SPEC-4: the task names the note and the project structure, asks to update or to write a
// specification for each topic, asks for the plan before the edits, and keeps the note unchanged.
#[test]
fn the_spec_task_asks_for_a_plan_first() {
    let task = spec_task("docs/notes/2026-11-17-tea.md");
    assert!(task.contains("docs/notes/2026-11-17-tea.md") && !task.contains("{{"), "{task}");
    assert!(task.contains("project-structure.toml") && task.contains("AGENTS.md"), "{task}");
    assert!(task.contains("update the specification that covers the topic") && task.contains("write a new specification"), "{task}");
    let plan = task.find("Answer first").unwrap();
    assert!(plan < task.find("Then make the edits").unwrap(), "the plan before the edits");
    assert!(task.contains("Do not change the note") && task.contains("status: draft"), "{task}");
    assert!(spec_question("docs/notes/a.md").contains("docs/notes/a.md"));
}
