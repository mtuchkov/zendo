---
status: "approved"
approved: "2026-11-17"
date: "2026-11-17"
sources:
  - "The user, 2026-11-17: the message of the user in the chat shall be per session; the chat text box is a part of the session, and the chat pane is a container for the session tabs and the active session."
  - "The user, 2026-11-17: the only error seen is the shared chat input on a switch of the session; keep the input of each session across a restart of zendo."
  - "../../zendo/threads.md (CLAUDE-43 to CLAUDE-47)"
  - "../../zendo/chat-chips.md (CLAUDE-48 to CLAUDE-56): the chip window and the choice of the open document live in the input of a session."
  - "ADR 0003 (state in the repository), ADR 0005 (sessions in own terminals), ADR 0008 (app state in sub-states)"
---

# 0011 The chat input belongs to the agent session

## Context

zendo has five agent sessions (CLAUDE-5). Each session has its thread in `Threads`, its Claude Code
terminal in the sessions of ADR 0005, and its last document in the saved state (FILES-54). The
chat input has one `Composer` for the whole app (ADR 0008). The composer holds the text, the chips,
the files and images for the next prompt, the context (for example the delta), the default
question, and the `alt-o` pick.

When the user switches the session, zendo closes the suggestions and keeps the rest. So the text
and the chips that the user wrote for one session go to the next question of another session. The
user saw this error. The chip window and the choice of the open document (chat-chips.md) also live
in the composer, so the same error would touch them.

The transcript cursor and the scroll of the chat pane are shared too. Each frame builds the rows
of the transcript from the active thread, and a switch turns the transcript off, so they cause no
error now.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------ | ------------------ |
| Keep one composer; clear it at a switch | A small change. | The user loses the text of a session. |
| One composer for each session in a chat pane container | Each session keeps its input; the composer does not change. | About 130 call sites change to the composer of the active session. |
| A full session object: thread, terminal, document, input | One owner for all that a session has. | `Threads` also holds recap and pinned threads that are not sessions; the terminals and the saved state change format; much risk for one error. |

## Decision

The second option is chosen.

- A new sub-state `ChatPane` shall hold one `AgentSession` for each of the five agent sessions,
  and the slash commands of the agent. The agent sends one list for all its sessions, so the list
  stays in the container.
- An `AgentSession` shall hold the `Composer` of the session. The composer shall keep its fields:
  the text, the chips, the files and images, the context, the default question, the `alt-o` pick,
  and, from chat-chips.md, the chip window and the choice of the open document.
- The active session shall stay `state.session`, the one value that zendo saves. `App::composer`
  and `App::composer_mut` shall give the composer of the active session; the call sites shall use
  them.
- An event that changes a chip or the input of a thread shall go to the composer of its own
  session: the name of a pasted image goes to the session whose input has the chip (CLAUDE-47), and
  the queued questions go back into the input of the session of the thread (CLAUDE-34). An action
  of the user goes to the active session.
- `Threads`, the transcript cursor, the scroll, the terminals and the last documents shall not
  change.
- zendo shall keep the draft of each session in `.zendo/db.json` (ADR 0003): the text, the chips
  with their labels, contexts and files, the context, and the default question. zendo shall write
  it at the quit, at a switch of the session, at a send, and 2 s after the last change of the
  input, so that a crash loses at most 2 s of typing.
- A chip whose file is gone shall stay in the draft; the chat input shows it in red (CLAUDE-46).
  zendo shall not keep the chip window, the suggestions, the `alt-o` pick, or the
  choice of the open document: after a restart, each session starts with `[x] open document`.

## Consequences

- No behavior changes except at a switch of the session and at a restart.
- The drafts, with the text of their chips, stay out of git: `.zendo` has its own `.gitignore`
  (ADR 0003), and zendo writes `db.json` through a temp file.
- `db.json` gets a new field. An old `db.json` without it loads with empty inputs; an older zendo
  ignores the field.
- The image names (`image_names.rs`) and `queue_to_input` look up the session of the chip or the
  thread, not the active one.
- A pasted image in `.zendo/pastes` stays on the disk while a draft holds its chip.
- `Threads.shown`, the pinned thread and the transcript stay in `Threads`. A later decision can
  move them into `ChatPane` when a new error asks for it.

## Verification

- `tests/chat_flows.rs`: a text and a chip typed in session 1 stay there when the user goes to
  session 2 and back; session 2 starts empty (CLAUDE-43, CLAUDE-44).
- `tests/chat_flows.rs`: a draft comes back after a new `App` on the same repository; a chip of a
  gone file shows in red (CLAUDE-45, CLAUDE-46).
- `tests/chat_flows.rs`: the name of an image pasted in session 1 labels its chip while session 2
  is active (CLAUDE-47); `esc` puts the queue of session 1 into the input of session 1.
- `src/app/chat_pane_tests.rs`: the save and the load of the drafts.

## Decision record

Proposed by the agent on 2026-11-17 from the requests of the user in the chat. The user chose to
keep the drafts across a restart, and a red chip for a gone file, and approved the record on 2026-11-17.
