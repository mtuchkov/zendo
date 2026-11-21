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
- **A slow update.** An `update` call that alone takes half of the budget or more, 4 ms, gets a
  record of its own, with the type of its message (User, 2026-11-21).
- **A slow write.** The write of a frame that takes more than 100 ms gets a record (User,
  2026-11-21). The user sees such a write as a freeze, and the limit keeps the records few: at most
  10 for each second.
- **The distribution of the writes.** The runtime also collects the time of each write in a
  t-digest. The digest collects one minute, and then goes to the telemetry as one record (User,
  2026-11-21). A record for each frame is too much for the log. A t-digest needs no choice of
  buckets, and an analytics service can join the digests of many minutes and read the quantiles
  later. The record holds the centroids of the digest, each with its mean and its count.

Design: a digest has at most 100 centroids. In a test with 60 minutes of simulated writes, up to
3,600 for each minute, 100 centroids gave a rank error of about 0.1 percentage points or less from
p50 to p99.9, for one minute and for the joined hour, at about 1 KB for each record. 50 centroids
gave two to three times the error; 200 gave two times the size.

Each record is one event of the log, with the fields of TELEM-6 and a field that names its kind
(Design).

No record holds text of the user, a file name or a path (TELEM-7).

This document has no architecturally significant requirement of its own: FRAME-1, BUDGET-2 and
BUDGET-3 of [The frame and the time budget](frame-and-budget.md) cover the measurements.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| FRAME-5 | Ubiquitous | The runtime shall measure the write of a frame to the terminal as a measurement of its own, apart from the budget. | User, 2026-11-21 |
| FRAME-8 | Unwanted | If the write of a frame takes more than 100 ms, then the runtime shall write a record of the write in the telemetry. | User, 2026-11-21 |
| FRAME-9 | Ubiquitous | The runtime shall collect the time of each write of a frame in a t-digest for each interval of one minute, and shall write the digest of each interval that has a write to the telemetry as one record. | User, 2026-11-21 |
| FRAME-10 | Ubiquitous | The record of a digest shall hold the centroids of the digest, each with its mean and its count, and at most 100 centroids. | User, 2026-11-21; Design |
| BUDGET-4 | Ubiquitous | A record of a measurement shall hold nothing that identifies the user (TELEM-7). | TELEM-7 |
| BUDGET-5 | Ubiquitous | The record of a slow cycle shall hold the time of the cycle, the time of the `view`, and for each type of message in the cycle its count, the sum of its `update` times and its longest `update`. | Design |
| BUDGET-6 | Unwanted | If one `update` call takes half of the budget or more, then the runtime shall write a record of that call in the telemetry, with the type of its message and its time. | User, 2026-11-21 |
| BUDGET-7 | Ubiquitous | Each record of a measurement shall be one event of the telemetry log, with a field that names its kind. | Design |
