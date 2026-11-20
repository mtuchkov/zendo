---
status: "draft"
date: "2026-11-20"
version: "0.121.21"
parent: "../zendo/zendo.md"
level: "the scope, the temporary behavior, and the acceptance of one milestone"
note: "Agent draft, written on 2026-11-20 at the request of the user as the handoff of the design task of the state and the UI. It delivers no specification, so it has no `spec` field. The order of the next steps is a proposal of the agent. Not reviewed by the user."
---

# Milestone: handoff of the design of the state and the UI

This milestone ends the design task of 2026-11-19 and 2026-11-20: how zendo manages its state, and
which model the UI follows. It hands the work to the session that continues it. The design is on
the branch `docs/io-spec`. The user closed pull request #51 of this branch on 2026-11-20: the
work is not ready. All documents of the design are drafts: the user gave the decisions in chat,
and did not review the documents.

## Scope

In scope, and done:

- [ADR 0028](../architecture/decisions/0028-ui-follows-the-elm-architecture-with-components.md):
  the UI follows the Elm architecture, with components. It holds the decisions of the user.
- The blueprints, a chain of five documents: [State](../blueprints/state.md),
  [Conflicts](../blueprints/conflicts.md), [The UI layer](../blueprints/ui-layer.md),
  [Candidate requirements](../blueprints/candidate-requirements.md) and
  [Evolution](../blueprints/evolution.md). They have no open point.
- The milestone [The UI on the Elm architecture](ui-elm-architecture.md): the steps of the
  migration of the code, and its checks.

Out of scope:

- The migration of the code: the milestone [The UI on the Elm architecture](ui-elm-architecture.md).
- The specifications of the state and of the UI layer, with the IDs of their requirements. No
  milestone names them yet.
- Requirements for the blocks of a document. The user stopped them on 2026-11-20; the words of the
  user are in [The UI layer](../blueprints/ui-layer.md).

## The next steps

A proposal of the agent, in this order:

1. The user reviews ADR 0028 and the blueprints, and approves or changes them.
2. Merge `main` into `docs/io-spec`, and set a version above `main`.
3. Write `src/AGENTS.md`: the rules for the code of a component ([State](../blueprints/state.md)).
4. Write the specifications of the state and of the UI layer from the blueprints.
5. Start step 1 of the migration.

## Acceptance

The milestone is accepted when the user approves ADR 0028. In addition, `cargo test --test
project_structure --test traceability` passes on the branch.

## Open items

| Open item | Affected | Needed from |
| --------- | -------- | ----------- |
| `docs/io-spec` is 16 commits behind `main`. `main` has version 0.126.1; the branch has 0.121.21. `docs/zendo/responsiveness.md` (RESP-1), which ADR 0028 names, comes with `main`. | The branch | The agent, before a new pull request |
| `0024-io-on-worker-threads.md` has the number of an ADR of `main`. 16 files of the branch name "ADR 0024", in `docs`, `src` and `tests`. | ADR 0024 of the branch | The user: the new number |
| Pull request #51 is closed, and the branch stays. It held the code that moves IO to workers, and this design. [Evolution](../blueprints/evolution.md) proposes to split that code into parts. | A new pull request | The user: when the work is ready |
| ADR 0028 has about 1,200 words. The limit is about 1,000. | ADR 0028 | The user: what to cut |
| [Conflicts](../blueprints/conflicts.md), [Candidate requirements](../blueprints/candidate-requirements.md) and [Evolution](../blueprints/evolution.md) still use "owner" and "event", the words of before the Elm architecture. | The blueprints | The agent |
| A write to the disk that waits too long may go to a regular error log file (User, 2026-11-20). zendo has no such file, and no requirement asks for it. | [Input and output](../zendo/io.md) | The user |
| `docs/notes/2026-11-16-ui-layers-and-pane-views.md` is in the worktree `.worktrees/2` and not in git. | The note | The user |

## Completion record

2026-11-20, commit `f362187`, version 0.121.20: the design documents. `cargo clippy --all-targets
-- -D warnings` was clean, `cargo nextest run` passed 993 tests and skipped 3, and `cargo test
--doc` passed. No terminal check and no check with real Claude ran: the commit changed no code.
The user did not review the documents.
