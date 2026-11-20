---
status: "approved"
approved: "2026-11-19"
revision: "3"
date: "2026-11-19"
parent: "zendo.md"
prev: "notes-list.md"
next: "float.md"
level: "turn a note into specifications: the buttons, the key, the session that gets the task, and the task"
note: "Agent draft of the request of the user on 2026-11-17. Approved by the user on 2026-11-17. Revision 2, asked and approved by the user on 2026-11-19: the button in the float (SPEC-6 to SPEC-8), the task goes to the free agent session with the lowest id (SPEC-1, SPEC-9), else into the chat input of the active session with `/new` (SPEC-10). SPEC-6 to SPEC-10 are new identifiers that the user approved. Revision 3, from the decisions of the user of 2026-11-19 after the code review: the task goes at the click with the name that the file has, and a title that comes later goes into the note, does not rename the file, and changes nothing else (SPEC-7). SPEC-11 and SPEC-12 are new identifiers for behavior that no requirement covered. The user approved revision 3 and the new identifiers on 2026-11-19, and answered the open point: the note pane shows the note of the user, also in a session with a checkout of its own (SPEC-11)."
---

# Note to spec

A note holds ideas, decisions and requirements in the order that the user had them. When they are
ready, the user turns the note into specifications with one click: `[ → spec ]` on the line of the
date of the note, in the note pane or in the [float](float.md) of a new note, or `s` in read mode.
zendo gives the task to an agent session and shows its chat, so the user sees the plan and the
edits, and can stop the turn with `esc`.

**Which session.** The task goes to a free agent session: one with no session ID (CLAUDE-40) and an
empty chat input (CLAUDE-43). The one with the lowest id gets it, so session 0, in the main checkout,
comes first; a session 1 to 4 makes the edits in its worktree ([zendo session](zendo-session.md)).
The task names the file of the note where the user wrote it, and the note pane shows that file,
also in a session with a checkout of its own. When no session is free, zendo puts
`/new` and the task into the chat input of the active session and sends nothing: `enter` starts a
new thread with the task ([Chat history](chat-history.md)), and the user can delete `/new` to send
it into the current thread. The instructions are a chip of that line, so they go only with it.

**From the float.** `[ → spec ]` in the float saves the note, closes the float, opens the note in the
note pane, and sends the task at the click, with the name that the file has. When the name model
titles the note later ([Title of a new note](float-title.md)), the title goes in only when the note
has none, the file keeps its name, and nothing else changes (User, 2026-11-19). A note that has no
file yet shows no button.

The agent decides how: for each topic of the note it updates the specification that covers it, or
writes a new one, with the plan first and the rules of the repository; the note stays as it is. The
instructions are in [resources/note-to-spec.md](../../resources/note-to-spec.md), part of the zendo
program. The notes list is in [Notes list and search](notes-list.md).

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| SPEC-1 | Event      | When the user clicks `[ → spec ]` in the head of a note, or presses `s` in read mode on a note, the system shall send the task to turn the note into specifications to the free agent session with the lowest id (SPEC-9), and show the chat of that session. | User, 2026-11-17; User, 2026-11-19 |
| SPEC-2 | Ubiquitous | The system shall have the agent choose, for each topic of the note, to update the specification that covers the topic or to write a new one, and state that plan before the edits. | User, 2026-11-17 |
| SPEC-6 | Event      | When the user clicks `[ → spec ]` in the float, the system shall save the note, close the float, open the note in the note pane, and send the task as in SPEC-1. | User, 2026-11-19 |
| SPEC-10 | Unwanted  | If no agent session is free, then the system shall put `/new` and the task at the start of the chat input of the active agent session, with the text that is already there under them, keep the instructions of the task with it as a chip, focus the chat input, and send nothing. | User, 2026-11-19 |

### Acceptance tests

**`s_and_the_button_send_the_note_to_spec`** (mock agent and a test terminal of 120×40; verifies SPEC-1, SPEC-3)

