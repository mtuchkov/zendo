---
status: "approved"
approved: "2026-11-18"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "transcript.md"
next: "interactivity.md"
level: "how the status line of a turn shows that the agent is alive: the thought, the waits of the agent, the pulse"
note: "Draft from the request of the user on 2026-11-18 in chat: a more alive status. Split from Interactivity of the chat to keep each document within two pages. Approved by the user on 2026-11-18."
---

# A status line that is alive

[Activity of the agent](activity.md) shows what the agent does in the status line. This document
makes the line show why the agent is quiet, so that a long wait does not look like a stuck turn.
What the user can do during a turn is in [Interactivity of the chat](interactivity.md). Some of it
uses extensions of the Claude adapter, not of ACP. Where an agent does not have a feature, only its
lines turn off (CLAUDE-14).

**The thought.** While the agent thinks, the status line shows the last line of its thought in the
dim color, cut to the width of the pane, such as `thinking: checks whether the hold blocks a
second ctrl-o…`. The thought does not go into the transcript.

**The waits of the agent.** When the agent waits for a rate limit, the status line says so and
counts down, such as `rate limited · retry in 20 s`. When the agent compacts the context of the
thread, the line shows `compacting context…`. zendo then knows why no news comes, so these lines
take the place of `no news` (ACT-2).

**The pulse.** While a turn runs, the spinner and the times of the steps change at least once per
second. The button of a session whose agent works pulses slowly and softly, so that it does not
look like the blink at the end of a turn ([Attention](attention.md)).

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| LIVE-1   | State   | While the agent thinks, the system shall show the last line of its thought in the status line. | User, 2026-11-18 |
| LIVE-2   | State   | While the agent waits for a rate limit or compacts the context, the system shall show the reason in the status line, and for a rate limit the time that is left. | User, 2026-11-18 |

### Acceptance tests

Tests: `tests/live_status.rs`, with the mock agent.

**`the_status_line_shows_the_thought_and_the_waits`** (mock agent; verifies LIVE-1, LIVE-2)

- Given a turn in which the mock agent sends a thought, then a rate limit of 20 s, then a compaction.
- Then the status line shows the last line of the thought, then `rate limited · retry in 20 s`
  counting down, then `compacting context…`.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| LIVE-3   | Ubiquitous | The system shall not put the thought of the agent into the transcript. | Design |
| LIVE-4   | State      | While a turn runs, the system shall change the spinner and the times of the steps at least once per second. | Design |
| LIVE-5   | State      | While the agent of a session works, the system shall pulse the button of that session slowly and softly, unlike the blink of ATTN-3. | Design |

## Open points

None. The user answered the open points on 2026-11-18.
