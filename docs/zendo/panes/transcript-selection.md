---
status: "implemented"
approved: "2026-11-19"
revision: "1"
date: "2026-11-19"
parent: "panes.md"
prev: "selection.md"
next: "keymap.md"
level: "the selection and the cursor in the chat transcript: what they hold, and what new output does to them"
note: "Agent draft from the design of the user in the chat on 2026-11-19, after a panic: the cursor of the transcript pointed past its rows. The user approved PANES-50 to PANES-58 on 2026-11-19. The decision is in ADR 0026. Implemented in zendo 0.126.0."
---

# Selection in the transcript

The transcript is the text of the chat thread. A selection in it is a range of that text, not a
range of the screen: it can start and end in rows that are out of view. The mouse and the
selection keys make the same selection. The general rules of selection are in
[Text selection](selection.md); the transcript keys are in [Chat](../chat.md) (CLAUDE-29). These
rules do not apply to the Claude Code session that `ctrl-o` shows in the chat pane: it is a
terminal, and its selection is the selection of the screen ([Sessions](../sessions.md)).

**A frozen transcript.** While the transcript has the keys, or a selection is in it, zendo keeps
the rows of the transcript as they were when this started. The text under the cursor and under the
selection does not move: the keyboard cursor stays on its text. New output of the agent goes below
these rows. `↓ 12 new rows · End` then covers the last row of the chat pane, so the rows in view
do not move. The key `bottom` (`End`, `G`) takes the new rows into the transcript and moves the cursor to the last row with text.

When the agent writes into the message that was the last one when the rows froze, the frozen rows
of that message stay as they were, and the rest of the message comes as new rows (Design).

**A still view.** A view at the end of the thread follows the new rows, unless the rows are
frozen: then it stays, and shows `↓`. A view that the user scrolled up keeps its rows, also with
no selection and without the keys.

**A drag at the edge.** A drag of the mouse with the pointer on the top or the bottom row of the
pane, or beyond it, scrolls the transcript, also while the pointer does not move. The selection
grows with the scroll.

**What starts over.** Another thread in the chat pane drops the frozen rows and the selection. A
new width of the pane wraps the text again: zendo freezes the new rows, drops the selection, and
puts the cursor on the last row with text. `o` opens or closes a step: zendo freezes the new rows,
keeps the cursor on the step, and drops the selection.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| PANES-50 | Ubiquitous | The system shall keep a selection in the chat transcript as a range of the rows of the thread, so that it can start and end in rows that are out of view, and the mouse and the selection keys shall make the same selection. | User, 2026-11-19 |
| PANES-51 | State      | While the transcript has the keys or holds a selection, the system shall keep its rows as they were when this started, and shall add new output only below them. | User, 2026-11-19 |

### Acceptance tests

**`a_selection_in_the_transcript_reaches_rows_out_of_view`** (unit test; verifies PANES-50,
PANES-52)

- Given a thread of 100 rows in a chat pane of 20 rows, at its end.
- When the user drags from row 90 to the top row of the pane and holds the pointer there for 30
  ticks.
- Then the view shows earlier rows, and the release copies the text from the row under the pointer
  to row 90.

**`new_output_waits_below_a_selection`** (mock agent; verifies PANES-51, PANES-53, PANES-54, PANES-58)

- Given a selection in the transcript while the agent answers.
- When the agent writes 12 more rows.
- Then the rows in view, the selected text and the cursor stay where they were, the last row of
  the pane shows
  `↓ 12 new rows · End`, and after `End` the cursor is on the last row with text.

**`a_shorter_thread_starts_the_transcript_over`** (unit test; verifies PANES-55)

- Given the cursor of the transcript on row 16, with a selection from row 12.
- When the chat pane shows a thread of 6 rows.
- Then zendo does not stop, the cursor is on the last row with text, and no text is selected.

## Detailed requirements

| ID       | Pattern  | Requirement | Source |
| -------- | -------- | ----------- | ------ |
| PANES-52 | State    | While the user drags the mouse in the chat transcript with the pointer on the top or the bottom row of the pane or beyond it, the system shall scroll the transcript in that direction on each tick and grow the selection to the row under the pointer. | User, 2026-11-19 |
| PANES-53 | State    | While new output waits below the frozen rows, the system shall show `↓ <n> new rows · End` on the last row of the chat pane. | User, 2026-11-19 |
| PANES-54 | Event    | When the user presses the key `bottom` while new output waits, the system shall add the new rows to the transcript and move the cursor to the last row with text. | User, 2026-11-19 |
| PANES-55 | Event    | When the chat pane shows another thread, the system shall drop the frozen rows and the selection, and put the cursor on the last row with text. | Design |
| PANES-56 | Event    | When the width of the chat pane changes, or the user opens or closes a step, the system shall freeze the new rows and drop the selection; the cursor shall go to the last row with text after a new width, and stay on the step after `o`. | User, 2026-11-19 (width); Design (step) |
| PANES-58 | State    | While the view of the transcript is not at its end, the system shall keep the same rows in view when rows come or go below them. | User, 2026-11-19 |
| PANES-57 | Unwanted | If the agent writes into the message that was the last one when the rows froze, then the system shall keep its frozen rows as they were and show the rest of the message as new rows. | Design |

Tests that verify detailed requirements: `bottom_takes_the_new_rows_and_keeps_the_selection`
(PANES-54), `the_cursor_stays_in_the_rows_when_they_get_fewer` (PANES-55),
`a_new_width_or_a_step_freezes_the_new_rows` (PANES-56),
`a_message_that_grows_keeps_its_frozen_rows` (PANES-57),
`a_scrolled_view_keeps_its_rows_when_rows_come_below` (PANES-58).
