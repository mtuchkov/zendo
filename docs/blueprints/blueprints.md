---
status: "draft"
date: "2026-11-19"
children:
  - "state.md"
  - "conflicts.md"
  - "ui-layer.md"
  - "evolution.md"
level: "the design of zendo at the top: its layers, its principles, and the candidate requirements with an effect on the architecture"
note: "Initial design blueprint. Agent draft from the statements of the user in chat on 2026-11-19, the vision of zendo, and the code review of pull request #51. Not reviewed by the user. Not a source of truth: the specifications in docs/zendo are."
---

# Blueprints of zendo

A blueprint is a first design of zendo or of one of its parts, written before the specifications
exist. It is not a source of truth: the specifications in [docs/zendo](../zendo/zendo.md) are, and
a decision is in a decision record in `docs/architecture/decisions`. A blueprint can state an
architecturally significant requirement as a candidate, without an ID; a specification gives it
its ID. A blueprint has no detailed requirements.

| Document | Topic |
| -------- | ----- |
| [State](state.md) | Owners, events, commands and queries |
| [Conflicts](conflicts.md) | Where a conflict shows, the strategies, how they are enforced |
| [The UI layer](ui-layer.md) | Windows and layers, what a view shows, jobs and notifications |
| [Evolution](evolution.md) | Where zendo goes, and the path from the code of today |

## Why this design

Pull request #51 moved IO off the UI thread, so that a key does not wait for the disk. It kept the
model of one thread: an answer that comes later acts on what is open at that time. Three reviews
found 42 defects in the code, most with this one cause. The user then stated what the design must
give (User, 2026-11-19). This blueprint is the design that follows from it.

## The layers

1. **State.** Each part of the state has one owner. An owner takes events and changes its state.
   Work that needs time runs outside the owner and comes back as an event. See [State](state.md)
   and [Conflicts](conflicts.md).
2. **The UI layer.** zendo has its own UI above ratatui: windows in layers, focus, loading,
   notifications and background jobs. A window knows which part of the state it shows. ratatui is
   the canvas. See [The UI layer](ui-layer.md).
3. **Input and output.** Where IO runs, its time limits and its sync to the disk:
   [Input and output](../zendo/io.md).

## Principles

From the statements of the user on 2026-11-19; the wording is of the agent.

1. **Eventual consistency is everywhere.** Memory, the screen, the disk, programs and other
   processes hold copies that come to agree. No part acts as if it sees the present.
2. **One owner for each complex part.** The owner takes all events for its part. It is a funnel.
3. **An owner states how it stays consistent.** Either it takes events of more than one sender,
   detects each conflict and has a strategy for it. Or it takes one stream that is consistent
   already, and the one sender of that stream is responsible for the order.
4. **A conflict is resolved where an effect becomes visible.** Each such place has a strategy, and
   the code cannot leave it out.
5. **An action that changes state completes as the user intended.** Work whose result changes only
   a view is cancelled when the user leaves the view.
6. **zendo takes an action at once and completes it in its own time.** No rule gives a time for
   the end.
7. **A visible part knows what it shows.** Its owner draws it again when that part of the state
   changes. Not each small element has a notice of its own.
8. **No action waits for work that it does not need.**

Open: how a change names what it is based on, a version or a copy of the piece of state. See
[Conflicts](conflicts.md).

## Candidate requirements with an effect on the architecture

Candidates without IDs. A specification of the state and a specification of the UI layer give them
their IDs and their acceptance tests.

| Pattern    | Candidate | Source |
| ---------- | --------- | ------ |
| Event      | When an event comes late, the system shall apply it only to the owner and the subject that it names. | User, 2026-11-19 |
| Unwanted   | If an event conflicts with the state of its owner, then the system shall resolve the conflict with the strategy that the owner states for that kind of event. | User, 2026-11-19 |
| Event      | When the user starts an action that changes a file, the state of zendo or a program, the system shall complete it as the user intended, also when the user leaves the view. | User, 2026-11-19 |
| Event      | When the user leaves a view before work that changes only that view has ended, the system shall stop the work and drop its result. | User, 2026-11-19 |
| Event      | When a part of the state changes, the system shall draw again each window that shows that part. | User, 2026-11-19 |
| Ubiquitous | The system shall keep the text of the user: no event shall remove it or put it into another file. | FAIL-3 |
| Event      | When the user starts an action, the system shall show within 100 ms that it took the action. | Design; IO-2 |
| State      | While an action has not ended, the system shall show that it is in progress. | Design |

## Libraries

Chosen by the user on 2026-11-19: `reactive_graph` for the state that windows watch, and `comemo`
for results that zendo derives. zendo builds its own UI layer; `ratatui-reactive` is a source of
ideas only. `salsa` is a candidate for later.

## What depends on this blueprint

- A specification of the state and one of the UI layer, in `docs/zendo`. They do not exist yet.
- [ADR 0025](../architecture/decisions/0025-ui-as-a-view-of-a-core-with-commands-and-events.md), a
  draft of the agent: the user will revisit it when the design of the state is settled.
- [ADR 0024](../architecture/decisions/0024-io-on-worker-threads.md) and
  [Input and output](../zendo/io.md) keep where IO runs.
