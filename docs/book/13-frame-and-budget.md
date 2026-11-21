---
review:
  date: "2026-11-21"
  passes: 1
  result: "pass"
  hash: "sha256:58ae23d4ceb435c982936e05fa53b4e0401ad432c72004993d2eaed2a349f523"
---
# Chapter 13: The frame and the time budget

Two places remain where the UI can still stop, also when all commands are asynchronous.

**First: the write to the terminal is also IO**

The drawing has two steps:

1. **Build the frame**: `view` fills a buffer in memory. This is fast.
2. **Write the frame** to the terminal through stdout. This is IO.

Usually the write is immediate. But through a slow SSH connection or with a busy tmux, stdout can
block. Then the loop stops, the keys collect, and the screen "freezes".

Thus step 2 goes to a **separate writer thread**. The loop builds the buffer and gives it to the
writer through a channel for **one frame**. If the writer did not finish the last frame yet, then
the new frame **replaces** the frame that waits in the channel. The intermediate frames are lost,
and this is correct: the user needs only the last frame. The writer itself keeps the last frame
that it wrote, and it sends only the difference to the terminal.

**Second: a slow `update`**

Async gives protection from a wait for IO. But it does not give protection from slow code in the
core itself. If `update` does an O(n²) operation on a large document, then the frame is late, and
tokio cannot help.

Thus the core has a **budget: 8 ms** for `update` and `view` together. This is half of a frame at
60 Hz, with a margin.

How the budget is checked:

- **The runtime measures each message in one place.** It measures the time from the arrival of the
  `Msg` to the end of the frame. The code of the components has no measurements.
- **If a message goes over the budget, then the runtime writes a record in the telemetry** with the
  type of the message. Thus you see more than "it is slow": you see "`Msg::Editor(Paste)` is slow on
  a 2 MB file".
- **A scenario test with slow services.** Fake services answer each request after 5 seconds. The
  test pushes keys and checks that the screen changes within the budget. If a person adds a wait for
  IO into the loop by mistake, then this test fails.

An honest note: the budget and the telemetry **find** a slow `update`, but they do
not **prevent** it. This is the only place where the architecture uses measurement, not
construction.
