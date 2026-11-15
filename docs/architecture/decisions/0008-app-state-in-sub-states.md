---
status: "draft"
date: "2026-11-15"
sources:
  - "The note of the user `docs/notes/2026-11-15-note.md`: Splitting the App struct into sub-states."
  - "../../../AGENTS.md (Code: the limits of a source file and of a function)"
---

# 0008 The state of the app lives in sub-states

## Context

`App` in `src/app/mod.rs` holds about 116 public fields, and 36 submodules each add an `impl App`
block. Each function can read and write each field, so the split into modules gives no isolation.
Three kinds of state share the struct: domain state (the open note, the threads, the agent
sessions), view state (cursors, scroll positions, the selection), and the frame cache (what the
last draw left for the mouse and the keys). The frame cache is the worst case: `ui` writes it and
`app` reads it, so drawing and key handling depend on each other through fields without an owner.

The split changes no behavior that a user can see. It adds no requirement to a specification.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Keep one `App`; make the fields `pub(crate)` | Smallest change. | Each module still reaches each field; the cycle between `ui` and `app` stays. |
| Sub-states with private fields; effects as return values | Each field has one owner; a sub-state has tests without an `App`; the compiler checks the boundaries. | Many changed call sites, most of them in tests; an effect type to keep small. |
| Messages between components (a reducer for each sub-state) | Strong isolation; each change of state is one message. | Much indirection for an app with one thread; each key becomes a message type. |

## Decision

This is a proposal until the user approves it. The second option is chosen: it gives owners to the
fields at the cost of mechanical changes, and it needs no new runtime structure.

- The app shall keep its state in sub-states: `Reader`, `Writer`, `Composer`, `Threads`,
  `AgentLink`, `Sessions`, `Files`, `Review`, `Frame` and `Pointer`. `App` shall hold one field for
  each sub-state, and the state that all of them need: the config, the mode, the focus, the keymap.
- A sub-state shall have private fields and shall not see `App`. Its methods shall take what they
  need as arguments.
- IF a method of a sub-state needs a change in another sub-state, THEN the method shall return an
  effect, and `App` shall apply it. A sub-state shall change its own fields directly.
- `ui` shall build a new `Frame` in each draw, from the last one, and `App` shall store it. Only
  `ui` shall write the fields of a `Frame`; the app shall read them through methods.
- WHILE `ui` draws, the app has no frame. A function of the app that `ui` calls in a draw shall not
  read the stored frame.
- WHEN the split is complete, no file in `src/ui/` shall assign to a field of `App`.

Six points differ from the note:

1. **The frame starts from the last one, not empty.** A draw writes only what it shows: write mode
   does not render the read rows. The keys of the next mode read the values of the last draw that
   had them. An empty frame in each draw changes behavior.
2. **`ui` corrects view state in a draw, through one method of the sub-state.** The layout is known
   only in the draw, and the rest of the same draw needs the corrected scroll and cursor. A draw
   that returns corrections has to keep two copies of these values in step. So `ui` calls a method
   of the sub-state (`Reader::fit`, `Threads::fit`), which changes its own fields.
3. **The word count is part of the frame.** The draw computes it; the status bar and the split
   read it.
4. **`peek` is not view state.** It names the version of the document on the screen (FILES-45,
   FILES-47). It stays on `App` until the step of `Files`.
5. **`Chat` is two sub-states** (decision of the user, 2026-11-15). `Composer` holds what goes
   into the next question: the input, the chips, the attachments, the suggestions, the context.
   `Threads` holds the threads, the transcript cursor, the pinned thread and the scroll of the
   chat pane. `ask`, `dispatch` and `open_panel` stay on `App`: they join the composer, the
   threads, the agent and the editor. So no effect type is needed before `Files`.
6. **`tab_run` stays on `App`, and the width of the input is part of the frame.** The run of
   `tab`s moves the focus between panes; the draw computes the width.

The order of work is one commit for each step, without a change of behavior: `Frame`; `Reader` and
`Pointer`; `Composer` and `Threads`; `Files`, then `Review`; `AgentLink` and `Sessions`.

## Consequences

- The integration tests in `tests/` reach the fields through the sub-states: `a.frame.read_text()`
  in place of `a.read_text`. This is most of the diff.
- `ui::draw` takes the last frame and returns the next one. `App::draw` stores it; the event loop
  and the tests call `App::draw`.
- The float (`src/app/float.rs`) swaps fields of several sub-states with the screen behind it. Each
  sub-state that it touches needs a swap method of its own.
- Each sub-state with an `Instant::now()` default needs its own `Default`.
- The effect type can grow into a second `App`. It shall hold cross-state effects only.
- Some fields (`notes`, `current`, `visit`) are read from several groups. The step of their group
  decides whether they stay on `App` or get a read-only method.

## Verification

- `cargo test` passes after each step; the steps change no assertion of behavior.
- `App` has fewer than 30 fields, and no field of the frame cache.
- No file in `src/ui/` assigns to a field of `App`.
- Each sub-state has a `*_tests.rs` file that builds it without an `App`.
- State of the steps: `Frame` (`src/ui/frame.rs`), `Reader`, `Pointer`, `Composer` and `Threads`
  (`src/app/`) are done, each with a `*_tests.rs` file. `App` has 73 fields. Open: `Files`,
  `Review`, `AgentLink`, `Sessions`, `Writer`; the public fields of `Transcript`.

## Decision record

Proposed by the agent from the note of the user, 2026-11-15. The user decided point 5 on
2026-11-15. The rest is not reviewed.
