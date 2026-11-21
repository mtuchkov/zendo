# Chapter 6: Layout and the mouse

Here there are two related tasks: find the sizes of the panes, and find where the user clicked.

**Layout: sizes without drawing**

```rust
fn layout(model: &Model) -> Layout
```

This is a pure function. The input is the model: the size of the screen, the open panes, the width
of the text column. The output is a stack of layers: the panes at the bottom, and one layer for each
overlay. The order is the overlay stack of the model. An overlay does not change the geometry of the
layers under it. The size of the screen goes into the model through `Msg::Resize`, as all other events do.

**Two** parts call layout:

- `update`, to correct the scroll. If the cursor moved down, then `update` asks layout for the
  height of the pane and moves the scroll so that the cursor is visible.
- `view`, to know where to draw.

It is the same function on the same model, thus the answer is the same. `update` and `view` cannot
have different sizes. And after each `update`, the model is already correct, with no wait for the
draw.

Thus only `layout` sets the geometry. An animation does not move an element and does not change its
size. If the view moves an element with time, then the view and `layout` have different sizes, and
`update` corrects the scroll for a pane that the user does not see.

**The mouse: a click becomes a message immediately**

In this architecture, an element of the screen itself tells what a click on it means:

```rust
button("▶ play").on_click(Msg::Music(Toggle))
```

While the runtime draws a frame, it fills a **hit map**: a grid of cells with the same size as the
frame. When the view draws an element into a cell, it also writes the id of the element into this
cell of the grid. An element that is drawn later writes over the cell. Thus each cell has the
element that the user sees. When a click comes at the point (x, y), the runtime reads the cell and
sends the message of its element to `update`. A modal overlay draws an invisible background on all
of the screen, with its own message for a click outside. The core gets `Msg::Music(Toggle)` and does not know the coordinates.

For text, where the exact position is important, the element does not give a finished message. It
gives a function of the local coordinates. For example,
`|row, col| Msg::Reader(ClickAt { row, col })`.

Why a map from the *last* frame is correct: the user clicks on what the user sees. And the user
sees the last frame.

As a result, the model has no frame cache. The mouse is only one more source of messages.
