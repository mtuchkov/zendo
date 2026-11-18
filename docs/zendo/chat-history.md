---
status: "draft"
revision: "2"
date: "2026-11-18"
parent: "zendo.md"
prev: "chat-messages.md"
next: "terminal/terminal.md"
level: "the history of the messages of the user: per zendo session, the common history, and the arrows that go through it"
note: "Agent draft of the request of the user on 2026-11-18. Revision 2: text chips in the history, `/clear` as a new thread, and a thread that sees the common history of its start, asked by the user on 2026-11-18. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
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

Tests: `up_and_down_go_through_the_messages_of_the_session` (HISTORY-1, HISTORY-5, HISTORY-6),
`ctrl_n_merges_the_session_into_the_common_history_by_time` (HISTORY-2, HISTORY-4),
`clear_merges_the_session_into_the_common_history` (HISTORY-2),
`up_brings_a_text_chip_back_as_a_chip` (HISTORY-5, HISTORY-7) and
`the_history_comes_back_after_a_restart` (HISTORY-3) in `tests/chat_history.rs`; the unit tests of
the history in `src/setup/history_tests.rs` (HISTORY-2, HISTORY-3, HISTORY-4, HISTORY-7) and of
the arrows and the chips in `src/app/composer_tests.rs` (HISTORY-5, HISTORY-6, HISTORY-7).
