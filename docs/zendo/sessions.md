---
status: "draft"
revision: "4"
date: "2026-11-13"
parent: "zendo.md"
prev: "claude.md"
next: "handoff.md"
level: "Claude Code sessions that zendo runs: open, leave, states, up to five at a time"
note: "Agent draft from the requests of the user on 2026-11-12. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Sessions

A *Claude Code session* is a Claude Code process that zendo runs for one of its five agent sessions
(`F6` to `F10`, see [Claude](claude.md)), in a terminal of its own inside zendo. It keeps running
when the user goes back to zendo: one researches while the user discusses in another. See
[ADR 0005](../architecture/decisions/0005-sessions-in-own-terminals.md).

**Open and leave.** `ctrl-o` in the chat opens the Claude Code session of the active agent session.
zendo starts it first if necessary. While the chat answers, `ctrl-o` waits for the end of the turn (see [Hand-off during a turn](handoff.md)). The session view takes the whole screen, with a bottom line of
zendo. All input goes to Claude Code, except `alt-q` (back to zendo; the same key leaves a terminal
of [Terminal](terminal/terminal.md)) and `F5` to `F10` (another session, or the `F5` terminal). When Claude
Code exits, zendo goes back.

**States.** Each running Claude Code session has one state: *works*, *waits* (for an answer or a
permission of the user), or *idle* (the turn ended). The session buttons, the chat of the agent
session and the bottom line of the session view show it. Claude Code itself tells zendo the state. When a turn ends while the user is elsewhere, the bell rings and the
button blinks (see [Attention](attention.md)).

**One writer for a thread.** While a Claude Code session runs, a question in its chat goes to Claude
Code, not through ACP; Claude Code queues it while it works. When the Claude Code session ends, zendo
opens the agent session again, so the chat shows what happened there.

**Quit.** A Claude Code session lives while zendo runs; the conversation stays. If sessions run,
the first quit asks. With Codex these keys are off (CLAUDE-14).

## Architecturally significant requirements

| ID         | Pattern    | Requirement                                                                                                                    | Source           |
| ---------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------ | ---------------- |
| SESSIONS-1 | Event      | When the user leaves a Claude Code session, the system shall keep it running until it ends or zendo quits.                    | User, 2026-11-12 |
| SESSIONS-2 | Ubiquitous | The system shall run up to five Claude Code sessions at a time, one for each agent session.                                  | User, 2026-11-12 |
| SESSIONS-3 | Ubiquitous | The system shall run each Claude Code session in a terminal that zendo owns, without tmux, a multiplexer, or a server process. | User, 2026-11-12 |
| SESSIONS-4 | State      | While a Claude Code session runs, the system shall send the questions of the chat of its agent session to that Claude Code session, not to the agent. | User, 2026-11-12 |
| SESSIONS-5 | Event      | When the state of a Claude Code session changes, the system shall show the new state within 1 s.                              | Design           |
| SESSIONS-6 | Unwanted   | If the user quits zendo while a Claude Code session runs, then the system shall ask the user before it stops the sessions.     | Design           |

### Acceptance tests

The **mock command line** replaces `claude`: it prints a counter each 100 ms, reports a state on a
key, echoes a line, and exits on `q`. The tests are in `tests/sessions.rs`.

**`left_sessions_keep_running`** (mock command line; verifies SESSIONS-1, SESSIONS-2, SESSIONS-3)

- Given two agent sessions with session IDs.
- When the user opens and leaves the Claude Code session of each, waits, and opens the first again.
- Then two processes of zendo run, both counters grew, and the view shows the last counter.

**`chat_of_an_open_session_goes_to_claude_code`** (mock agent and mock command line; verifies SESSIONS-4)

- Given a running Claude Code session.
- When the user sends a question in its chat.
- Then the mock agent gets no prompt, and the mock command line gets the question.

**`state_shows_within_a_second`** (mock command line; verifies SESSIONS-5)

- Given a running Claude Code session that the user left.
- When the mock command line reports *waits*.
- Then within 1 s the session button shows *waits*.

**`quit_asks_while_sessions_run`** (mock command line; verifies SESSIONS-6, SESSIONS-19)

- Given a running Claude Code session.
- When the user quits zendo twice.
- Then zendo asks the first time, and stops the session and quits the second time.

## Detailed requirements

| ID          | Pattern    | Requirement                                                                                                                                         | Source           |
| ----------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| SESSIONS-7  | Event      | When the user presses `ctrl-o` in the chat, the system shall show the Claude Code session of the active agent session, and start it first if it does not run. | CLAUDE-10 |
| SESSIONS-8  | Event      | When the user opens a Claude Code session again, the system shall show the screen of Claude Code as it is now.                                     | User, 2026-11-12 |
| SESSIONS-9  | State      | While the session view is open, the system shall send all keys and paste to Claude Code, except the leave key and the keys of the agent sessions.  | Design           |
| SESSIONS-10 | Event      | When the user presses `alt-q` in the session view, the system shall show the screen from where the user opened the session.                         | User, 2026-11-12 |
| SESSIONS-11 | Event      | When the user presses the key of an agent session in the session view, the system shall show its Claude Code session if it runs, else the chat on that agent session. | Design |
| SESSIONS-12 | Event      | When Claude Code exits while its session view is open, the system shall show the screen from where the user opened the session.                    | Design           |
| SESSIONS-13 | Event      | When a Claude Code session ends, the system shall open its agent session again, so that the chat shows the messages from Claude Code.               | Design           |
| SESSIONS-14 | Ubiquitous | The system shall show the state of each running Claude Code session on its session button and in the bottom line of the session view.             | Design           |
| SESSIONS-16 | Event      | When the terminal changes size, the system shall give the new size to each Claude Code session.                                                    | Design           |
| SESSIONS-17 | Unwanted   | If Claude Code does not report a state, then the system shall show *works* while the session prints and *idle* otherwise.                          | Design           |
| SESSIONS-18 | Ubiquitous | The system shall not change the settings files of Claude Code of the user.                                                                          | Design           |
| SESSIONS-19 | Event      | When the user quits again within 5 s of the question, the system shall stop each Claude Code session and quit.                                     | Design           |
| SESSIONS-20 | State      | While a Claude Code session runs, the system shall show its state at the end of the chat of its agent session, with a spinner while it works.      | User, 2026-11-12 |

Test of SESSIONS-20: `chat_shows_the_state_of_its_claude_code_session` in `tests/sessions.rs`.

Retired in revision 4: SESSIONS-15 (zendo refused `ctrl-o` while the chat answered). [Hand-off during a turn](handoff.md) replaces it. Revision 3 changed SESSIONS-4 and added SESSIONS-20.

## Open points

| Open question                                                                                                                                         | Affects                 | Answered by |
| ----------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- | ----------- |
| The session view takes the whole screen. Should it take the place of the chat pane beside the note instead?                                           | SESSIONS-9, PANES-7     | User        |
