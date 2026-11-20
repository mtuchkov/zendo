---
status: "draft"
date: "2026-11-20"
next: "conflicts.md"
level: "the state of zendo: three kinds of state by the place of their truth, and the rules for a change"
note: "Initial design blueprint. Agent draft, written again on 2026-11-20 after the user rejected the content and the structure of the list of points: three kinds of state and five rules. A proposal of the agent, except where a line names the user. The user chose on 2026-11-20 to record it and to keep the open points open, and then chose a plain structure for the model, not signals of reactive_graph. Not reviewed by the user. Not a source of truth: the specifications in docs/zendo are."
---

# State

Next: [Conflicts](conflicts.md).

zendo keeps its state in The Elm Architecture (TEA), as its own code on ratatui and without a
framework (User, 2026-11-19). TEA says who changes the model. It does not say that a large part of
the model is a copy of things outside zendo: the disk, git, the agents, the board. So zendo sorts
its state by the place of the truth.

## Three kinds of state

| Kind | Where the truth is | Examples | What changes it |
| ---- | ------------------ | -------- | --------------- |
| Internal state | The memory of zendo | The text that is not saved, the undo history, the stack of windows, the focus, the commands that wait | Only `update` |
| Copy of external state | Outside zendo | The text of a file on the disk, the file tree, the facts of git, the state of an agent session, the board, the size of the terminal | Only a message from outside |
| Derived result | Nowhere: zendo computes it | The rows of a note, the layout, the mark "not saved" | Nothing: `comemo` keeps it |

The model holds the first two kinds, in the fields of a plain structure (User, 2026-11-20): the
code of `update` shows the way of each change. A copy that zendo did not read yet is "not known",
not a default. The cursor, the scroll and the selection are not in the model: the component holds
them ([The UI layer](ui-layer.md)).

## The rules

- **One writer.** Only `update` changes the model, and it does no IO (IO-1). An effect and a
  message are values: no closure acts on the model when an answer comes, and both can cross a
  socket to the daemon of ADR 0022 later.
- **A message names its subject.** It carries the identifier of a document, a session or a request,
  never "the open document".
- **A copy does not guess.** `update` does not write an expected result into a copy. An action
  that the outside did not confirm is internal state: a command that waits, and its window shows
  it at once (IO-2). No code takes a change back.
- **A write carries its base.** A command is an effect that changes a file or a program. It carries
  the content that it was made from (User, 2026-11-20), and the worker changes nothing when the
  outside differs at the moment of the write. The save does this today. The commands for one
  subject run in the order of the user, one at a time, each to its end, also when its window is
  closed.
- **A query carries a number.** A query is an effect whose answer only one window needs. zendo
  drops each answer that does not have the number of the newest query of its window, and each
  answer for a closed window. Find does this today.

A source that gives its messages in order, such as the journal of the board, needs no base and no
number. A time limit ends the wait, not the work: a query that passes its limit fails with a
message. A write to the disk has no time limit; zendo may write an error to a regular error log
file when a write waits too long (User, 2026-11-20).

## The one conflict

With these rules an answer cannot reach the wrong subject, and an old answer cannot replace a newer
one. One conflict stays: the outside changed below internal state that is not saved, such as a
file that an agent writes while the user edits it. It shows at two moments: a copy gets a new
value, or a write finds another base. At both, the type of the internal state names the strategy,
and code without one does not compile ([Conflicts](conflicts.md)).

## How zendo checks it

- **A reader can follow one key** (User, 2026-11-19). A component is one file, one `enum` lists
  the messages, and `update` has one branch for each message. A debug build writes one line for
  each key, such as `ctrl-s > NotePane > Msg::Save > Effect::Write(a.md) > Msg::Written(a.md)`.
  The rules for the code of a component go to `src/AGENTS.md`.
- **A test gives the answers in each order.** It checks that no text of the user is lost or goes
  into another file, and that in the end each command is done or failed with a message.
