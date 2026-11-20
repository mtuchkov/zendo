---
status: "approved"
approved: "2026-11-19"
revision: "4"
date: "2026-11-19"
parent: "zendo.md"
prev: "chat-messages.md"
next: "activity.md"
level: "the history of the messages of the user: per zendo session, the common history, and the arrows that go through it"
note: "Agent draft of the request of the user on 2026-11-18. Revision 2: text chips in the history, `/clear` as a new thread, and a thread that sees the common history of its start, asked by the user on 2026-11-18. Approved by the user on 2026-11-18. Revision 3, asked and approved by the user on 2026-11-19: `/new <text>`, a command of zendo (HISTORY-8 to HISTORY-10, ADR 0027). HISTORY-8 to HISTORY-10 are new identifiers that the user approved. Revision 4, after the code review of 2026-11-19: the queue keeps `/new` as typed, and a question behind it goes to the new thread (HISTORY-10); `/new` does not go to Claude Code (HISTORY-11, a new identifier); approved by the user on 2026-11-19."
---

# Chat history

The chat input is in [Chat](chat.md). A zendo session is in [zendo session](zendo-session.md).

**Two histories.** zendo keeps the messages that the user sends from the chat input. Each zendo
session has a history of its own: the messages of its current thread. The repository has one
common history. zendo keeps both in `.zendo/db.json`, so they stay after a restart. The common
history keeps the last 1000 messages.

**A new thread.** `ctrl-n`, or the command `/clear`, starts a new thread in the zendo session.
Then zendo moves the history of the session into the common history and puts all the messages in
the order of the time when the user sent them. The new thread starts with an empty history of its
own.

**`/new <text>`.** `/new` is a command of zendo, not of the agent: it starts a new thread as
`ctrl-n` does, and then sends the text after it as a question, with the context of the chat input.
`/new` alone only starts the thread. While the session answers, `/new` waits in the queue as every
question, so it never drops a thread in the middle of a turn. [Note to spec](note-to-spec.md) puts
`/new` and its task into the chat input when no agent session is free. Design:
[ADR 0027](../architecture/decisions/0027-zendo-slash-commands.md).

**What a thread sees.** A thread sees the common history as it was when the thread started, and
its own messages, in the order of time. The messages that another session moves into the common
history later do not show in the thread; the next new thread of the session shows them. A session
that has no thread yet sees the whole common history, until its first message starts a thread.

**The arrows.** `up` on the first row of the chat input shows the message before; `down` on the
last row shows the next message. After the newest message, `down` puts back the text and the chips
that the user wrote before the first `up`. While the suggestions of `@` or `/` are open, the arrows
choose a suggestion.

**What a message keeps.** The history keeps the text of the message and its text chips: a
selection, a paste, a message of the transcript. `up` shows a text chip as a chip again, and its
text goes with the question again. File and image chips do not go into the history (Design). A
message that is the same as the last message of the thread, with the same chips, goes in once
(Design).

## Detailed requirements

| ID         | Pattern    | Requirement | Source |
| ---------- | ---------- | ----------- | ------ |
| HISTORY-1  | Event      | When the user sends a message from the chat input, the system shall add its text, with the time, to the history of the active zendo session. | User, 2026-11-18 |
| HISTORY-2  | Event      | When the user starts a new thread in a zendo session with `ctrl-n` or the command `/clear`, the system shall move the history of that session into the common history, in the order of time, and start the session with an empty history of its own. | User, 2026-11-18 |
| HISTORY-3  | Ubiquitous | The system shall keep the history of each zendo session and the common history in `.zendo/db.json`, and keep only the last 1000 messages of the common history. | User, 2026-11-18 |
| HISTORY-4  | Ubiquitous | The system shall give each thread of a zendo session the common history as it was at the start of the thread, and its own history, in the order of time. | User, 2026-11-18 |
| HISTORY-5  | Event      | When the user presses `up` on the first row of the chat input, the system shall show the message before in the history of the session; when the user presses `down` on the last row, the next message. | User, 2026-11-18 |
| HISTORY-6  | Event      | When the user presses `down` on the newest message of the history, the system shall put back the text and the chips of the chat input from before the first `up`. | Design |
| HISTORY-7  | Ubiquitous | The system shall keep the text of a message with its text chips, show them as chips again when the message comes back, and shall not add a message that has no text or that is the same as the last message of the thread. | User, 2026-11-18 |
| HISTORY-8  | Event      | When the user sends `/new <text>`, the system shall start a new thread in the zendo session as `ctrl-n` does (HISTORY-2), then send `<text>` to the new thread as a question with the context of the chat input. | User, 2026-11-19 |
| HISTORY-9  | Event      | When the user sends `/new` with no text, the system shall start a new thread as `ctrl-n` does and send nothing to the agent. | User, 2026-11-19 |
| HISTORY-10 | State      | While the zendo session answers, the system shall queue `/new` as every question (CLAUDE-66), as the user typed it, and start the new thread when the turn ends; a question behind it in the queue goes to the new thread. | Design |
| HISTORY-11 | Unwanted   | If Claude Code runs in the zendo session, then the system shall not send `/new` to Claude Code, shall keep it in the chat input, and shall say that the session is open in Claude Code. | Design |

Tests: `up_and_down_go_through_the_messages_of_the_session` (HISTORY-1, HISTORY-5, HISTORY-6),
`ctrl_n_merges_the_session_into_the_common_history_by_time` (HISTORY-2, HISTORY-4),
`clear_merges_the_session_into_the_common_history` (HISTORY-2),
`up_brings_a_text_chip_back_as_a_chip` (HISTORY-5, HISTORY-7) and
`the_history_comes_back_after_a_restart` (HISTORY-3), `new_starts_a_thread_and_sends_its_text`
(HISTORY-8, HISTORY-9), `new_waits_for_the_turn_end`,
`a_question_behind_a_queued_new_is_the_first_of_the_new_thread` and
`a_queued_new_goes_back_into_the_input_as_new` (HISTORY-10) and `new_does_not_go_to_claude_code`
(HISTORY-11) in `tests/chat_history.rs`; the unit tests of
the history in `src/setup/history_tests.rs` (HISTORY-2, HISTORY-3, HISTORY-4, HISTORY-7) and of
the arrows and the chips in `src/app/composer_tests.rs` (HISTORY-5, HISTORY-6, HISTORY-7).
