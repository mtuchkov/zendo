---
review:
  reviewer: "agent, new context"
  date: "2026-11-20"
  passes: 1
  result: "pass"
  hash: "sha256:73ba97d7532386f5a1116fd9c450f0c4e1a4c3550a6b134a80c56316caf86334"
---
# Chapter 5: The view: the screen as a function of the model

`view` takes the model and builds the screen from it:

```rust
fn view(model: &Model, cache: &Cache, now: Instant) -> Ui
```

The key point here is `&Model`, not `&mut Model`. The view only **reads**. It cannot change the
cursor, the scroll, or a flag. The compiler checks this, not the review.

**The screen is drawn again completely.** After each batch of messages, the runtime calls `view`
and gets all of the screen. This is not slow: ratatui itself compares the new frame with the old
frame and sends only the changed cells to the terminal. We do not need to remember "what changed".
It is sufficient to describe how the screen looks with this state.

This gives a simple rule: **the same Model always gives the same screen.** Thus it is easy to test
the screen. Make a model, call `view` on `TestBackend`, and compare the lines.

**Why the view must not change anything.** If the view does not change the state, then only
`update` changes it. After each `update`, the model is already correct. A test sees the same state
as the user. In this architecture, a separate pure function calculates the sizes of the panels
before the view. Then `update` corrects the scroll, and the view
only shows. The chapter about layout tells how.

**Expensive rendering.** Markdown, code highlight, diff: these are heavy. Do not calculate them
again for each frame. Thus the view has a `Cache`: a finished result, attached to
the hash of its input. If the input is the same, the view takes the result from the cache. If the
input changed, a background command calculates the new result, and until then the view shows
the old result. The cache is outside the model: it is not the state of the application, it only
makes the view faster.

**Animation.** A blinking button is a good example. The model keeps only one fact: "the button
started to blink at time T0". `update` sets this time when the turn ends, and `update` clears it
when the user pushes a key. The model does not keep the phase of the animation. The view calculates
it: the shade is a function of `now - T0`. Thus the frame depends on the time, but it is still
deterministic: the same model and the same `now` always give the same frame. A test gives `now`
explicitly.

An animation changes only the style of an element: the color, the shade, the symbol. It does not
change the geometry. The chapter about layout tells why.

The view also tells the runtime when it needs the next frame: `next_frame`. While the button
blinks, the view asks for a frame after 100 ms. When nothing moves, the view asks for no frame. The
runtime draws again at this time, but it sends no `Msg` and it does not call `update`: the model
did not change.

But time can also change the behavior: for example, a key does a different thing after some time.
Then this state is not a part of the animation. It goes into the model, and a timer `Msg` changes
it.
