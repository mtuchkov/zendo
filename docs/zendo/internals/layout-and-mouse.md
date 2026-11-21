---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "view.md"
next: "components.md"
level: "the sizes of the panels without a draw, and the way of a mouse click to a message"
note: "Agent draft from the chapter Layout and the mouse of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Layout and the mouse

Sources: the chapter [Layout and the mouse](../../book/06-layout-and-mouse.md) of the zendo book.

Parent: [Internals of the TUI](internals.md). What the user sees of the panes is in
[Panes and layouts](../panes/panes.md).

## Layout: sizes without drawing

```rust
fn layout(model: &Model) -> Layout
```

`layout` is a pure function. Its input is the model: the size of the screen, the open panels, the
width of the text column. Its output is a stack of layers: the panels at the bottom, and one layer
for each [overlay](active-panel-and-overlays.md).

Two parts call it. `update` corrects the scroll: when the cursor moves down, `update` asks `layout`
for the height of the panel and moves the scroll so that the cursor is visible. `view` asks where
to draw. It is the same function on the same model, so the two cannot have different sizes, and the
model is correct after each `update`, with no wait for the draw.

Only `layout` sets the geometry. An animation does not move an element
([VIEW-5](view.md)): if it did, `update` would correct the scroll for a panel that the user does
not see.

## The mouse: a click becomes a message

An element of the screen tells what a click on it means:

```rust
button("▶ play").on_click(Msg::Music(Toggle))
```

While the runtime draws a frame, it fills a hit map: a grid of cells with the size of the frame.
Each cell holds the element that was drawn last into it, so each cell has the element that the
user sees. When a click comes, the runtime reads the cell and sends the message of its element. The
core does not know the coordinates.

The map of the last frame is correct: the user clicks on what the user sees. As a result, the
model has no frame cache. The mouse is one more source of messages.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| LAYOUT-1 | Ubiquitous | `layout` shall be a pure function of the model that gives the geometry of the screen without a draw. | User, 2026-11-20 |
| LAYOUT-2 | Ubiquitous | `update` and `view` shall take the geometry only from `layout`, so that both have the same sizes. | User, 2026-11-20 |
| LAYOUT-3 | Event | When a message moves the cursor or changes a size, `update` shall correct the scroll with `layout` in the same call, so that the model is correct before the draw. | User, 2026-11-20 |
| MOUSE-1 | Event | When a mouse event comes, the runtime shall send to `update` the message of the element that the last frame shows at that point. | User, 2026-11-20 |
| MOUSE-2 | Ubiquitous | The model shall hold no data of a frame: no rectangle, no row and no cell of the last draw. | User, 2026-11-20 |

### Acceptance tests

**`update_and_view_have_the_same_sizes`** (unit test, not written yet; verifies LAYOUT-1, LAYOUT-2)

- Given a model with three open panels and one overlay, at 80 × 24 and at 200 × 50.
- When the test calls `layout` two times and draws the view.
- Then the two results are equal, and each panel is drawn in its rectangle of `layout`.

**`the_cursor_is_visible_after_update`** (unit test, not written yet; verifies LAYOUT-3)

- Given a model with a document that is longer than its panel, and the cursor on the last visible
  row.
- When the test gives the message that moves the cursor one row down, and draws nothing.
- Then the scroll in the model shows the row of the cursor inside the height that `layout` gives.

**`a_click_becomes_the_message_of_the_element`** (unit test, not written yet; verifies MOUSE-1)

- Given a frame with the music button, and an overlay that covers a part of the button.
- When a click comes on a free cell of the button, and then on a covered cell.
- Then `update` gets the message of the button, and then the message of the overlay.

**`the_core_handles_a_click_without_a_frame`** (unit test, not written yet; verifies MOUSE-2)

- Given a model, and no frame drawn.
- When the test gives the click message of a row and a column of the reader.
- Then `update` moves the cursor to that position of the text.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| LAYOUT-4 | Event | When the terminal changes its size, the new size shall come into the model in a message. | User, 2026-11-20 |
| LAYOUT-5 | Ubiquitous | `layout` shall give a stack of layers: the panels at the bottom, then one layer for each overlay, in the order of the overlay stack. | User, 2026-11-20 |
| LAYOUT-6 | Ubiquitous | An overlay shall not change the geometry of the layers under it. | User, 2026-11-20 |
| MOUSE-3 | Ubiquitous | An element that the user can click shall hold the message of its click. | User, 2026-11-20 |
| MOUSE-4 | Ubiquitous | The runtime shall fill, with each frame, a hit map of the size of the frame, where each cell names the element that was drawn last into it. | User, 2026-11-20 |
| MOUSE-5 | Ubiquitous | A modal overlay shall draw a background on the whole screen that holds its own message for a click outside the overlay. | User, 2026-11-20 |
| MOUSE-6 | Optional | Where the position of the click in an element matters, as in text, the element shall give a function from its local row and column to a message. | User, 2026-11-20 |
| MOUSE-7 | Ubiquitous | The core shall get no screen coordinates of a mouse event. | User, 2026-11-20 |
