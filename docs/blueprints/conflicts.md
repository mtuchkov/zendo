---
status: "draft"
date: "2026-11-19"
parent: "blueprints.md"
prev: "state.md"
next: "ui-layer.md"
level: "where a conflict shows in zendo, the strategies that resolve it, what a change is based on, and how the rules are enforced"
note: "Initial design blueprint. Agent draft from the statements of the user in chat on 2026-11-19. Not reviewed by the user. Not a source of truth: the specifications in docs/zendo are."
---

# Conflicts

Parent: [Blueprints of zendo](blueprints.md). Before: [State](state.md).

A **conflict**: an event was made for a state that is not the state of its owner now. Conflicts
will happen (User, 2026-11-19). The design does not prevent them; it gives each one a place and a
strategy.

## Where a conflict shows

A conflict shows where an effect becomes visible. There are two kinds of place:

- an event enters the state of an owner, and the screen shows it;
- a command reaches the disk, a program or another process, and others see it.

An owner that takes one consistent stream has no such place for that stream: its sender resolved
the order before ([State](state.md)).

## The strategies

Each place where a conflict can show names one strategy of this list. The list is closed: a new
strategy is a change of this design.

| Strategy | Use | Example |
| -------- | --- | ------- |
| Merge | Both changes can hold. | Text: operational transformation (ADR 0014). |
| Do again on the present | The intent holds, its base does not. | The next free name of a new note. |
| The newest wins | Only the last state counts. | The results of find; the heartbeat. |
| Ask the user | zendo cannot choose, and text can be lost. | The undo of a change from outside (UNDO-12). |
| Fail and change nothing | No other strategy is safe. | A write that the disk refuses (FAIL-2). |

"The newest wins" is never the strategy for the text of the user.

## What a change is based on (open)

To detect a conflict, a change must say what it was made for. Three forms, from the one with the
fewest conflicts:

1. **The operation.** "Toggle the box of this task", "put this title". The owner applies it to its
   present state. Two operations merge.
2. **A copy of the exact piece of state** that the change replaces or was computed from, or its
   hash. The change holds while that piece is the same.
3. **A version** of the piece or of a larger entity. It is small to pass. A version of a large
   entity reports a conflict for each change of the entity, also one that does not matter.

The user left the choice between a copy of the piece and a version of a smaller entity open on
2026-11-19. What `reactive_graph` gives for it is to be looked at.

## How the rules are enforced

The user asked that a strategy at each such place is enforced, not a habit.

- The type of each kind of event names its strategy. Code that applies an event without one does
  not compile.
- The fields of an owner are private (ADR 0008): the owner is the only gate to its state.
- The work of commands and queries runs on a runtime that a test replaces. A generated test runs
  each action of the user against each answer that waits, in both orders, and checks the rules
  that hold always ([State](state.md)).
- A check in `cargo test`: no part of the state holds code that acts on the app later.

## Open points

| Open question | Affects |
| ------------- | ------- |
| A copy of the piece of state, or a version of a smaller entity? | What a change is based on |
| Which is the smallest entity with one strategy: a document, a block of it, a field? | The strategies |
| Does the check of a save need a lock between processes, for another zendo or an agent that writes between the check and the write? | Where a conflict shows |