- Given a note of the notes folder and `docs/design.md`.
- When the user opens the note, clicks `[ → spec ]`, and later presses `s` on `docs/design.md`.
- Then the chat of the active agent session has the task for the note, and `docs/design.md` shows
  no button and sends no task.

**`the_spec_task_goes_to_the_lowest_free_session`** (mock agent; verifies SPEC-1, SPEC-9)

- Given agent session 0 with a session ID, session 1 with a draft in its chat input, and session 2
  to 4 free, with session 0 active.
- When the user presses `s` on a note.
- Then session 2 gets the task, the chat shows session 2, and session 0 and 1 are unchanged.

**`the_float_sends_the_note_to_spec_at_the_click`** (mock agent and a mock name model; verifies SPEC-6, SPEC-7, SPEC-8)

- Given the float with `# ` and an empty body.
- When the user types one line of text, then clicks `[ → spec ]`, and the name model writes the
  title after the click.
- Then the float shows no button before the first save; at the click the float is closed, the note
  pane shows the note, and the task names its file; after the title came, the note has the title
  and the file has the same name.

**`no_free_session_puts_new_and_the_task_into_the_input`** (mock agent; verifies SPEC-10)

- Given five agent sessions with session IDs, and the text `draft` in the chat input of the active one.
- When the user presses `s` on a note.
- Then the chat input holds `/new` and the task, and under them `draft`; the chat input has the
  focus, and no question went to the agent.

**`the_spec_task_asks_for_a_plan_first`** (unit test; verifies SPEC-2, SPEC-4)

- Given the note `docs/notes/2026-11-17-tea.md`.
- When zendo makes the task for the agent.
- Then the task names the note and `project-structure.toml`, asks to update or to write
  specifications for each topic, asks for the plan before the edits, and tells the agent not to
  change the note.

## Detailed requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| SPEC-3 | Unwanted   | If the open file is not a note of the notes folder, then the system shall show no `[ → spec ]` button and send no task for `s`. | Design |
| SPEC-4 | Ubiquitous | The system shall have the agent keep the note unchanged, follow the rules of the repository, and write new documents as drafts. | Design |
| SPEC-5 | Unwanted   | If the chat answers already when the user asks for a spec, then the system shall queue the task as every question. | Design |
| SPEC-7 | Event      | When the name model titles a note (FLOAT-19) after the note went to spec, the system shall put the title into the note only when the note has no title, keep the name of the file, and change no other state: not the open document, the agent session or the mode. | User, 2026-11-19 |
| SPEC-8 | Unwanted   | If the note in the float has no file yet, then the system shall show no `[ → spec ]` button in the float. | User, 2026-11-19 |
| SPEC-9 | Ubiquitous | The system shall count an agent session as free while it has no session ID (CLAUDE-40) and its chat input is empty (CLAUDE-43). | User, 2026-11-19 |
| SPEC-11 | Event     | When the system sends the task to a free agent session, the system shall name and attach the file of the note where the user wrote it, and show that file in the note pane until another document opens, also when that session has a checkout of its own (an exception to FILES-38 and FILES-40). | Design; User, 2026-11-19 |
| SPEC-12 | Unwanted  | If no agent session is free and Claude Code runs in the active agent session, then the system shall put no task into the chat input and shall say that the session is open in Claude Code. | Design |

Tests that verify detailed requirements: `a_late_title_changes_only_the_note`,
`a_late_title_goes_into_the_open_editor`, `a_note_that_goes_to_spec_after_esc_keeps_its_name` and
`a_late_title_does_not_replace_the_title_of_a_note_that_went_to_spec` (SPEC-7);
`the_float_over_a_full_screen_session_shows_the_note` (SPEC-6);
`enter_sends_the_task_of_the_input_to_a_new_thread` (SPEC-10); `the_task_names_the_note_where_it_is`
(SPEC-11); `no_task_goes_into_the_input_of_a_claude_code_session` (SPEC-12).
