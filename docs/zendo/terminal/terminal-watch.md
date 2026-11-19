---
status: "draft"
revision: "1"
date: "2026-11-17"
parent: "terminal.md"
prev: "terminal-output.md"
next: "terminal-mouse.md"
level: "watched commands: the play-and-watch button, what output goes to the agent, push and pull, the check-in, the end of a watch"
note: "Agent draft from the requests of the user on 2026-11-17. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. See ADR 0018."
---

# Watched commands

The agent proposes a command, the user runs it, and the agent must see the output, also while the
command runs. A *watched command* gives the output to the agent without `alt-o`
([Terminal output](terminal-output.md)). The user still starts each command (TERM-2). See
[ADR 0018](../../architecture/decisions/0018-command-results-to-the-agent.md).

**Two buttons.** Each shell block shows two buttons ([Shell blocks](shell-blocks.md)):

`▶` (`F5`) puts the command into a terminal, as today. `▶▶` (`alt-F5`) runs it and *watches* it
for the agent session of the answer that holds the block. `▶` is the way to run a command without
a watch. Only the output of that command goes to
the agent, and only to that session; a typed command or a command of `▶` does not.

**Push** wakes the agent. zendo sends a message of zendo to the thread of the session when the
command ends, and with the new output after each progress interval while it runs. During a turn,
zendo keeps the output and sends one message when the turn ends. The message is marked as an
event of zendo, not as a question of the user.

**Pull** serves the agent in a turn. The tool `command_result` of the MCP server `zendo`
([zendo session](../zendo-session.md)) gives the new output, and can wait a short time for more.
Push and pull share one position in the output, so each line goes to the agent once. Long output
keeps the last 500 lines (TERM-32).

**The check-in.** A watch has no time limit. After each check-in interval (10 min by default),
zendo asks the agent whether the run goes well: how long the command runs, how long ago the last
output came, and the new output. The check-in comes also when the command prints nothing, so the
agent sees a command that hangs. The agent answers in the chat; it can end the watch with the tool
`end_watch`, for example for a dev server that runs well. It cannot stop the command.

**The end.** A watch ends with the command, when the user clicks the line of the watch in the chat,
when the agent calls `end_watch`, or when zendo quits. The command continues after the watch. While a session runs in Claude Code
([Sessions](../sessions.md)), zendo does not type into it: the push waits for the chat.

`Run in zendo shell` on a question of the agent starts a watch too
([Questions of the agent](../questions.md), ASK-4).

**Safety.** Each watch starts with a choice of the user; the agent cannot start one. The output can
hold secrets and text that asks the agent to act. zendo sends it as data, and each command of the
agent waits for the user (ASK-2).

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TERM-55 | Event      | When the user presses `alt-F5` or clicks `▶▶` on a shell block, the system shall run the command as TERM-21 does, and watch it for the agent session of the answer that holds the block. | User, 2026-11-17 |
| TERM-56 | Event      | When a watched command ends, the system shall send to the thread of its session, as a message of zendo, the command, the exit code and the output that the agent did not get yet. | User, 2026-11-17 |
| TERM-57 | State      | While a watched command runs, the system shall send its new output to the thread of its session after each progress interval in which new output came. | User, 2026-11-17 |
| TERM-58 | State      | While the agent of a session is in a turn, the system shall keep the output for that session and send it as one message when the turn ends. | User, 2026-11-17 |
| TERM-59 | Event      | When the agent calls the tool `command_result` of the MCP server `zendo`, the system shall return the output of the watched commands of its session that the agent did not get yet, and their state. | User, 2026-11-17 |
| TERM-60 | Ubiquitous | The system shall give each line of a watched command to the agent at most once, by push or by `command_result`. | User, 2026-11-17 |
| TERM-61 | Ubiquitous | The system shall mark each message of zendo in a thread so that the agent can tell it from a message of the user, and shall keep the output from closing the mark. | Design |
| TERM-62 | Ubiquitous | The system shall end a watch only when its command ends, when the user or the agent stops the watch, or when zendo quits, and shall not stop the command at the end of a watch. | User, 2026-11-17 |
| TERM-70 | State      | While a watched command runs, the system shall send a check-in to the thread of its session after each check-in interval, with the run time, the time since the last output and the new output, and ask the agent whether the run goes well. | User, 2026-11-17 |

