---
status: "approved"
approved: "2026-11-19"
date: "2026-11-19"
sources:
  - "docs/zendo/panes/transcript-selection.md (PANES-50 to PANES-58)"
  - "docs/zendo/chat.md (CLAUDE-29)"
  - "docs/zendo/panes/selection.md (PANES-3, PANES-13)"
  - "The design of the user in the chat on 2026-11-19, after the panic at src/ui/thread.rs:293"
---

# 0026 Selection over frozen transcript rows

## Context

zendo stopped with a panic at `src/ui/thread.rs:293`: the cursor of the transcript was on row 16
of a transcript of 6 rows. The cursor and the selection anchor of `Transcript` are row numbers of
the last frame, and each frame builds the rows again. The rows move under the numbers when the
status rows at the end come and go, when another thread shows, when the width wraps the text
again, when `o` opens or closes a step, when a tool call changes its status, and when an item goes
in above (`resumed the earlier thread`, replayed messages).

The mouse selection of the chat pane is a second model: `text::select::Selection` keeps screen
cells and reads the text from the frame buffer, so it cannot reach rows out of view.

The decision needed: what the cursor and the selection of the transcript point at, so that they
always point at the text that the user chose.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| A. Clamp the cursor into the rows on each frame | Small; removes the panic. | The cursor and the selection still point at other text after a change above them; the cursor lands on a blank row at the end. |
| B. Positions in the content (item, char in the item), mapped to rows on each frame, live view | The view stays live during a selection. | Each frame maps positions to rows; inserts above need index shifts; the selection can scroll away while the user drags. |
| C. Freeze the rows while the transcript has the keys or a selection; new output below, with `↓ n new rows · End` | Row numbers stay valid with no mapping; one model for mouse and keys; the highlight always matches the text. | The view does not follow the agent during a selection; width changes and `o` need a new freeze. |
| D. Keep the selection of screen cells for the mouse | No change. | Bound to the view; two models for one thing. |

## Decision

Option C, chosen by the user on 2026-11-19.

`Transcript` shall own the frozen rows while it has the keys or a selection, and the cursor and
the anchor shall be positions in those rows. The draw shall show the frozen rows, and the count of
rows that the live thread has below them. The mouse selection in the chat transcript shall set the
anchor and the cursor of `Transcript`, and copy from its rows, not from the frame buffer. The tick
of the event loop shall scroll the transcript while a drag holds the pointer at the edge of the
pane.

## Consequences

- `src/app/frozen.rs`: the frozen rows, the count of new rows, the thread key and the width that
  they belong to; `Threads` owns them, and `Transcript` gets the rows that the draw shows.
- `src/ui/chat.rs`: the draw shows the frozen rows, and the `↓` row.
- `src/app/mouse.rs`, `src/app/pointer.rs`, `src/app/transcript_mouse.rs`: a drag in
  `Region::Panel` over the transcript goes to `Transcript`; the note, the delta and the terminal
  view keep the selection of screen cells.
- `src/app/threads.rs`: the scroll of the view stays an offset from the bottom; while the view is
  not at the end, the offset grows and shrinks with the rows below, so they do not move it
  (PANES-58).
- The tick of 100 ms in `src/tui/mod.rs` drives the scroll at the edge.
- The clamp of option A stays as a guard in `show_rows`: a frozen transcript does not get fewer
  rows, but a new freeze can.
- Limitation: during a long selection, the user sees new output only as a count.

## Verification

The acceptance tests of [Selection in the transcript](../../zendo/panes/transcript-selection.md):
`a_selection_in_the_transcript_reaches_rows_out_of_view`, `new_output_waits_below_a_selection`,
`a_shorter_thread_starts_the_transcript_over`.
