---
status: "draft"
date: "2026-11-19"
parent: "blueprints.md"
prev: "ui-layer.md"
level: "where zendo goes in the next years, what that asks of the design, and the path from the code of today"
note: "Initial design blueprint. Agent draft from the vision of zendo (document of the user, 2026-11-19), ADR 0022 and the code review of pull request #51. Not reviewed by the user. Not a source of truth: the specifications in docs/zendo are."
---

# Evolution

Parent: [Blueprints of zendo](blueprints.md).

## Where zendo goes

From the vision of zendo (document of the user, 2026-11-19) and
[ADR 0022](../architecture/decisions/0022-one-daemon-for-each-user.md):

| Stage | What changes | What it asks of the design |
| ----- | ------------ | -------------------------- |
| Today | One person, one process, some agents on one machine. | A key does not wait for the disk or a program. |
| The daemon | The UI is a client on a socket; the sessions live in the daemon; two UIs can show one repository. | Each answer is a message. Code cannot cross a socket; only values can. |
| Background agents | Agents compare the intent with the code while no member is connected. | Work that changes state does not depend on a view. Its result is state that a UI finds later. |
| A team through git | Most changes come from other members and their agents, late and many at once. | Each result names the exact content that it is true for. A change from outside is an event like the others. |

The last stage gives the strongest reason. The vision promises that a member can trust a change
without reading its code, by a proof that the code agrees with a requirement. A proof for one
content that zendo shows beside another content is false. So a late answer on the wrong subject is
more than a defect of the screen: it breaks what zendo is for.

The vision also says what zendo is not: no server that zendo runs, no plugin interface, no setting
for everything. So the design needs no general interface for others, and the shared state of a
team arrives through git, not through a service.

## What the code of today has

Three shapes of work hold when answers are late:

- a command and an event with the key of their subject: the agent link (`Cmd`, `Event`) and the
  board worker (`Op`, `Answer`);
- a query where only the newest answer counts: find, with a sequence number;
- a text with more than one writer: a document, with loglets and OT
  ([ADR 0014](../architecture/decisions/0014-undo-loglets-with-operational-transformation.md)).

Pull request #51 added a fourth: code that waits for an answer and then acts on what is open
(`when_saved`, `when_loaded`, `after_job`), and a fact with a default while zendo does not know it.
Most of the 42 findings are there. The design keeps the three shapes and removes the fourth.

## The path

A proposal of the agent. Each step can merge by itself.

1. **Stop the freezes.** Split pull request #51: the parts that touch no flow of user data (the
   heartbeat, the clipboard, the input of the terminals, the state of zendo) merge after their own
   findings are fixed.
2. **A spike on one window.** The note pane and its float with `reactive_graph` and `comemo`: what
   it costs, and whether the tests stay deterministic.
3. **The specifications.** One of the state and one of the UI layer, from this blueprint, with the
   IDs of the requirements. From then on they are the source of truth.
4. **Documents as owners.** One text of a document in memory, the disk as a copy, and all changes
   of a file as commands of its document. This removes the findings that lose or misplace text
   (R-1, R-8, R-13, R-17 to R-21, R-28, R-33).
5. **Windows and layers.** The layout as a derived value; the float as a window with its own
   document; no mode and no focus for the whole app.
6. **Jobs and notifications.** One job for all work that runs; loading, notifications and the list
   of background jobs come from it (R-10, R-29, R-30, R-34, R-42).
7. **The test runtime.** Each action against each answer that waits, in both orders (R-4).

Later, each with its own decision: the sessions in the daemon; a repository service that gives the
changes of a pull as one event; the graph of requirements, tests and code, with `salsa` as a
candidate.

## What becomes of the present documents

- [Input and output](../zendo/io.md) keeps where IO runs, the time limits and the sync. The rules
  for late answers go to the specification of the state.
- [ADR 0024](../architecture/decisions/0024-io-on-worker-threads.md) keeps the writer thread, the
  limits, the slow-tick log and the check of the UI code. "Wait, then act" goes.
- [ADR 0025](../architecture/decisions/0025-ui-as-a-view-of-a-core-with-commands-and-events.md) is
  a draft of the agent. The user will revisit it when the design of the state is settled (User,
  2026-11-19).
- [ADR 0008](../architecture/decisions/0008-app-state-in-sub-states.md) stays: its sub-states with
  private fields become the owners.

## Risks

- `reactive_graph` runs effects on the next turn of the runtime. The 100 ms and the tests must
  count that turn.
- Three caches can overlap: the memos of `reactive_graph`, `comemo` and later `salsa`. A rule is
  needed: memos for small values of a window, `comemo` for pure functions that cost time, `salsa`
  only for the graph over a repository.
- The step "documents as owners" changes the editor, the reader and the reload at the same time.
- The cut of markdown into blocks can be wrong at fences, link definitions and lists.
- The two libraries were chosen from their documentation. No code of zendo ran on them yet.
