---
status: "draft"
date: "2026-11-19"
parent: "blueprints.md"
next: "conflicts.md"
level: "how the state of zendo changes: owners, events, the two ways to stay consistent, commands and queries"
note: "Initial design blueprint. Agent draft from the statements of the user in chat on 2026-11-19. Not reviewed by the user. Not a source of truth: the specifications in docs/zendo are."
---

# State: owners and events

Parent: [Blueprints of zendo](blueprints.md). Next: [Conflicts](conflicts.md).

## Terms

- The **state** is what zendo knows in memory: the documents, the sessions, the chat threads, the
  tree, the settings.
- An **owner** is the one part of the code that can change one part of the state.
- An **event** is news for an owner: a key, a tick of a timer, an answer of the disk or of a
  program, a message of an agent, a line of the board, a change of a file from outside.
- A **sender** is where an event comes from.
- A **command** is work that changes state: a file, the state of zendo, a program. A **query** is
  work whose result changes only a view.

## The owner is a funnel

All events for a part of the state go to its owner. The owner handles one event to its end before
it takes the next. So a part of the state never changes in two places at once, and no code sees a
change that is half done. A timer is a sender like the others: the tick brings the time, and the
owner does not read the clock while it handles an event.

What leaves the owner (work for a thread, a program or another process) gets a copy of what it
needs. What comes back is an event with values. Nothing that crosses holds a reference to the
state, and no code waits in memory to act on "what is open" when an answer comes.

## Two ways to stay consistent

An owner states one of the two for each kind of event that it takes (User, 2026-11-19).

**More than one sender, with conflict detection.** Events of different senders can meet. The owner
detects each conflict and resolves it with a strategy ([Conflicts](conflicts.md)). Example: a
document. Its senders are the editor, the agent and git through the disk, the name model for its
title, a click on a task box, and another zendo process.

**One consistent stream.** The owner takes the events of one sender only, in the order of that
sender, and applies them as they come. The sender is responsible for the order and for the
consistency. Examples: the thread of a chat (its sender is the agent link), the screen of a
terminal (the reader of its PTY), the state of the board (the reader of the journal).

The second way is simpler, and is the choice where one sender is possible. An owner that gets a
second sender changes to the first way.

## Commands and queries

- The owner of a command is the part of the state that it changes. The commands of one owner run
  one at a time, in the order of the user. All changes of one file are commands of its document:
  the save, the title, the front matter, a task box, a rename.
- A command is in the state of its owner before its work starts. So the screen shows it at once,
  later events see it, and it completes or fails with a message when the user is in another place.
- The owner of a query is the view. When the view goes or asks again, the work stops and its
  result is dropped. Only the newest result counts.
- A fact that zendo did not read yet is "not known". It is not a default. An action that needs the
  fact waits for it.
- Each command and each query has a time limit, after which it fails with a message.

## What zendo shows of an action

zendo takes an action at once and completes it in its own time. An action has a state that the
user can see: taken, in progress, done or failed. No rule gives a time for the end; the rule is for
the first sign, within the 100 ms of IO-2. How the UI shows these states is in
[The UI layer](ui-layer.md).

## What holds always, and what holds in the end

Eventual consistency is everywhere: memory, the screen, the disk, programs, other zendo processes
and the members of a team hold copies that come to agree. So the rules are of two kinds.

Always:

- No text of the user is lost or goes into another file.
- A change enters the state only through its owner, and a conflict only through its strategy.
- zendo shows at once that it took an action.

In the end, when no more events come:

- Each action that zendo took is complete, or failed with a message.
- Each window shows the present state of what it shows.
- The disk has the text of each document.
- Each derived result fits its sources.
