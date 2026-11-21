---
status: "approved"
approved: "2026-11-18"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "codex-cli.md"
next: "zendo-session.md"
level: "when zendo ends a quiet Claude Code session by itself, so that the thread goes back to the agent of the chat through ACP"
note: "Draft from the request of the user on 2026-11-18 in chat: when the user is back in the chat for long enough, catch a good pause of the Claude Code session and switch the thread back to ACP. It changes SESSIONS-1. Approved by the user on 2026-11-18."
---

# Back to the agent

While a Claude Code session runs, its chat talks to Claude Code, not to the agent of the chat
(SESSIONS-4), and the chat shows only what the hooks give
([Activity of a Claude Code session](cli-activity.md)). When the user went back to the chat and
stays there, the session view is not needed any more. zendo then waits for a good pause of Claude
Code, ends it, and opens the same thread through ACP again (SESSIONS-13). The conversation goes
on, with the full detail of [Activity of the agent](activity.md). `ctrl-o` starts Claude Code again
with a resume (SESSIONS-7).

**Long enough.** The user left the session view (`alt-q`) at least 5 min ago, and did not open it
again (User, 2026-11-18). `back_to_agent = false` in `~/.zendo/config.toml` turns the switch off.

**A good pause.** Claude Code is idle: its turn ended at least 30 s ago, and no hook came since
(Design). A pause is not good, and zendo waits for the next one, while one of these is true:

- Claude Code waits for an answer or a permission (CLIACT-3).
- A tool, a subagent or a background task of Claude Code runs: a background task would end with
  the process ([Background processes](background.md)).
- The user typed keys in the session view after the last prompt that Claude Code got: the text
  that the user did not send would be lost. zendo knows the keys that it gave to Claude Code; it
  does not read the screen (CLIACT-2).
- The user types in the chat input now, or typed there in the last 10 s.

**The switch.** zendo ends Claude Code as `/exit` would, and opens the thread through ACP. The chat
shows one line: `back to the chat agent · ctrl-o opens Claude Code again`. A question that the
user sends after the switch goes to the agent. When the agent cannot load the thread, zendo starts
Claude Code again with a resume and says so.

## Architecturally significant requirements

| ID       | Pattern  | Requirement | Source |
| -------- | -------- | ----------- | ------ |
| RETURN-1 | Event    | When the user has been out of the session view of a running Claude Code session for 5 min and Claude Code has been idle for 30 s, the system shall end the Claude Code session and open its thread through ACP. | User, 2026-11-18 |
| RETURN-2 | Unwanted | If Claude Code waits for an answer, runs a tool, a subagent or a background task, or has keys of the user that it did not get as a prompt, then the system shall not end it, and shall wait for the next good pause. | User, 2026-11-18; Design |
| RETURN-3 | Unwanted | If the agent cannot load the thread after the switch, then the system shall start Claude Code again with a resume and show the reason. | Design |

### Acceptance tests

Tests: `tests/back_to_the_agent.rs`, with the mock agent, the mock command line, and a fixed clock.

**`a_quiet_session_goes_back_to_the_agent`** (mock agent and mock command line; verifies RETURN-1)

- Given a Claude Code session that the user left 5 min ago, with a turn that ended 30 s ago.
- When the clock moves on.
- Then the mock command line ends, the agent loads the thread, the chat shows
  `back to the chat agent`, and the next question goes to the agent.

**`a_busy_session_stays`** (mock agent and mock command line; verifies RETURN-2)

- Given a Claude Code session that the user left 5 min ago, with a background task that runs, and
  another one with keys that the user typed in the session view after the last prompt.
- When the clock moves on by 10 min.
- Then both mock command lines still run.

**`a_failed_load_starts_claude_code_again`** (mock agent and mock command line; verifies RETURN-3)

- Given a quiet Claude Code session, and a mock agent that cannot load its thread.
- When the switch comes.
- Then the mock command line runs again with a resume, and the chat shows the reason.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| RETURN-4 | Unwanted   | If the user types in the chat input, or typed there in the last 10 s, then the system shall not end the Claude Code session. | Design |
| RETURN-5 | Event      | When the system ends a Claude Code session for RETURN-1, the system shall show `back to the chat agent · ctrl-o opens Claude Code again` in its chat. | Design |
| RETURN-6 | Event      | When the user opens the session view again before the switch, the system shall start the 5 min again from the next `alt-q`. | Design |
| RETURN-7 | Optional   | Where `~/.zendo/config.toml` has `back_to_agent = false`, the system shall not end a Claude Code session for RETURN-1. | User, 2026-11-18 |

## Open points

None. The user answered the open points on 2026-11-18.
