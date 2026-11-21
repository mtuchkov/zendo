# Chapter 8: The active panel and overlays

**The active panel**

The model has one active panel. The chapter about the keymap tells how a key goes to it.

The mode is a part of the panel. There is no global mode. The editor has its own modes (`Read`,
`Write`).

**Overlays: a stack of windows on top**

Also, there are windows on top of all: the music question, the tips, the window of the key change,
the chip window, the float, the wait for the end of a turn. An overlay is a panel. When it opens, it
becomes the active panel, and it decides the keys.

In this architecture, the overlays are in a **stack**:

```rust
overlays: Vec<Overlay>
```

The overlay that opened last is on top. The priority is the order of opening. It is visible in the
data, not in the code.

The same stack also controls the drawing: the view draws the overlays from the bottom to the top.
Thus the top overlay is always visible on top of the others. To close a window is to remove it from
the stack, and then the previous panel becomes active again.
