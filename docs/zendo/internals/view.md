---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "model.md"
next: "layout-and-mouse.md"
level: "the view as a function of the model: read only, the whole frame, the cache of expensive results, animation"
note: "Agent draft from the chapter The view of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# The view

Sources: the chapter [The view: the screen as a function of the model](../../book/05-view.md) of
the zendo book.

Parent: [Internals of the TUI](internals.md).

## The screen is a function of the model

```rust
fn view(model: &Model, cache: &Cache, now: Instant) -> Ui
```

The view only reads: it gets `&Model`, so the compiler checks that it changes no cursor, no scroll
and no flag. After each `update`, the model is already correct, and a test sees the same state as
the user. [Layout](layout-and-mouse.md) gives the sizes before the view; `update` corrects the
scroll.

After each batch of messages, the runtime calls `view` and gets the whole screen. No code remembers
what changed: ratatui compares the new frame with the old one and sends only the changed cells.

## Expensive results

Markdown, the highlight of code and a diff are heavy. The view takes them from a `Cache`: a
finished result under the hash of its input. If the input changed, a command calculates the new
result ([ASYNC-4](async-io.md)), and until then the view shows the old result. The cache is not
state of the application; it only makes the view faster.

## Animation

For a button that blinks, the model keeps one fact: the time T0 when the blink started. The view
calculates the shade from `now - T0`. The frame depends on the time and stays deterministic. An
animation changes only the style of an element: the color, the shade, the symbol.

The view tells the runtime when it needs the next frame. When nothing moves, it asks for none.
If time changes what a key does, then that is state: it is in the model, and a timer message
changes it.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| VIEW-1 | Ubiquitous | `view` shall only read the model: it shall change no cursor, no scroll and no other state. | User, 2026-11-20 |
| VIEW-2 | Ubiquitous | The same model, the same cache and the same time shall always give the same frame. | User, 2026-11-20 |
| VIEW-3 | Ubiquitous | The view shall build the whole frame from the model each time, and the screen shall keep no state of its own. | User, 2026-11-20 |
| VIEW-4 | Ubiquitous | The view shall not calculate an expensive result (markdown, the highlight of code, a diff): it shall take the result from a cache outside the model. | User, 2026-11-20 |
| VIEW-5 | Ubiquitous | An animation shall change only the style of an element, and shall not change the geometry. | User, 2026-11-20 |

### Acceptance tests

**`the_view_does_not_change_the_model`** (unit test, not written yet; verifies VIEW-1)

- Given a model with a document that is longer than the screen, and a copy of the model.
- When the view draws the model two times.
- Then the model is equal to the copy.

**`the_same_model_gives_the_same_screen`** (unit test, not written yet; verifies VIEW-2, VIEW-3)

- Given a model, a cache and a time, and two test backends; one of them shows a different screen.
- When the view draws the model on both.
- Then the lines of the two screens are equal.

**`the_view_calls_no_renderer`** (unit test, not written yet; verifies VIEW-4)

- Given a model with a large markdown file whose render is not in the cache, and a renderer that
  counts its calls.
- When the view draws the model.
- Then the frame shows the old result or the plain text, and the renderer has no call.

**`an_animation_changes_only_the_style`** (unit test, not written yet; verifies VIEW-5)

- Given a model with a button that started to blink at T0.
- When the view draws at T0, at T0 + 100 ms and at T0 + 500 ms.
- Then the symbols of the three frames are in the same cells, the styles differ, and the hit map is
  the same.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| VIEW-6 | Ubiquitous | The cache shall keep each result under the hash of its input. | User, 2026-11-20 |
| VIEW-7 | State | While the cache has no result for the current input, the view shall show the old result or a placeholder. | User, 2026-11-20 |
| VIEW-8 | Ubiquitous | The model shall keep only the start time of an animation, and the view shall calculate the phase from `now`. | User, 2026-11-20 |
| VIEW-9 | Ubiquitous | The view shall tell the runtime the time of the next frame that it needs, and shall ask for no frame while nothing moves. | User, 2026-11-20 |
| VIEW-10 | Event | When the time of the next frame comes, the runtime shall draw again, shall send no message, and shall not call `update`. | User, 2026-11-20 |
| VIEW-11 | Unwanted | If time changes the behavior of zendo and not only its look, then the state shall be in the model, and a timer message shall change it. | User, 2026-11-20 |
