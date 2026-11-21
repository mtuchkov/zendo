---
review:
  date: "2026-11-21"
  passes: 1
  result: "pass"
  hash: "sha256:164ae240f56cff4798f38ee8562c46efaf2084a328bd440b4b4d448e7371f4b3"
---
# Chapter 8: The active panel and overlays

**The active panel**

The model has one active panel. The chapter about the keymap tells how a key goes to it.

The mode is a part of the panel. There is no global mode. The editor has its own modes (`Read`,
`Write`).

**Overlays: a stack on top of the panels**

Also, there are overlays on top of all panels. For example: the music question, the tips, the
window of the key change, the chip window, the float, the wait for the end of a turn. An overlay is a panel. When it opens, it
becomes the active panel, and it decides the keys.

In this architecture, the overlays are in a **stack**:

```rust
overlays: Vec<Overlay>
```

The overlay that opened last is on top. The priority is the order of opening. It is visible in the
data, not in the code.

The same stack also controls the drawing: the view draws the overlays from the bottom to the top.
Thus the top overlay is always visible on top of the others. To close an overlay is to remove it from
the stack, and then the previous panel becomes active again.
