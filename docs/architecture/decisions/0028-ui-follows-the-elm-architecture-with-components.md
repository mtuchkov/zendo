---
status: "draft"
date: "2026-11-20"
note: "Agent draft, not reviewed by the user. The user pasted this text into the chat of zendo session 2 on 2026-11-20 and asked to record it as a file in this branch; the body is as pasted. On 2026-11-20 the user chose this record; the earlier draft of this branch for the same decision (the UI as a view of a core) was removed from the branch at the request of the user. The text was written from `main`: this branch already does IO on worker threads (ADR 0024, ../../zendo/io.md), and ../../zendo/responsiveness.md is not in this branch yet."
sources:
  - "The user, 2026-11-19: a complete refactor of the UI; it freezes on each slow action; it must be reactive and responsive, with events all the way."
  - "The user, 2026-11-19: the Elm architecture is chosen over tui-realm and reactive_graph."
  - "ADR 0008 (sub-states and effects), ADR 0009 (git off the UI thread), ADR 0022 (one daemon for each user)"
  - "../../zendo/responsiveness.md (RESP-1)"
---

# 0028 The UI follows the Elm architecture, with components

## Context

The UI runs on one thread. `tui::run` waits in `select!` for a key, an agent event or a tick, calls
a handler of `App`, and draws. Each handler runs to its end before the next draw. About 70 calls
block that thread: git (`agent/work/recap.rs`, `agent/work/mod.rs`, `app/version.rs`,
`app/worktree.rs`), file reads (`app/files.rs`, `app/chat.rs`, `agent/work/`), the walk in
`app/at.rs`, the clipboard and `claude auth status`. `draw` also renders the markdown of the whole
note in each frame (`ui/read.rs`).

ADR 0009 moved one git call to a thread. Each new feature can add a new blocking call, because no
rule forbids it. ADR 0008 gave each field one owner and made cross-state changes explicit
(`Effect`), but it has no model for work that takes time. Phase 2 (ADR 0022) moves the sessions
into a daemon: the UI then gets its state as messages through a socket.

The decision needed: one model for input, slow work and results, that keeps the UI thread free.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Move each blocking call to a thread, as in ADR 0009 | Small steps; no new structure. | No rule stops new blocking calls; each move invents its own pending state. |
| Own Elm architecture: `Msg`, `update`, `Cmd`, `view`, with components | One writer of the state; slow work is data; messages can cross the socket of phase 2; `update` has tests without a terminal. | Message types to write; a migration of each sub-state. |
| tui-realm | Components, focus and events ready. | Its components do not fit the editor, the `vt100` screens or the delta; each one becomes a wrapper. ADR 0008 rejected messages through a framework. |
| reactive_graph (signals, as Vue and Leptos) | Derived values update by themselves. | A second runtime beside `App`; implicit effects; a signal cannot cross a socket. |
| React-like hooks (as iocraft) | Local state in each component. | A runtime for hooks; ratatui already compares the cells. |

## Decision

Proposal, not yet accepted: the own Elm architecture. From React it takes components with local
state; from Vue it takes cached derived values.

**The loop**

- The UI thread shall own `App`. No other thread shall hold a reference to `App`.
- Keys, agent events, PTY output, results of workers and timers shall arrive as `Msg` values in one
  queue.
- The UI thread shall change `App` only in `update`, for one `Msg` at a time.
- `update` shall not block: it shall not start a process and wait for it, read or write a file, or
  wait for a lock that a worker holds.
- When `update` needs work that can take time, it shall return a `Cmd` with all its input data. A
  runner shall do the `Cmd` on a task or a blocking thread and send the result as a `Msg`.
- Each `Cmd` and each result shall name its subject: a document, a session or a request.
- A `Cmd` that only reads shall carry a request number. If a result does not have the newest
  number of its window, then `update` shall drop it.
- A `Cmd` that changes a file or a program shall carry the content that it was made from. The
  runner shall change nothing when the outside differs at the moment of the write. The `Cmd`
  values for one subject shall run one at a time, in the order of the user, each to its end.
- `Msg` and `Cmd` shall hold data, not references, so that they can cross the socket of phase 2.
- The loop shall take messages until the queue is empty or 12 ms have passed, and then draw once.

**Components**

- Each pane shall be a component with its own UI-local state: scroll, cursor, selection, open items.
- A global key shall always go to `App` first, not to a window. Each other key shall go to the top
  window of the stack of windows: the panes are the bottom layer, and each floating window is a
  layer above. A window shall pass a key that it does not use to the window below.
- A component shall get domain data from its parent as a reference (props) in `update` and `view`.
- A component shall change only its own state. It shall return events for its parent (`Out`) and
  `Cmd` values. It shall not read or change the state of another component.
- `Out` and `Cmd` take the place of `Effect` of ADR 0008.

**State**

- Services shall own the resources: PTYs, `vt100` screens, agent processes, git, the walk and the
  index of the find panel. `App` shall hold a projection of their state that `update` changes from
  their messages.
- The PTY service shall give the UI a read-only handle of each screen behind a trait. In phase 2 the
  trait reads a copy that the daemon keeps current with messages.
- `App` shall own the text of each open document and its undo history. A file service shall read
  and write the disk. `App` shall hold a copy of what the disk has, and the copy is "not known"
  until the first answer.
- When the saved state of zendo (`.zendo/db.json`) changes, `update` shall return a save `Cmd`
  with a delay of at most 1 s. The runner shall write it with `zendo::atomic`. IO-4 and IO-7 of
  [Input and output](../../zendo/io.md) say what becomes of a write that waits.

**The view**

- `view` shall read `App` and shall not change it. It shall leave no record of the frame: the
  `Frame` of ADR 0008 goes.
- A click of the mouse shall go to the top window of the stack under the pointer. Each component
  shall have one function that computes the places of its buttons and its rows; `view` and the
  mouse shall both ask it.
- `App` shall hold the size of the terminal, from `Msg::Resize`.
- Derived values (the layout of the windows, markdown, wrapped lines, highlight) shall be computed
  when they are asked for, and cached with `comemo`, which keeps a result while what the function
  read is the same. `update` and `view` shall ask for the same values.
- After `update` and before `view`, the loop shall fit the scroll and the cursor of each component
  on the screen to its rectangle and its wrapped lines, in one step. The `fit` methods of ADR 0008
  move into this step.

## Consequences

- ADR 0008 is partly superseded: the sub-states stay; `Effect`, the `Frame` and the `fit` calls
  from `ui` go.
- ADR 0009 becomes one case of this model; its pending answer becomes a request number.
- The migration goes in steps. The milestone
  [The UI on the Elm architecture](../../milestones/ui-elm-architecture.md) holds the steps.
- Phase 2 moves the services and the runner behind the socket. `update` and `view` do not change.
- Each new kind of slow work needs a `Cmd` and a result `Msg`: more code than a direct call.

## Verification

The milestone [The UI on the Elm architecture](../../milestones/ui-elm-architecture.md) names the
checks of this decision.

## Decision record

Proposed by the agent on 2026-11-20 from the request of the user of 2026-11-19. The user chose the
Elm architecture on 2026-11-19. Not yet approved.
