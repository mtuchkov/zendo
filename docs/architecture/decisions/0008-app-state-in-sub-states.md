---
status: "draft"
date: "2026-11-15"
sources:
  - "The note of the user `docs/notes/2026-11-15-note.md`: Splitting the App struct into sub-states."
  - "../../../AGENTS.md (Code: the limits of a source file and of a function)"
---

# 0008 The state of the app lives in sub-states

## Context

`App` in `src/app/mod.rs` held about 116 public fields, and 36 submodules each added an `impl App`
block. Each function could read and write each field, so the modules gave no isolation. The frame
cache (what the last draw left for the mouse and the keys) was the worst case: `ui` wrote it and
`app` read it, so drawing and key handling depended on each other through fields without an owner.

The split changes no behavior that a user can see. It adds no requirement to a specification.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Keep one `App`; make the fields `pub(crate)` | Smallest change. | Each module still reaches each field. |
| Sub-states with private fields; effects as return values | Each field has one owner; a sub-state has tests without an `App`. | Many changed call sites; an effect type to keep small. |
| Messages between components | Strong isolation. | Much indirection for an app with one thread. |

## Decision

This is a proposal until the user approves it. The second option is chosen: it gives owners to the
fields, and it needs no new runtime structure.

- The app shall keep its state in sub-states. `App` shall hold one field for each sub-state, and
  the state that all of them need: the config, the saved state, the mode, the focus, the keymap.
- A sub-state shall have private fields and shall not see `App`. Its methods shall take what they
  need as arguments.
- IF a method of a sub-state needs a change in another sub-state, THEN the method shall return an
  `Effect`, and `App::apply` shall do it. A sub-state shall change its own fields directly.
- `ui` shall build a new `Frame` in each draw, from the last one, and `App::draw` shall store it.
  Only `ui` shall write the fields of a `Frame`; the app shall read them through methods.
- WHILE `ui` draws, a function of the app that `ui` calls shall not read the stored frame.
- No file in `src/ui/` shall assign to a field of `App`.

The sub-states:

| Sub-state | Holds | File |
| --------- | ----- | ---- |
| `Frame` | The panes, the buttons, the rows of the text, the word count: what a draw leaves. | `src/ui/frame.rs` |
| `Reader` | The read cursor, the selection, the scroll of the note. | `src/app/reader.rs` |
| `Writer` | The draft in the editor, the float, the mark of "say it here", the meta queue. | `src/app/writer.rs` |
| `Pointer` | The mouse selection, the pane edge in a drag, the copied text. | `src/app/pointer.rs` |
| `Composer` | The chat input, the chips, the suggestions, the context of the next question. | `src/app/composer.rs` |
| `Threads` | The chat threads, the transcript cursor, the pinned thread, the chat pane. | `src/app/threads.rs` |
| `Files` | The documents and the open one, the tree, the find panel, the questions about a file. | `src/app/files_state.rs` |
| `Work` | The delta, the visit, the recaps and reviews of the agent. | `src/app/work.rs` |
| `AgentLink` | The handle and the events of the agent, its name, the questions for names. | `src/app/link.rs` |
| `Lists`, `Update`, `KeyHelp` | The notes list and the search; the update check; the help of the keys. | `src/app/` |

Where the result differs from the note, or settles what it left open:

1. **The frame starts from the last one, not empty.** A draw writes only what it shows: write mode
   does not render the read rows, and the keys of the next mode read them. An empty frame in each
   draw changes behavior.
2. **`ui` corrects view state through one method of the sub-state.** The layout is known only in
   the draw, and the rest of the same draw needs the corrected scroll and cursor. So `ui` calls
   `Reader::fit`, `Threads::fit`, `Files::fit_tree`, `Work::fit_delta` and `App::keep_focus_in`.
3. **`Chat` is `Composer` and `Threads`** (decision of the user, 2026-11-15). `ask`, `dispatch` and
   `open_panel` stay on `App`: they join both with the agent and the editor.
4. **`Files` holds the whole group of the note** (decision of the user, 2026-11-15). It does the
   file actions on disk and returns `Flash`, `Deleted`, `Moved`, `MadeFile` or `MadeFolder`. The
   find panel and the held key stay code of `App`.
5. **`Review` of the note is `Work`.** `Review` is already the type of the review of one file.
6. **There is no `Sessions` sub-state.** `cli`, `hold`, `attention` and `music` are types of their
   own with their own tests. A wrapper adds a hop and no boundary.
7. **The word count, the width of the input and the buttons of the sound and the music are part
   of the frame.** The draw computes them.

## Consequences

- The integration tests reach the fields through the sub-states: `a.frame.read_text()`.
- The float (`src/app/float.rs`) swaps the draft, the view, the open document and the pinned
  thread with the screen behind it, each through a swap method of its sub-state.
- Not private yet: the fields of `Transcript`; the tree, the find panel and the worktree popup of
  `Files`, and six fields of `Composer`, inside `src/app/`. `Composer` and `Writer` hand out their
  editor to the keys.
- The effect type can grow into a second `App`. It shall hold cross-state effects only.

## Verification

- `cargo test` passes after each step; the steps change no assertion of behavior.
- `App` has 29 fields, and no field of the frame cache.
- No file in `src/ui/` assigns to a field of `App`.
- Each sub-state has a `*_tests.rs` file that builds it without an `App`.

## Decision record

Proposed by the agent from the note of the user, 2026-11-15. The user decided points 3 and 4 on
2026-11-15, and asked the agent to finish the split on its own. The rest is not reviewed.
