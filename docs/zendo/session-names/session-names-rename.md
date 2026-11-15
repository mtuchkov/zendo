---
status: "implemented"
approved: "2026-11-14"
revision: "1"
date: "2026-11-14"
parent: "session-names.md"
next: "session-names-buttons.md"
level: "when zendo asks the agent again for the name of an agent session, and when a name changes"
note: "From the requests of the user on 2026-11-14, split from session-names.md in its revision 2. Approved by the user on 2026-11-14. Implemented in zendo 0.70.0."
---

# Session names: rename

The first question for the name of an agent session comes with the first agent message of its
Claude session ([Session names](session-names.md), NAMES-1). This document tells when zendo asks
again, and when a name changes.

**The thresholds.** zendo asks again when the conversation has grown by 200 characters, or when a
turn has run 30 s more and the conversation has grown at all. The conversation is the questions,
the messages of the agent and the titles of its tool calls; the thinking of the agent has no text,
so only its time counts.

**No name yet.** While the topic is not clear yet, the model answers `-`, and the agent session
shows its key alone. zendo asks again at the thresholds.

**A name that no longer fits.** While a session has a name, zendo asks the model at the thresholds
whether the name still fits. The model keeps it with `=` or gives a new word. Each `=` doubles the
thresholds: 30 s, 1 min, 2 min, 4 min, and so on, and 200, 400, 800 characters. So a long
conversation on one topic costs few questions, and its name stays still.

## Architecturally significant requirements

| ID       | Pattern | Requirement | Source |
| -------- | ------- | ----------- | ------ |
| NAMES-14 | State   | While an agent session has a name, the system shall ask the agent whether the name still fits after 200 more characters of the conversation or 30 s more of the turn, double these thresholds after each answer that keeps the name, and change the name only to a new free name that the agent gives. | User, 2026-11-14 |

### Acceptance tests

**`a_changed_topic_renames_the_session`** (mock agent, in `tests/session_names.rs`; verifies
NAMES-14, NAMES-3)

- Given agent session 1 with a name, and a mock agent that keeps the name once and then gives a new
  word.
- When the conversation grows by 200 characters, and then by more than 400.
- Then the first check keeps the name and doubles the thresholds, and the second one shows the new
  word. The unit test `due_after_the_thresholds_doubled_by_each_kept_name` checks the thresholds.

## Detailed requirements

| ID       | Pattern  | Requirement | Source |
| -------- | -------- | ----------- | ------ |
| NAMES-10 | Unwanted | If the agent finds the topic not clear yet or gives no free name, then the system shall show the key alone and ask again after 200 more characters of the conversation or 30 s more of the turn. | User, 2026-11-14 |

Tests of the detailed requirements in `src/agent/work/names_tests.rs`:
`due_after_the_thresholds_doubled_by_each_kept_name` (NAMES-10, NAMES-14) and
`told_needs_a_question_and_a_complete_agent_message` (NAMES-1).
