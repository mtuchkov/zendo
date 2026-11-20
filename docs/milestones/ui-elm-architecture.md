---
status: "draft"
date: "2026-11-20"
parent: "../zendo/zendo.md"
spec: "../zendo/io.md"
level: "the scope, the temporary behavior, and the acceptance of one milestone"
note: "Agent draft. The user asked on 2026-11-20 to move the steps of the migration and the checks from ADR 0028 to a milestone document, because they are implementation details. The steps and the checks are those of ADR 0028, as they were on 2026-11-20. No version is set: each step ships in its own version. Not reviewed by the user."
---

# Milestone: the UI on the Elm architecture

This milestone moves the UI of zendo to the model of
[ADR 0028](../architecture/decisions/0028-ui-follows-the-elm-architecture-with-components.md): one
queue of messages, one `update`, work that takes time as `Cmd` values, components, and a view that
changes nothing. The user gets a UI that answers each key while the disk or a program is slow
(IO-1, IO-2 of [Input and output](../zendo/io.md)).

## Scope

In scope: the migration of the code, in these steps. Each step passes the tests and gets its own
version.

1. The `Msg` queue and the runner; the arms of `select!` become sources of `Msg`.
2. The blocking calls of the context move to `Cmd`: git first, then files, clipboard, walk.
3. The cache of the rendered markdown.
4. The sub-states become components, one in each step, each with the places of its buttons and
   rows; `Effect` and the `Frame` go after the last one.
5. The `fit` methods move into the step between `update` and `view`.

Out of scope: phase 2 of ADR 0022, which moves the services and the runner behind the socket.

## Temporary behavior

Until step 4 ends, the code has both models: the sub-states with `Effect` and the `Frame` of ADR
0008, and the components. No requirement is temporary.

## Acceptance

The milestone is accepted when this acceptance test of the specification passes:
`ui_code_does_no_io` in `tests/ui_thread_io.rs`. It fails when a file of `src/app`, `src/ui` or
`src/tui` calls file, program or system IO directly. A list in the test names each allowed file
and its reason. The slow-tick log of debug builds finds the calls that are not direct (IO-8).

In addition:

- An integration test with a mock git that waits 5 s: keys typed during the wait draw on the
  screen before git answers.
- A test that sends the result of a read with an old request number: the state does not change.
- A test that changes a file outside zendo between a save and its write: the runner changes
  nothing, and the text from outside is not lost.
- A test that makes the terminal smaller: before any draw, the cursor of each component is inside
  its rectangle.
- A test that clicks, with no draw before it, a point where a floating window covers a button of
  a pane: the floating window gets the click, and the button does nothing.
- The telemetry log (RESP-1): the time of each action before and after the migration, for the
  actions that ran git or read files.

## Open items

| Open item | Affected | Needed from |
| --------- | -------- | ----------- |
| The version of each step | The front matter | The agent, at each step |
| The time of a full frame on the note pane and its float, and whether the tests stay deterministic | Step 4 | The agent, with the first component |
| No specification of the state and of the UI layer exists yet; the milestone names [Input and output](../zendo/io.md) and ADR 0028 | Scope, Acceptance | The user |

## Completion record

Not completed.
