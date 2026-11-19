---
status: "draft"
date: "2026-11-19"
parent: "blueprints.md"
prev: "conflicts.md"
next: "evolution.md"
level: "the UI of zendo above ratatui: windows and layers, what a view shows, derived results, jobs and notifications"
note: "Initial design blueprint. Agent draft from the statements of the user in chat on 2026-11-19. Not reviewed by the user. Not a source of truth: the specifications in docs/zendo are."
---

# The UI layer

Parent: [Blueprints of zendo](blueprints.md).

ratatui is the canvas of zendo. zendo builds its own UI above it (User, 2026-11-19): layers,
floating windows, loading, notifications and background jobs.

## What ratatui does, and what zendo does

| Task | ratatui | zendo |
| ---- | ------- | ----- |
| The grid of cells, the styles, the output to the terminal | Yes | - |
| The difference of two frames: only the changed cells go to the terminal | Yes | - |
| The split of a rectangle by constraints, with a cache | Yes | - |
| Windows, layers and their order | No | The UI layer |
| The active window, the focus, the way of a key | No | The UI layer |
| The window under the mouse | No | The UI layer |
| The content of a rectangle, such as markdown as rows | Each frame again | Cached, see below |

## Windows and layers

- A **window** is a part of the screen with one owner: a pane (the tree, the note, the delta, the
  chat) or a floating window (the new note, the chip window, the key float, find, the help).
- Each window is bound to its subject: a document, a session, a list. No window asks for "the open
  draft". The float of a new note has its own document, and the note behind it keeps its own.
- The windows are in layers. From the size of the terminal and the set of windows, zendo computes
  the rectangle of each window, the order of the layers and the active window. This layout is a
  derived value, and the draw and the input use the same one: the draw paints the windows from the
  bottom layer up, a key goes to the active window, and the mouse goes to the top window under it.
- The layers replace three things of today: the cache of what the last draw left for the keys and
  the mouse (ADR 0008), the one mode and the one focus of the whole app, and the swap of state
  that makes the float.

## What a view shows

- The state that windows show is held as signals of `reactive_graph`. A window reads the signals
  of what it shows, and the library records that. When one of them changes, the owner of the window
  knows that it must draw again. The window is the unit: an element in it has no notice of its own.
- A change asks for a frame. zendo draws all visible windows into the buffer of ratatui, and
  ratatui sends only the difference. zendo keeps no list of changed rectangles.
- An effect of `reactive_graph` runs on the next turn of the async runtime, not at the change. The
  main loop gives that turn, and a test must give it too. The turn is part of the 100 ms in which
  zendo shows that it took an action.

## Derived results

- A result that costs time and depends only on its inputs is cached with `comemo`: the rows of a
  note, a delta, the links between requirements and tests. `comemo` keeps a result while what the
  function read is the same, also when other state changed.
- A long document is cut into blocks (User, 2026-11-19). The rows of each block are cached. An edit
  computes only the changed block again, and the place of each block is the running sum of the
  heights. The key of the cache is the content of a block, not its number.
- Blocks of markdown are not always independent: a code fence that is not closed changes all that
  follows, the definition of a link can be far from its use, and a numbered list goes on over
  blocks. The cut is at blocks of the top level, and what blocks share is an input that `comemo`
  tracks.
- `comemo` gives no notice. The signals say when to ask again; `comemo` makes the answer cheap.

## Jobs, loading and notifications

Loading, notifications and background jobs show one thing: work that has not ended.

- A **job** is a command or a query that runs (see [State](state.md)). It has an owner, a kind and
  a state: taken, in progress, done or failed.
- **Loading** shows a job in place, while the window that asked is on the screen.
- A **notification** shows the end of a job when its window is not on the screen, and each failure.
- The list of **background jobs** shows the jobs that run.
- The owner gives the rule of cancellation. A job of a window stops when the window closes. A job
  of a document or a session goes on, and stays in the list.

Today each of these is made by hand, with no common job, and the status bar has one slot for a
message, where the newest replaces the older. A failure can go from the screen before the user
reads it.

## Open points

| Open question | Affects |
| ------------- | ------- |
| How do the sub-states of ADR 0008 become owners with signals: one signal for each sub-state, or one for each field? | What a view shows |
| Does `reactive_graph` give what a change needs to name its base? | [State](state.md) |
| What does a spike on one window cost (the note pane and its float), and do the tests stay deterministic? | All sections |
| Which ideas of `ratatui-reactive` are worth taking? | Windows and layers |
