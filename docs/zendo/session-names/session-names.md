---
status: "draft"
revision: "3"
date: "2026-11-17"
parent: "../zendo.md"
prev: "../sessions.md"
next: "../handoff.md"
children:
  - "session-names-rename.md"
  - "session-names-api.md"
  - "session-names-buttons.md"
level: "the names of the agent sessions: when they come, how long they stay, where they show"
note: "From the requests of the user on 2026-11-14; the user approved the requirements in the chat. NAMES-8, NAMES-11 to NAMES-13 came later on the same day. Revision 2 (approved by the user on 2026-11-14, implemented in zendo 0.70.0) names a session from its first agent message, during the turn. Revision 3 (draft, 2026-11-17): a name stays for its Claude session (NAMES-17, NAMES-14 retired); the API of the agent answers (session-names-api.md, ADR 0010)."
---

# Session names

Each agent session (`F6` to `F10`, see [Threads](../threads.md)) gets a name of one word that tells what
its conversation is about, for example `parser`. The name stands next to the key of the session.

**When a name comes.** As soon as the thread of a Claude session has a question of the user and one
message of the agent, intermediate or final, zendo asks a light model for a name in the background,
outside the thread of the session; it does not wait for the end of the turn. While the topic is not
clear yet, zendo asks again; see [Session names: rename](session-names-rename.md). The config names
the model for each agent in `[name_models]` of `~/.zendo/config.toml`; by default `haiku` for Claude
and `luna` for Codex. No two agent sessions have the same name.

**Who answers.** The question needs one word back, so zendo asks the model API of the agent that
the config names when the agent has a credential for it: an API key, the `ant` sign-in or
`claude -p` for Claude, an API key for Codex. Without one, zendo asks the agent in a throwaway
session. The names of pasted images ([Chat](../chat.md), CLAUDE-37) take the same way; see
[Session names: API](session-names-api.md) and ADR 0010.

**How long a name stays.** A name belongs to one Claude session, the conversation with one session
ID. It does not change while that session lasts: zendo asks no more questions for it. It stays when
zendo starts again, and when the user continues the session in Claude Code. A new Claude session,
such as a new thread in the chat or `/clear` in Claude Code, has no name until its own first agent
message. zendo learns about a new Claude session from the agent and from the hooks of each run of
Claude Code (SESSIONS-18), so it needs no periodic check (Design).

**Where a name shows.** After the key of the session: on its button, in the bottom line of the
session view, and in the top line; see [Session names: buttons](session-names-buttons.md). The
terminals get no name.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| NAMES-1  | Event      | When the thread of a Claude session of an agent session has a question of the user and a message of the agent, the system shall ask for a name of one word that tells what the conversation is about, also while the turn runs. | User, 2026-11-14 |
| NAMES-2  | Ubiquitous | The system shall give each agent session a name that no other agent session has. | User, 2026-11-14 |
| NAMES-3  | State      | While a Claude session lasts, the system shall keep the name of its agent session, also after zendo starts again. | User, 2026-11-14 |
| NAMES-17 | State      | While the Claude session of an agent session has a name, the system shall not ask for a name of that session. | User, 2026-11-17 |
| NAMES-4  | Event      | When a new Claude session starts in an agent session, the system shall remove the name of the agent session within 1 s. | User, 2026-11-14 |
| NAMES-5  | Ubiquitous | The system shall give no name to a terminal. | User, 2026-11-14 |
| NAMES-12 | Ubiquitous | The system shall get the names from the model that `[name_models]` of the config names for the agent: by default `haiku` for Claude and `luna` for Codex. | User, 2026-11-14 |

### Acceptance tests

The tests are in `tests/session_names.rs`, with the mock agent and the mock command line of
[Sessions](../sessions.md). The mock agent gives the same list of names to each question for a name.

**`the_first_agent_message_names_the_session`** (mock agent; verifies NAMES-1, NAMES-5, NAMES-6,
NAMES-9)

- Given agent session 1 with no name.
- When the user asks a question, and the mock agent sends an intermediate message and keeps the turn
  running.
- Then before the end of the turn, and not before the message, the button shows `F6` and the first
  name of the mock agent, in the row above the header; the thread has only the question and the
  agent text; the `F5` button shows `F5` alone.

**`two_sessions_get_two_names`** (mock agent; verifies NAMES-2)

- Given agent session 1 with the first name of the mock agent.
- When agent session 2 gets its first answer.
- Then agent session 2 gets the second name of the mock agent.

**`the_name_stays_after_a_restart`** (mock agent; verifies NAMES-3)

- Given agent session 1 with a name, and zendo quits.
- When zendo starts again.
- Then the button of agent session 1 shows the same name, before any question.

**`a_named_session_keeps_its_name`** (mock agent; verifies NAMES-3, NAMES-17)

- Given agent session 1 with a name.
- When the conversation grows by more than 600 characters in two more turns.
- Then zendo asks no more questions for a name, and the button shows the same name.

**`a_new_claude_session_clears_the_name`** and **`a_new_thread_clears_the_name`** (mock agent,
mock command line; verify NAMES-4)

- Given agent session 1 with a name.
- When the mock command line reports a session start after `/clear`, or the user starts a new thread.
- Then within 1 s the button shows `F6` alone.

**`a_light_model_names_the_session`** (mock agent; verifies NAMES-12, NAMES-13)

- Given a config model that the mock agent has, or one that it does not have.
- When agent session 1 gets its first answer.
- Then the mock agent names the session with that model, or with its default model.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| NAMES-7  | Event      | When the user continues a Claude session in Claude Code, the system shall keep its name. | User, 2026-11-14 |
| NAMES-9  | Ubiquitous | The system shall get a name without a message in the thread of the agent session. | Design |
| NAMES-13 | Unwanted   | If the agent has no model of the name in the config, or the config names none, then the system shall get the name from the default model of the agent. | Design |

Tests of NAMES-7 and NAMES-6 in `tests/session_names.rs`: `the_bottom_line_shows_the_name`.
