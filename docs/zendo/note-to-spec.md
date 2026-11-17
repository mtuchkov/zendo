---
status: "approved"
approved: "2026-11-17"
revision: "1"
date: "2026-11-17"
parent: "zendo.md"
prev: "notes-list.md"
next: "float.md"
level: "turn a note into specifications: the button, the key, and the task that the agent gets"
note: "Agent draft of the request of the user on 2026-11-17. Approved by the user on 2026-11-17."
---

# Note to spec

A note holds ideas, decisions and requirements in the order that the user had them. When they are
ready, the user turns the note into specifications with one click: `[ → spec ]` on the line of the
date of the note, or `s` in read mode. zendo gives the task to the agent in the chat of the active
agent session, so the user sees the plan and the edits, and can stop the turn with `esc`.

The agent decides how. It reads the note, `project-structure.toml` for the place and the template
of specifications, and the specifications that already cover the topics of the note. For each topic
it updates the specification that covers it, or writes a new one where none does. It answers first
with the plan in a few lines, then makes the edits with the rules of the repository (`AGENTS.md`,
`CLAUDE.md`): requirements in EARS, new requirement IDs only as proposals, new documents as drafts.
The note stays as it is. The instructions are in
[resources/note-to-spec.md](../../resources/note-to-spec.md), part of the zendo program. The
notes list is in [Notes list and search](notes-list.md).

## Architecturally significant requirements

| ID     | Pattern | Requirement                                                                                                                                                                       | Source           |
| ------ | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| SPEC-1 | Event   | When the user clicks `[ → spec ]` in the head of a note, or presses `s` in read mode on a note, the system shall send the agent in the chat of the active agent session the task to turn the note into specifications. | User, 2026-11-17 |
| SPEC-2 | Ubiquitous | The system shall have the agent choose, for each topic of the note, to update the specification that covers the topic or to write a new one, and state that plan before the edits. | User, 2026-11-17 |

### Acceptance tests

**`s_and_the_button_send_the_note_to_spec`** (mock agent and a test terminal of 120×40; verifies SPEC-1, SPEC-3)

- Given a note of the notes folder and `docs/design.md`.
- When the user opens the note, clicks `[ → spec ]`, and later presses `s` on `docs/design.md`.
- Then the chat of the active agent session has the task for the note, and `docs/design.md` shows
  no button and sends no task.

**`the_spec_task_asks_for_a_plan_first`** (unit test; verifies SPEC-2, SPEC-4)

- Given the note `docs/notes/2026-11-17-tea.md`.
- When zendo makes the task for the agent.
- Then the task names the note and `project-structure.toml`, asks to update or to write
  specifications for each topic, asks for the plan before the edits, and tells the agent not to
  change the note.

## Detailed requirements

| ID     | Pattern    | Requirement                                                                                                                       | Source           |
| ------ | ---------- | --------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| SPEC-3 | Unwanted   | If the open file is not a note of the notes folder, then the system shall show no `[ → spec ]` button and send no task for `s`. | Design           |
| SPEC-4 | Ubiquitous | The system shall have the agent keep the note unchanged, follow the rules of the repository, and write new documents as drafts.  | Design           |
| SPEC-5 | Unwanted   | If the chat answers already when the user asks for a spec, then the system shall queue the task as every question.              | Design           |
