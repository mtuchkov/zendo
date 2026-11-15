---
status: "approved"
approved: "2026-11-14"
revision: "1"
date: "2026-11-14"
parent: "zendo.md"
prev: "sessions.md"
next: "handoff.md"
level: "the names of the agent sessions: when they come, how long they stay, where they show"
note: "From the requests of the user on 2026-11-14; the user approved the requirements in the chat. NAMES-8, NAMES-11 to NAMES-13 came later on the same day."
---

# Session names

Each agent session (`F6` to `F10`, see [Claude](claude.md)) gets a name of one word that tells what
its conversation is about, for example `parser`. The name stands next to the key of the session.

**When a name comes.** After the second answer of a Claude session, zendo asks a light model of the
agent for a name in the background, outside the thread of the session. The config names the model
for each agent in `[name_models]` of `~/.zendo/config.toml`; by default `haiku` for Claude and `luna`
for Codex. No two agent sessions have the same name.

**How long a name stays.** A name belongs to one Claude session, the conversation with one session
ID: it stays when zendo starts again, and when the user continues the session in Claude Code. A new
Claude session, such as a new thread in the chat or `/clear` in Claude Code, has no name until its
own second answer. zendo learns about a new Claude session from the agent and from the hooks of each
run of Claude Code (SESSIONS-18), so it needs no periodic check (Design).

**Where a name shows.** The buttons of the agent sessions stand in one row above the header of the
chat, each with its key and its name: `F6 parser·  F7  F8 hooks•`. In a narrow chat the names get
shorter. When the row does not fit even then, and while a terminal shows in the chat pane, the
buttons stand on the right edge, one under the other, with their keys alone, above the terminal
buttons. The name also shows in the bottom line of the session view and in the top line (ATTN-10).
The terminals (`F5` and `[>]`) get no name, and their buttons stay on the right edge (TERM-14).

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| NAMES-1  | Event      | When the agent ends its second answer in a Claude session of an agent session, the system shall give the agent session a name of one word that tells what the conversation is about. | User, 2026-11-14 |
| NAMES-2  | Ubiquitous | The system shall give each agent session a name that no other agent session has. | User, 2026-11-14 |
| NAMES-3  | State      | While a Claude session lasts, the system shall keep the name of its agent session, also after zendo starts again. | User, 2026-11-14 |
| NAMES-4  | Event      | When a new Claude session starts in an agent session, the system shall remove the name of the agent session within 1 s. | User, 2026-11-14 |
| NAMES-5  | Ubiquitous | The system shall give no name to a terminal. | User, 2026-11-14 |
| NAMES-12 | Ubiquitous | The system shall get the names from the model that `[name_models]` of the config names for the agent: by default `haiku` for Claude and `luna` for Codex. | User, 2026-11-14 |

### Acceptance tests

The tests are in `tests/session_names.rs`, with the mock agent and the mock command line of
[Sessions](sessions.md). The mock agent gives the same list of names to each question for a name.

**`the_second_answer_names_the_session`** (mock agent; verifies NAMES-1, NAMES-5, NAMES-6, NAMES-9)

- Given agent session 1 with no name.
- When the user asks one question, and then a second one.
- Then after the second answer, and not before, the button shows `F6` and the first name of the
  mock agent, in the row above the header; the thread has only the two questions and their
  answers; the `F5` button shows `F5` alone.

**`two_sessions_get_two_names`** (mock agent; verifies NAMES-2)

- Given agent session 1 with the first name of the mock agent.
- When agent session 2 gets its second answer.
- Then agent session 2 gets the second name of the mock agent.

**`the_name_stays_after_a_restart`** (mock agent; verifies NAMES-3)

- Given agent session 1 with a name, and zendo quits.
- When zendo starts again.
- Then the button of agent session 1 shows the same name, before any question.

**`a_new_claude_session_clears_the_name`** and **`a_new_thread_clears_the_name`** (mock agent,
mock command line; verify NAMES-4)

- Given agent session 1 with a name.
- When the mock command line reports a session start after `/clear`, or the user starts a new thread.
- Then within 1 s the button shows `F6` alone.

**`a_light_model_names_the_session`** (mock agent; verifies NAMES-12, NAMES-13)

- Given a config model that the mock agent has, or one that it does not have.
- When agent session 1 gets its second answer.
- Then the mock agent names the session with that model, or with its default model.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| NAMES-6  | Ubiquitous | The system shall show the name of an agent session after its key: on its button, in the bottom line of the session view, and in the top line. | User, 2026-11-14 |
| NAMES-7  | Event      | When the user continues a Claude session in Claude Code, the system shall keep its name. | User, 2026-11-14 |
| NAMES-8  | Unwanted   | If the buttons of the agent sessions with their names do not fit into one row, then the system shall shorten the names, and then show the buttons on the right edge of the chat, one under the other, with their keys alone. | User, 2026-11-14 |
| NAMES-9  | Ubiquitous | The system shall get a name without a message in the thread of the agent session. | Design |
| NAMES-10 | Unwanted   | If the system gets no free name from the agent, then it shall show the key alone and ask again after the next answer. | Design |
| NAMES-11 | State      | While a terminal shows in the chat pane, the system shall show the buttons of the agent sessions on the right edge of the chat, one under the other, with their keys alone. | User, 2026-11-14 |
| NAMES-13 | Unwanted   | If the agent has no model of the name in the config, or the config names none, then the system shall get the name from the default model of the agent. | Design |

Tests of the detailed requirements in `tests/session_names.rs`:
`narrow_chats_shorten_the_names_and_then_put_the_buttons_on_the_side` (NAMES-8),
`a_terminal_in_the_chat_pane_puts_the_buttons_on_the_side` (NAMES-11), and
`the_bottom_line_shows_the_name` (NAMES-6, NAMES-7).
