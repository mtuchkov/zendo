---
status: "draft"
date: "2026-11-19"
prev: "state.md"
next: "ui-layer.md"
level: "where a conflict shows in zendo, the strategies that resolve it, what a change is based on, and how the rules are enforced"
note: "Initial design blueprint. Agent draft from the statements of the user in chat on 2026-11-19. Not reviewed by the user. Not a source of truth: the specifications in docs/zendo are."
---

# Conflicts

Before: [State](state.md). Next: [The UI layer](ui-layer.md).

A **conflict**: an event was made for a state that is not the state of its owner now. Conflicts
will happen (User, 2026-11-19). The design does not prevent them; it gives each one a place and a
strategy.

## Where a conflict shows

A conflict shows where an effect becomes visible. There are two kinds of place:

- an event enters the state of an owner, and the screen shows it;
- a command reaches the disk, a program or another process, and others see it.

An owner that takes one consistent stream has no such place for that stream: its sender resolved
the order before ([State](state.md)).

zendo takes no lock between processes for a save (User, 2026-11-20). An agent and git do not know
a lock of zendo, so a lock protects only against a second zendo.

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

The entity with one strategy depends on what the window works with (User, 2026-11-20): for the
note pane it is the document, whose text merges (ADR 0014); for the file tree it is the state of
the tree; for the delta of a worktree it is the state of git.

## What a change is based on

To detect a conflict, a change must say what it was made for. Three forms, from the one with the
fewest conflicts:

1. **The operation.** "Toggle the box of this task", "put this title". The owner applies it to its
   present state. Two operations merge.
2. **A copy of the exact piece of state** that the change replaces or was computed from, or its
   hash. The change holds while that piece is the same.
3. **A version** of the piece or of a larger entity. It is small to pass. A version of a large
   entity reports a conflict for each change of the entity, also one that does not matter.

The user chose on 2026-11-20: a write carries the content that it was made from, and a read
carries a request number ([State](state.md)).

## How the rules are enforced

The user asked that a strategy at each such place is enforced, not a habit.

- The type of each kind of event names its strategy. Code that applies an event without one does
  not compile.
- The fields of an owner are private (ADR 0008): the owner is the only gate to its state.
- The work of commands and queries runs on a runtime that a test replaces. A generated test runs
  each action of the user against each answer that waits, in both orders, and checks the rules
  that hold always ([State](state.md)).
- A check in `cargo test`: no part of the state holds code that acts on the app later.
