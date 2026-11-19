---
status: "draft"
date: "2026-11-19"
note: "Agent draft from the code review of pull request #51 and the statements of the user in chat on 2026-11-19. Not reviewed by the user. On hold: the user asked on 2026-11-19 to revisit it when the design of the state is settled (docs/blueprints/state.md). It was written before the blueprints and does not yet have their principles: owners with a stated way to stay consistent, a strategy at each place where a conflict shows, and the UI layer of zendo."
sources:
  - "The user, 2026-11-19, in chat: the eventing mechanism of zendo shall guarantee correctness without unnecessary blocking; an abandoned action is cancelled only when its result changes nothing but the view; an operation that changes state completes as intended; the design shall be future proof."
  - "The vision of zendo (document of the user, 2026-11-19); the code review of pull request #51; ADR 0008, 0014, 0022, 0024; ../../zendo/io.md"
---

# 0025 The UI is a view of a core that it reaches with commands and events

## Context

ADR 0024 moved IO to worker threads. Before it, the order of the calls on one thread kept the state
correct: "save, then open the float" could not interleave. Now an answer comes later, after the
user did other things. The review of pull request #51 found 27 defects. Most have one cause: an
answer acts on what is open when it comes, or an action runs before its answer.

zendo already has three shapes of work that hold when answers are late:

- A command and an event with the key of their subject: the agent link and the board worker.
- A query where only the newest answer counts: find, with a sequence number.
- A text with more than one writer: a document, with loglets and OT (ADR 0014).

ADR 0024 added a fourth: a closure that waits for an answer and then acts on the state of the app
(`when_saved`, `when_loaded`, `after_job`), and a fact with a default while zendo does not know it.
The defects are there. The direction of zendo makes this shape a dead end:

- ADR 0022 splits zendo into a daemon and UI clients. A closure cannot cross a socket.
- The vision has background agents that work while no member is connected: work that changes
  state cannot depend on a view.
- The vision has a proof that the code agrees with a requirement. A proof is true for one exact
  content: each answer must name what it is true for.

The decision needed: one design for late answers that holds for the daemon, for work without a
view, and for a team.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Fix each call site | Smallest change. | No guarantee; each new feature can make the same defect. |
| A scope and a cancellation for each closure | Answers of a dead scope are dropped. | Each caller must follow the rule; one file keeps two writers; no closure crosses a socket. |
| A core and a view; commands, events and queries as plain data | The three shapes that hold, for all work; the messages cross a socket later; a test delivers answers in each order. | The save path and each closure change, in steps. |
| The daemon of ADR 0022 first | The socket forces messages. | Too large now; that daemon holds no documents. |

## Decision

Proposal, not yet accepted: the third option.

- **Core and view.** The core shall own the state and all IO. The UI thread shall take events into
  the model, draw, and send commands; it shall do nothing else.
- **Plain data.** A command, an event and a query shall be values that zendo could write to a
  journal or a socket. Each shall name its subject (a document, a session, a view) and the version
  that it started from. No closure shall wait for an answer.
- **Commands change state.** The owner of a command is the thing that it changes. The commands of
  one owner shall run one at a time, in the order of the user, each to its end or to a failure
  with a message, also when the user left the view or closed the UI.
- **Queries change only the view.** The owner of a query is the view. Only the newest answer
  counts. When the view goes or asks again, the core shall drop the answer and stop the work. Each
  query has a time limit.
- **A document owns its file.** Each change of a file by zendo (save, title, front matter, task
  box, rename) shall be a command of its document. The editor, the float and the note pane are
  views of documents.
- **Not known is a state.** An action that needs a fact that the core did not read yet shall wait
  for the fact; the view shall not use a default.
- **Versions by content.** A result that zendo derives (a delta, a recap, a link from a requirement
  to a test) shall name its inputs by a hash of the text or a git object. zendo shall show it only
  for that content.

This record replaces in ADR 0024: "wait, then act" with closures, `Serial` for the files of notes,
and the defaults of `Memo`. It keeps the writer thread, the time limits, the slow-tick log and the
check of the UI code.

## Consequences

- Step 1: documents with an identity and one queue for each file; the answers of saves are events
  (R-1, R-8, R-13, R-17 to R-21). Step 2: one query type with a sequence number, a stop signal and
  a limit; facts that can be "not known" (R-2, R-3, R-7, R-10, R-25, R-27). Step 3: one channel of
  events into the main loop, and a runtime that a test replaces.
- Pull request #51 can be split: the parts that touch no flow of user data (the heartbeat, the
  clipboard, the input of the terminals, the state of zendo) stop the freezes and can merge first.
- The effect type of ADR 0008 stays for changes between sub-states. ADR 0008 rejected messages "for
  an app with one thread"; off the UI thread that reason ended.
- The board gets the version rule: R-23 shows that a key without a version is not enough.
- Later, each with its own decision: the changes of a pull as one event, the sessions in the
  daemon, the graph of requirements and code, a crate for the UI that cannot reach the file system.

## Verification

- The six probes of the review become tests, and pass.
- A test delivers the answers of the disk to the save path in each order: each file has the text
  of its own document, as with a disk that answers at once.
- A check in `cargo test`: `App` has no field that holds a closure over `App`.

## Decision record

Agent draft of 2026-11-19, from the statements of the user in chat and the vision of zendo. Not
reviewed.
