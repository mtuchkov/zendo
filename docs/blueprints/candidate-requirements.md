---
status: "draft"
date: "2026-11-19"
prev: "ui-layer.md"
next: "evolution.md"
level: "the candidate requirements of the state and of the UI layer that have an effect on the architecture, one row each"
note: "Initial design blueprint. Agent draft from the statements of the user in chat on 2026-11-19. Not reviewed by the user. Not a source of truth: the specifications in docs/zendo are."
---

# Candidate requirements of the state and the UI layer

Before: [The UI layer](ui-layer.md). Next: [Evolution](evolution.md).

Candidates without IDs, from [State](state.md), [Conflicts](conflicts.md) and
[The UI layer](ui-layer.md). A specification of the state and a specification of the UI layer give
them their IDs and their acceptance tests. These specifications do not exist yet.

| Pattern    | Candidate | Source | Design |
| ---------- | --------- | ------ | ------ |
| Event      | When an event comes late, the system shall apply it only to the owner and the subject that it names. | User, 2026-11-19 | [State](state.md) |
| Unwanted   | If an event conflicts with the state of its owner, then the system shall resolve the conflict with the strategy that the owner states for that kind of event. | User, 2026-11-19 | [Conflicts](conflicts.md) |
| Event      | When the user starts an action that changes a file, the state of zendo or a program, the system shall complete it as the user intended, also when the user leaves the view. | User, 2026-11-19 | [State](state.md) |
| Event      | When the user leaves a view before work that changes only that view has ended, the system shall stop the work and drop its result. | User, 2026-11-19 | [State](state.md) |
| Event      | When a part of the state changes, the system shall draw again each window that shows that part. | User, 2026-11-19 | [The UI layer](ui-layer.md) |
| Ubiquitous | The system shall keep the text of the user: no event shall remove it or put it into another file. | FAIL-3 | [Conflicts](conflicts.md) |
| Event      | When the user starts an action, the system shall show within 100 ms that it took the action. | Design; IO-2 | [State](state.md) |
| State      | While an action has not ended, the system shall show that it is in progress. | Design | [The UI layer](ui-layer.md) |
