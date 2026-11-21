---
review:
  reviewer: "agent, new context"
  date: "2026-11-20"
  passes: 1
  result: "pass"
  hash: "sha256:6e4de2bd9822213f43991b8df057aa7d5cbd6152eb05ffe4a3b5189c07d95915"
---
# Chapter 1: Core and Runtime

All of the architecture is based on one division. The application has two parts.

**The core** only decides. It knows all the state of the application: the open document, the
cursor, the chats, the mode. When an event occurs, the core decides how the state changes and what
to do outside. The core itself does nothing outside. It does not read files, it does not use the
network, and it does not read the clock.

**The runtime** only does. It reads the keyboard, writes to the disk, talks to the agent, and draws
on the screen. It makes no decisions of its own. It does what the core tells it to do.

A comparison: the core is a cook who writes an order on a sheet of paper ("bring the flour, turn on
the oven"). The runtime is a helper who goes to get the flour. While the helper is away, the cook
does not stand and wait. The cook takes the next order.

This gives the first benefit. The core never waits for a slow operation, because the operation does
not run in the core. Thus the core cannot stop the screen because of the disk or the network.

The second benefit: it is easy to test the core. Give it an event and check the result. You need no
temporary folders, no mock agent, and no delays.
