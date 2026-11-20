---
status: "draft"
date: "2026-11-19"
prev: "conflicts.md"
next: "candidate-requirements.md"
level: "the UI of zendo above ratatui: windows and layers, the way of a key and of the mouse, the draw of a frame, derived results, jobs and notifications"
note: "Initial design blueprint. Agent draft from the statements of the user in chat on 2026-11-19. Written again on 2026-11-19 on The Elm Architecture, which the user chose. Not reviewed by the user. Not a source of truth: the specifications in docs/zendo are."
---

# The UI layer

Before: [Conflicts](conflicts.md). Next: [Candidate requirements](candidate-requirements.md).

ratatui is the canvas of zendo. zendo builds its own UI above it (User, 2026-11-19): layers,
floating windows, loading, notifications and background jobs. This blueprint uses the terms of The
Elm Architecture ([State](state.md)): model, message, `update`, effect and component. tui-realm
is a source of ideas only. The sections are a proposal of the agent, except where one names the
user.

## What ratatui does, and what zendo does

| Task | ratatui | zendo |
| ---- | ------- | ----- |
| The grid of cells, the styles, the output to the terminal | Yes | - |
| The difference of two frames: only the changed cells go to the terminal | Yes | - |
| The split of a rectangle by constraints, with a cache | Yes | - |
| Windows, layers and their order | No | The UI layer |
| The focus, and the way of a key and of the mouse | No | The UI layer |
| The content of a rectangle, such as markdown as rows | Each frame again | Cached, see below |

## Windows and layers

- A window is a pane (the tree, the note, the delta, the chat) or a floating window (the new note,
  the chip window, the key float, find, the help).
- Each window is bound to its subject: a document, a session, a list. No window asks for "the open
  draft". The float of a new note has its own document, and the note behind it keeps its own.
- The windows are in a **stack** of layers: the panes are the bottom layer, and each floating
  window is a layer above.
- A **modal** window takes all keys while it is open, except the global keys.
- When a sub-state of ADR 0008 becomes a component, the data that another window, a save or a
  command needs goes to the model, and its cursor, scroll and selection go to the component
  (User, 2026-11-20).
- The **layout** is the rectangle of each window. zendo computes it from the size of the terminal
  and the stack, and `update`, the draw and the input use the same layout.

## Keys and the mouse

- A global key, such as the key of find, always goes to the app first, not to a window (User,
  2026-11-20). The global keys are in one table and not in the code of the windows. While a
  terminal has the keys, the table is the short list of TERM-13 and SESSIONS-9.
- Each other key goes to the focus: the top window of the stack (User, 2026-11-20). A window that
  is not modal passes a key that it does not use to the window below.
- The mouse goes to the top window under it. Its component has one function that computes the
  places of its buttons and rows; the draw and the mouse both ask it (User, 2026-11-20).
- When a window closes, the focus goes back to the window that had it before.

The stack replaces four things of today: the order of conditions that chooses the code for a key,
the one mode and focus of the whole app, the cache of what the last draw left for the keys and the
mouse (ADR 0008), and the swap of state that makes the float.

## The draw of a frame

- After `update`, zendo fits the scroll and the cursor of each window on the screen to its
  rectangle and its rows, in one step and not in the code of each message (User, 2026-11-20).
- Then zendo draws one frame, at the latest 12 ms after the first message that waits (User,
  2026-11-20): the windows from the bottom layer up, into the buffer of ratatui.
  ratatui sends only the difference to the terminal. zendo keeps no list of changed rectangles.
- `draw` returns nothing, fills the whole rectangle of its window, and changes nothing.
- zendo does not draw a window that another window covers fully.
- zendo does not cut a window that is covered in part. The cut saved 0.2 ms of a frame in a
  measurement of 2026-11-19, and needs code in each component.
- Before zendo draws a modal window, it dims all that is below. The stack does it; the windows
  have no code for it.

## Derived results

- A result that costs time and depends only on its inputs is cached with `comemo` (User,
  2026-11-19): the rows of a note, a delta, the links between requirements and tests. `comemo`
  keeps a result while what the function read is the same.
- A cached result is not a part of the model: zendo can compute it again.
- The layout and the rows of a document after the wrap are such results (User, 2026-11-20): zendo
  computes them when they are asked for, and again only after their inputs changed. So `update`
  can use them, and not only the draw.
- A document is cut into blocks, so that a change from the user or from outside computes only a
  part of the derived results again (User, 2026-11-20). The borders of the blocks are the native
  parts of the content: the blocks of markdown. Source code is cut at a fixed number of lines.
  How a window shows the content is a decision of its view. No requirement covers the blocks
  yet (User, 2026-11-20).
- Blocks of markdown are not always independent: a code fence that is not closed changes all that
  follows, the definition of a link can be far from its use, and a numbered list goes on over
  blocks. The cut is at blocks of the top level, and what blocks share is an input that `comemo`
  tracks.

## Jobs, loading and notifications

Loading, notifications and background jobs show one thing: work that has not ended.

- A **job** is a command or a query that runs. The model holds it with its subject, its kind and
  its state: taken, in progress, done or failed.
- **Loading** shows a job in its window, while that window is on the screen.
- A **notification** shows the end of a job when its window is not on the screen, and each failure.
- The list of **background jobs** shows the jobs that run.
- A query stops when its window closes. A command goes on, and stays in the list.

Today each of these is made by hand, with no common job, and the status bar has one slot for a
message, where the newest replaces the older. A failure can go from the screen before the user
reads it.
