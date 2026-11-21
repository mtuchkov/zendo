---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "frame-and-budget.md"
next: "file-writes/file-writes.md"
level: "what the runtime measures of a cycle, of an update and of the write of a frame, and which records go to the telemetry"
note: "Draft from the decisions of the user of 2026-11-21 on the measurements, and from the chapter The frame and the time budget of the zendo book. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# The frame and the time budget: measurements

Sources: the decisions of the user on 2026-11-21; the chapter
[The frame and the time budget](../../book/13-frame-and-budget.md) of the zendo book.

Parent: [Internals of the TUI](internals.md). The writer thread and the budget are in
[The frame and the time budget](frame-and-budget.md). The log is in
[Telemetry log](../telemetry.md).

## Three measurements

The runtime measures three things, each in one place. The components hold no measurement
(BUDGET-2).

| Measurement | From | To |
| ----------- | ---- | -- |
| A cycle of the UI loop | The first message of the cycle | The filled buffer of the frame |
| An `update` call | The start of the call | The end of the call |
| The write of a frame | The writer takes the frame | The frame is on the terminal |

The write of a frame is not a part of the cycle: a slow terminal does not look like a slow core.

## Records in the telemetry

- **A slow cycle.** A cycle over the budget gets one record: the time of the cycle, the time of
  the `view`, and the times of its messages by type. The record shows both cases: one heavy paste,
  and 50 small messages that together are too slow.
- **A slow update.** An `update` call that alone goes over the budget gets a record of its own,
  with the type of its message (User, 2026-11-21).
- **A slow write.** The write of a frame over its limit gets a record (User, 2026-11-21).
- **The histogram of the writes.** The runtime also counts the time of each write in a histogram.
  The histogram collects one minute, and then goes to the telemetry as one record (User,
  2026-11-21). A record for each frame is too much for the log.

Design: the histogram has fixed buckets on a logarithmic scale. Two histograms with the same
buckets add up exactly, so an analytics service can join the minutes, the processes and the
machines. A t-digest needs no choice of buckets and is more exact in the tail, but its joins are
approximate, its record is larger, and the service must know its format. The tail is exact here
without it: each slow write has its own record.

No record holds text of the user, a file name or a path (TELEM-7).

This document has no architecturally significant requirement of its own: FRAME-1, BUDGET-2 and
BUDGET-3 of [The frame and the time budget](frame-and-budget.md) cover the measurements.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| FRAME-5 | Ubiquitous | The runtime shall measure the write of a frame to the terminal as a measurement of its own, apart from the budget. | User, 2026-11-21 |
| FRAME-8 | Unwanted | If the write of a frame takes longer than its limit, then the runtime shall write a record of the write in the telemetry. | User, 2026-11-21 |
| FRAME-9 | Ubiquitous | The runtime shall count the time of each write of a frame in a histogram for each interval of one minute, and shall write the histogram of each interval that has a write to the telemetry as one record. | User, 2026-11-21 |
| BUDGET-4 | Ubiquitous | A record of a measurement shall hold nothing that identifies the user (TELEM-7). | TELEM-7 |
| BUDGET-5 | Ubiquitous | The record of a slow cycle shall hold the time of the cycle, the time of the `view`, and for each type of message in the cycle its count, the sum of its `update` times and its longest `update`. | Design |
| BUDGET-6 | Unwanted | If one `update` call alone goes over the budget, then the runtime shall write a record of that call in the telemetry, with the type of its message and its time. | User, 2026-11-21 |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| What is the limit of a slow write? | FRAME-8 | User |
| Fixed buckets on a logarithmic scale (the proposal above), or a t-digest? And with fixed buckets: which limits? | FRAME-9 | User |
| Is "slow" for one `update` the whole budget of 8 ms, or a smaller limit? | BUDGET-6 | User |
| [Telemetry log](../telemetry.md) states one kind of event, the action of the keymap (TELEM-1). These records are new kinds of event. Does the telemetry specification get a revision for them? | FRAME-8, FRAME-9, BUDGET-3, BUDGET-6 | User |
