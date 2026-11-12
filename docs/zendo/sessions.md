---
status: "draft"
revision: "2"
date: "2026-11-12"
parent: "zendo.md"
prev: "claude.md"
next: "chat.md"
level: "Claude Code sessions that zendo runs: open, leave, states, up to five at a time"
note: "Agent draft from the requests of the user on 2026-11-12. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Sessions

zendo has five agent sessions, `F6` to `F10` (see [Claude](claude.md)). Each one is a Claude Code
session. A *Claude Code session* in this document is a Claude Code process that zendo runs for one
agent session, in a terminal of its own, inside the zendo process. It keeps running when the user
goes back to zendo: one session does research while the user discusses in another. Thus up to five
Claude Code sessions run at a time, one for each agent session. zendo needs no other program for
this. See [ADR 0005](../architecture/decisions/0005-sessions-in-own-terminals.md).

**Open and leave.** `ctrl-o` in the chat opens the Claude Code session of the active agent session.
If it does not run, zendo starts it with the session ID of the agent session. The session view
takes the whole screen; the bottom line belongs to zendo and shows the five sessions and the keys.
The view shows the screen of Claude Code as it is now. All input goes to Claude Code, except
`ctrl-]` (back to zendo) and `F6` to `F10`: the F key of a running Claude Code session shows that
session; the F key of another agent session goes back to the chat on it. When Claude Code exits,
zendo also goes back.

**States.** Each running Claude Code session has one state: *works*, *waits* (for an answer or a
permission of the user), or *idle* (the turn ended). The session buttons of the chat and the bottom
line of the session view show it. Claude Code itself tells zendo the state.

**One writer for a thread.** While a Claude Code session runs, the chat of its agent session does
not send questions; it says that the session is open in Claude Code. When the Claude Code session
ends, zendo opens the agent session again, so the chat shows what happened in Claude Code.

**Quit.** A Claude Code session lives while zendo runs; the conversation stays in Claude Code. If
sessions run when the user quits, zendo asks first: the user quits again to stop them.

**Agents.** These keys need a command line with a resume. With Codex they are off (CLAUDE-14).

## Architecturally significant requirements

| ID         | Pattern    | Requirement                                                                                                                    | Source           |
| ---------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------ | ---------------- |
| SESSIONS-1 | Event      | When the user leaves a Claude Code session, the system shall keep it running until it ends or zendo quits.                    | User, 2026-11-12 |
| SESSIONS-2 | Ubiquitous | The system shall run up to five Claude Code sessions at a time, one for each agent session.                                  | User, 2026-11-12 |
| SESSIONS-3 | Ubiquitous | The system shall run each Claude Code session in a terminal that zendo owns, without tmux, a multiplexer, or a server process. | User, 2026-11-12 |
| SESSIONS-4 | State      | While a Claude Code session runs, the system shall not send the questions of the chat of its agent session to the agent.       | CLAUDE-5, Design |
| SESSIONS-5 | Event      | When the state of a Claude Code session changes, the system shall show the new state within 1 s.                              | Design           |
| SESSIONS-6 | Unwanted   | If the user quits zendo while a Claude Code session runs, then the system shall ask the user before it stops the sessions.     | Design           |

### Acceptance tests

The **mock command line** replaces `claude`: it prints a counter each 100 ms, reports a state on a
key, and exits on `q`. The tests are in `tests/sessions.rs`.

**`left_sessions_keep_running`** (mock command line; verifies SESSIONS-1, SESSIONS-2, SESSIONS-3)

- Given two agent sessions with session IDs, and the mock command line.
- When the user opens the Claude Code session of the first, leaves it, opens the one of the second,
  leaves it, waits, and opens the first again.
- Then two processes run, the counter of each grew while the user was away, and the view shows the
  last counter of the first.

**`chat_of_an_open_session_sends_no_question`** (mock agent and mock command line; verifies SESSIONS-4)

- Given an agent session whose Claude Code session runs.
- When the user writes a question in its chat.
- Then the mock agent gets no prompt, and the chat says that the session is open in Claude Code.

**`state_shows_within_a_second`** (mock command line; verifies SESSIONS-5)

- Given a running Claude Code session that the user left.
- When the mock command line reports *waits*.
- Then within 1 s the session button shows *waits*.

**`quit_asks_while_sessions_run`** (mock command line; verifies SESSIONS-6, SESSIONS-19)

- Given a running Claude Code session.
- When the user quits zendo once, then again.
- Then after the first quit zendo asks and the session runs; after the second, the session stops
  and zendo quits.

## Detailed requirements

| ID          | Pattern    | Requirement                                                                                                                                         | Source           |
| ----------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| SESSIONS-7  | Event      | When the user presses `ctrl-o` in the chat, the system shall show the Claude Code session of the active agent session, and start it first if it does not run. | CLAUDE-10 |
| SESSIONS-8  | Event      | When the user opens a Claude Code session again, the system shall show the screen of Claude Code as it is now.                                     | User, 2026-11-12 |
| SESSIONS-9  | State      | While the session view is open, the system shall send all keys and paste to Claude Code, except the leave key and the keys of the agent sessions.  | Design           |
| SESSIONS-10 | Event      | When the user presses `ctrl-]` in the session view, the system shall show the screen from where the user opened the session.                        | Design           |
| SESSIONS-11 | Event      | When the user presses the key of an agent session in the session view, the system shall show its Claude Code session if it runs, else the chat on that agent session. | Design |
| SESSIONS-12 | Event      | When Claude Code exits while its session view is open, the system shall show the screen from where the user opened the session.                    | Design           |
| SESSIONS-13 | Event      | When a Claude Code session ends, the system shall open its agent session again, so that the chat shows the messages from Claude Code.               | Design           |
| SESSIONS-14 | Ubiquitous | The system shall show the state of each running Claude Code session on its session button and in the bottom line of the session view.             | Design           |
| SESSIONS-15 | Unwanted   | If the chat of an agent session is answering, then the system shall not open its Claude Code session, and shall say why.                            | Design           |
| SESSIONS-16 | Event      | When the terminal changes size, the system shall give the new size to each Claude Code session.                                                    | Design           |
| SESSIONS-17 | Unwanted   | If Claude Code does not report a state, then the system shall show *works* while the session prints and *idle* otherwise.                          | Design           |
| SESSIONS-18 | Ubiquitous | The system shall not change the settings files of Claude Code of the user.                                                                          | Design           |
| SESSIONS-19 | Event      | When the user quits again within 5 s of the question, the system shall stop each Claude Code session and quit.                                     | Design           |

Retired in revision 2: none. Revision 2 changed SESSIONS-2, SESSIONS-4, SESSIONS-7, SESSIONS-11,
SESSIONS-13 to SESSIONS-15 for the five agent sessions of CLAUDE-5.

## Open points

| Open question                                                                                                                                         | Affects                 | Answered by |
| ----------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- | ----------- |
| `ctrl-]` leaves the session view, so that `ctrl-o` keeps its meaning in Claude Code. Is it the right default? The keymap can change it.                | SESSIONS-10             | User        |
| The session view takes the whole screen. Should it take the place of the chat pane beside the note instead?                                           | SESSIONS-9, PANES-7     | User        |
| When a session goes to *waits* while the user is elsewhere: only the button, or also the terminal bell?                                                | SESSIONS-5, SESSIONS-14 | User        |
