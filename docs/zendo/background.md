---
status: "approved"
approved: "2026-11-18"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "interactivity.md"
next: "terminal/terminal.md"
level: "the background processes of the agent and of Claude Code: that they still run, which ones, their end, their stop, and the quit"
note: "Agent draft from the request of the user on 2026-11-18 in chat: show that background processes still run, and which ones; much asked for. It replaces ACT-12, INTER-4 and INTER-12, which are retired. Approved by the user on 2026-11-18."
---

# Background processes

The agent often starts a command that goes on after its step, such as a dev server, a watch, a
long test run or a monitor. The user must see at any time that such a process still runs, and which
one, also after the turn ended. zendo finds these processes itself, in the processes that it
started: the agent and each Claude Code session
([ADR 0016](../architecture/decisions/0016-background-processes-from-the-process-tree.md)). So this
works on ACP v1 and v2, in a Claude Code session, and with each agent.

**What counts.** A background process is a command of the agent or of Claude Code that still runs
when no step runs it: its step ended, or it was started in the background. A process that the agent
needs for itself, such as an MCP server, does not count.

**The background line.** While at least one background process of the agent session runs, the chat
pane shows one line above the chat input, also between turns:
`⏵ 2 running · npm run dev 12 min · cargo watch -x test 3 min`. Each process has its command, cut to
the width, and the time since its start. When the agent named the command in a step, the line
shows the same text as the step. A click on a name opens its step in the transcript
([Interactivity of the chat](interactivity.md)); for a process without a step, the click shows its
whole command and its folder.

**Other sessions.** The button of an agent session with background processes shows their number,
such as `F7 ⏵2`, so that the user sees them from each session.

**The end.** When a background process ends, the line drops it, and the chat shows one line with
its command, its exit code when zendo knows it, and how long it ran: `npm run dev ended · exit 1 ·
14 min`.

**Stop one.** `×` at a name in the background line, or `x` on its step in the transcript, stops only
that process and the processes that it started; the turn and the agent go on. zendo asks the
process to end, and ends it by force when it does not end within 5 s (Design).

**When the agent goes.** Background processes can outlive the agent: when the agent stops or a
Claude Code session ends, zendo keeps them in the line until they end. When zendo goes back from a
Claude Code session to the agent ([Back to the agent](back-to-the-agent.md)), the processes stay.

**Quit.** When the user quits zendo while background processes run, zendo asks first, as for the
terminals (SESSIONS-6): stop them, or leave them running. The default answer stops them (User,
2026-11-18).

## Architecturally significant requirements

| ID   | Pattern    | Requirement | Source |
| ---- | ---------- | ----------- | ------ |
| BG-1 | State      | While a background process of the agent or of a Claude Code session of an agent session runs, the system shall show its command and its time in one background line of the chat of that session, also between turns. | User, 2026-11-18 |
| BG-2 | Ubiquitous | The system shall find the background processes in the processes that zendo started for the agent and for each Claude Code session, with each agent and on each ACP version. | User, 2026-11-18; Design |
| BG-3 | Event      | When the user stops one background process, the system shall end only that process and the processes that it started, and the turn and the agent shall go on. | User, 2026-11-18 |
| BG-4 | Unwanted   | If background processes run when the user quits zendo, then the system shall ask the user whether to stop them or to leave them running, with stop as the default answer. | User, 2026-11-18 |

### Acceptance tests

Tests: `tests/background.rs`, with the mock agent and the mock command line. Each test starts real
child processes in a folder of its own.

**`a_background_process_shows_after_the_turn`** (mock agent; verifies BG-1, BG-2)

- Given a turn of the mock agent with a step that starts `sleep 60` and leaves it running.
- When the turn ends.
- Then the background line shows `⏵ 1 running · sleep 60` and its time, and the next draw still
  shows it.

**`a_claude_code_session_shows_its_background_process`** (mock command line; verifies BG-1, BG-2)

- Given a mock command line that starts `sleep 60` in a process group of its own.
- Then the chat of its agent session shows the background line with `sleep 60`.

**`one_process_stops_and_the_turn_goes_on`** (mock agent; verifies BG-3)

- Given a running turn, and two background processes of the agent.
- When the user clicks `×` at the first one.
- Then the first process and its children end, the second one runs, and the turn still runs.

**`quit_asks_about_background_processes`** (mock agent; verifies BG-4)

- Given one background process of the agent.
- When the user quits zendo.
- Then zendo asks whether to stop it or to leave it running, and `enter` stops it.

## Detailed requirements

| ID   | Pattern    | Requirement | Source |
| ---- | ---------- | ----------- | ------ |
| BG-5 | Ubiquitous | The system shall not count a process that the agent needs for itself, such as an MCP server, as a background process. | Design |
| BG-6 | State      | While an agent session that the user does not see has background processes, the system shall show their number on its button. | User, 2026-11-18 |
| BG-7 | Event      | When a background process ends, the system shall remove it from the background line and show one line in the chat with its command, its exit code when known, and the time that it ran. | Design |
| BG-8 | Event      | When the user clicks a name in the background line, the system shall open its step in the transcript, or show its whole command and its folder when it has no step. | Design |
| BG-9 | Unwanted   | If a stopped process does not end within 5 s, then the system shall end it by force. | Design |
| BG-10 | State     | While a background process runs after its agent stopped or its Claude Code session ended, the system shall keep it in the background line until it ends. | Design |

Retired: ACT-12 (the background line on ACP v2 only), INTER-4 and INTER-12 (the stop of one task on
ACP v2 only), because BG-1 to BG-3 cover them on every version (User, 2026-11-18).

## Open points

None. The user answered the open points on 2026-11-18.
