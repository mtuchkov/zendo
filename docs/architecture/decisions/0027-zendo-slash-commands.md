---
status: "approved"
approved: "2026-11-19"
date: "2026-11-19"
note: "Draft of the decision of the user on 2026-11-19 in the chat about note to spec; approved by the user on 2026-11-19. The numbers 0024 to 0026 are taken by records of other sessions that are not in main yet."
sources:
  - "../../zendo/chat.md (CLAUDE-28): zendo passes the slash commands of the agent through"
  - "../../zendo/chat-history.md (HISTORY-2, HISTORY-8 to HISTORY-10): `ctrl-n`, `/clear` and `/new`"
  - "../../zendo/note-to-spec.md (SPEC-10): with no free agent session, `/new` and the task go into the chat input"
  - "The user, 2026-11-19: zendo handles `/new <text>`"
---

# 0027 Slash commands of zendo

## Context

When no agent session is free, note to spec puts the task into the chat input of the active session
(SPEC-10). The user then chooses: send it into the current thread, or into a new one. One line in the
input should carry that choice, so that `enter` does it.

`/clear <task>` does not work. zendo sends an input that starts with `/` to the agent as the user
typed it, without the context of the input (CLAUDE-28, `App::ask_question`), so the instructions of
the task get lost. Claude Code takes no text after `/clear`. zendo only moves the history for it
(HISTORY-2).

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| `ctrl-n`, then `enter` | No new rule; `ctrl-n` keeps the input | Two steps; the choice is in a flash, not in the input |
| zendo reads `/clear <text>` | One known name | One name for two commands: the one of the agent and the one of zendo do different things |
| A command of zendo, `/new <text>` | One step; the choice is visible in the input and can be deleted | The first command that zendo does not pass through; a clash with an agent command of the same name |

## Decision

The user chose `/new <text>` and approved this record on 2026-11-19.

The chat input shall do a slash command of zendo itself and shall not send it to the agent. zendo
shall list its commands with the commands of the agent, and a command of zendo shall win over a
command of the agent with the same name. Every other slash command goes to the agent as typed.

`/new <text>` starts a new thread as `ctrl-n` does, then sends `<text>` as a question with the
context of the input (HISTORY-8). It waits in the queue while the session answers (HISTORY-10).

## Consequences

- The send path of the chat input checks the commands of zendo before the test for `/` in
  `App::ask_question`.
- The slash suggestions (`src/app/slash.rs`) merge the commands of zendo into the list of the agent.
- An agent with a `/new` of its own (Codex) cannot get it from the zendo chat. zendo's `/new` does
  the same as that command, a new conversation, so the user loses nothing.
- Each later command of zendo needs a requirement and a name that the agents do not use.

## Verification

- A test of `chat-history.md`: `/new <text>` starts a new thread and sends the text with its
  context; `/new` alone sends nothing; `/new` while the session answers waits for the turn end.
- The test of CLAUDE-28: `/new` shows in the suggestions, and an agent command `/new` does not.
