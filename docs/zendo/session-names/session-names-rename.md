---
status: "draft"
revision: "2"
date: "2026-11-17"
parent: "session-names.md"
next: "session-names-api.md"
level: "when zendo asks again for the name of an agent session, and why a name does not change"
note: "From the requests of the user on 2026-11-14, split from session-names.md in its revision 2. Revision 1 approved by the user on 2026-11-14, implemented in zendo 0.70.0. Revision 2 (draft, from the request of the user on 2026-11-17): a name does not change while its Claude session lasts; NAMES-14 is retired."
---

# Session names: rename

The first question for the name of an agent session comes with the first agent message of its
Claude session ([Session names](session-names.md), NAMES-1). This document tells when zendo asks
again. A name does not change while its Claude session lasts (NAMES-17): only a new Claude session,
such as a new thread or `/clear`, gets a new name.

**No name yet.** While the topic is not clear yet, the model answers `-`, and the agent session
shows its key alone. zendo asks again when the conversation has grown by 200 characters, or when a
turn has run 30 s more and the conversation has grown at all. The conversation is the questions,
the messages of the agent and the titles of its tool calls; the thinking of the agent has no text,
so only its time counts. After the first name, zendo asks no more.

**No rename.** Revision 1 asked at these thresholds whether the name still fits and changed it to a
new word (NAMES-14). The user found a name that moves during a session confusing, so revision 2
retires NAMES-14. Its identifier is not used again.

## Architecturally significant requirements

None in this document; NAMES-17 is in [Session names](session-names.md).

## Detailed requirements

| ID       | Pattern  | Requirement | Source |
| -------- | -------- | ----------- | ------ |
| NAMES-10 | Unwanted | If the model finds the topic not clear yet or gives no free name, then the system shall show the key alone and ask again after 200 more characters of the conversation or 30 s more of the turn. | User, 2026-11-14 |

Tests of the detailed requirements in `src/agent/work/names_tests.rs`:
`due_after_200_characters_or_30_seconds` (NAMES-10) and
`told_needs_a_question_and_a_complete_agent_message` (NAMES-1).