### Acceptance tests

The tests are in `tests/terminal_watch.rs`. They use the mock agent, and `bash` as the shell.

**`double_play_runs_and_sends_the_result_to_its_thread`** (mock agent; verifies TERM-55, TERM-56, TERM-61)

- Given an answer of agent session 2 with a block `echo zendo-$((40+2)); false`.
- When the user clicks `▶▶` on the block.
- Then the `F5` terminal shows `zendo-42`, and session 2 gets one message of zendo with the
  command, the exit code `1` and `zendo-42`. Session 1 gets nothing.

**`only_the_watched_command_goes_to_the_agent`** (mock agent; verifies TERM-25, TERM-55)

- Given an answer with a block `echo one`.
- When the user presses `F5` and `enter` on it, then types `echo two` and `enter`.
- Then the agent gets no message.

**`progress_goes_to_the_agent_while_the_command_runs`** (mock agent; verifies TERM-57)

- Given a progress interval of 1 s, and a watched block
  `echo a; sleep 2; echo b; sleep 2; echo c`.
- When the command runs to its end.
- Then the agent gets `a`, then `b`, then a message with the exit code `0` and `c`.

**`output_waits_for_the_end_of_the_turn`** (mock agent; verifies TERM-58)

- Given an agent in a turn that lasts 3 s, and a watched command that ends within 1 s.
- When the turn ends.
- Then the agent gets one message with the result, after the end of the turn.

**`push_and_pull_give_each_line_once`** (mock agent; verifies TERM-59, TERM-60)

- Given a watched command that printed `a` and `b`, and a push with `a`.
- When the agent calls `command_result`.
- Then the result holds `b` and not `a`, and the next push holds no `b`.

**`a_silent_command_gets_a_check_in`** (mock agent; verifies TERM-70)

- Given a check-in interval of 2 s, and a watched block `sleep 5`.
- When 3 s pass.
- Then the agent got one check-in with the run time and no output, and `sleep` still runs.

**`a_click_on_the_watch_line_stops_the_watch`** (mock agent; verifies TERM-62, TERM-67)

- Given a progress interval of 1 s, and a watched block `echo a; sleep 3; echo late`.
- When the agent got `a`, and the user clicks the line of the watch in the chat.
- Then the agent gets no `late`, and `late` shows in the terminal.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TERM-63 | Ubiquitous | The system shall show a play-and-watch button `▶▶` after `▶` on the first line of each shell block. | User, 2026-11-17 |
| TERM-64 | Event      | When the agent calls `command_result` with a wait, the system shall answer when the command ends, when new output comes, or when the wait ends, whichever is first. | User, 2026-11-17 |
| TERM-65 | Event      | When the system sends a message of zendo, the system shall show it in the chat as one line with the command, its state and the number of lines. | Design |
| TERM-66 | State      | While a session runs in Claude Code in a terminal, the system shall keep the output for that session and send it when the session comes back to the chat. | Design |
| TERM-67 | Event      | When the user clicks the line of a watch in the chat, the system shall send the output that the agent did not get yet, and end the watch. | User, 2026-11-17 |
| TERM-69 | Ubiquitous | The system shall take the progress interval from the config, 30 s by default, and the check-in interval, 10 min by default. | User, 2026-11-17 |
| TERM-71 | Event      | When the agent calls the tool `end_watch` of the MCP server `zendo` for a watch of its session, the system shall end the watch and show in the chat that the agent ended it. | Design |
