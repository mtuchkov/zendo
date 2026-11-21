---
status: "draft"
date: "2026-11-21"
note: "Draft. The decision is of the user (2026-11-20); the record, its options and its consequences wait for the review of the user."
sources:
  - "The user, 2026-11-20: the TUI follows The Elm Architecture; all IO is asynchronous and never blocks the UI."
  - "The user, 2026-11-20: docs/book is the target architecture; specifications take the place of docs/architecture/tui-architecture."
  - "The user, 2026-11-21: the book takes priority over an older specification or decision; this is a redesign."
  - "../../book/zendo-book.md"
  - "../../zendo/internals/internals.md"
  - "../../zendo/responsiveness.md (RESP-1, RESP-2), ../../zendo/failures.md (FAIL-1, FAIL-2)"
  - "ADR 0008, ADR 0009, ADR 0017, ADR 0024, ADR 0026"
---

# 0028 A core that decides and a runtime that does

## Context

ADR 0008 divided the state of the app into sub-states with private fields. A sub-state returns an
`Effect` for a change in a different sub-state, and the draw leaves a `Frame` that the keys and the
mouse read. ADR 0008 rejected messages between components: "much indirection for an app with one
thread".

Since then, each defect of responsiveness got its own mechanism:

- git of the tree runs on a thread, and the tick takes the answer (ADR 0009);
- the find panel has one worker thread, and drops results with an old number (ADR 0017);
- the telemetry log writes in a thread of its own (TELEM-5);
- the cursor of the transcript pointed at rows of the last frame, and zendo stopped with a panic
  (ADR 0026).

These mechanisms have three causes in the code before this decision:

- IO runs where the code needs its result, also on the UI thread. Nothing in the structure tells
  which call can wait.
- The draw corrects state (`Reader::fit`, `Threads::fit`), and the frame starts from the last one.
  So the state is correct only after a draw.
- Each feature that goes off the UI thread builds its own thread, its own numbers for late
  answers, and its own poll.

The decision needed: one structure for the state, for IO, and for answers that come late.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Keep the sub-states of ADR 0008; move each slow call off the UI thread when it shows | No large change. | A mechanism for each case; a new feature can put IO on the UI thread again; the frame cache stays. |
| The Elm Architecture: a core without IO, a runtime that does commands and subscriptions | The UI loop cannot wait for IO, by construction. One mechanism for late answers. The core has tests with values only. | A move of all of the TUI code. One more wrapper for each message and each level of components. A slow `update` is still possible. |
| An actor for each component, with a thread and a queue of its own | Strong isolation; no component stops a different one. | The state is in many places; the order of events between actors is not fixed, so a replay is not deterministic. |

## Decision

The user decided on 2026-11-20 that the TUI follows The Elm Architecture: the second option. This
record is the draft of that decision.

- The TUI shall be a core and a runtime. The core shall hold the state and decide; it shall do no
  IO and shall read no clock. The runtime shall do all IO and shall make no decision.
- Each event shall come into the core as a message, and each effect shall leave it as a command.
- The UI loop shall wait only for messages and for the frame timer.
- The specifications in [Internals of the TUI](../../zendo/internals/internals.md) shall state the
  rules, one document for each chapter of the [zendo book](../../book/zendo-book.md). The book
  explains them.

## Consequences

- The book takes priority over an older decision (User, 2026-11-21). This record takes the place
  of ADR 0008 when the user approves it. The `Effect` type becomes the
  out-message of a component (COMP-3). The `Frame` sub-state goes: `layout` gives the sizes to
  `update` and to `view` (LAYOUT-2), and the hit map of the runtime takes the mouse (MOUSE-1).
- The decisions of ADR 0009 and ADR 0017 for the user stay: git and the search do not stall the
  UI, and the search uses the ripgrep crates. Their threads and polls become commands and
  subscriptions (ASYNC-3, SUB-2), and the number of a query becomes a ticket (TICKET-1).
- The hook of ADR 0024 moves into the runtime, which sees each message (BUDGET-2). RESP-1 and
  RESP-2 do not change.
- The frozen rows of ADR 0026 stay state of the model.
- The writes of user data get a queue for each path, and a write in place: see ADR 0029.
- The move of the code needs a milestone document; it is not a part of this change (User,
  2026-11-21). Until the code moves, no test names the requirements of the internals.
- Limitation: the budget of 8 ms finds a slow `update` by measurement; nothing prevents it
  (BUDGET-1).

## Verification

The acceptance tests of the internals specifications. The main ones:

- `the_core_uses_no_io` (CORE-1): no file of the core uses the file system, the network, a clock,
  a process or tokio.
- `a_replayed_log_gives_the_same_state` (MSG-4).
- `a_key_is_handled_while_a_command_runs` (ASYNC-1) and
  `keys_change_the_screen_while_services_are_slow` (BUDGET-1).
- `a_late_answer_of_an_old_request_does_not_show` (TICKET-1).

None is written yet.
