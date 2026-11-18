---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "chat-messages.md"
next: "terminal/terminal.md"
level: "the history of the messages of the user: per zendo session, the common history, and the arrows that go through it"
note: "Agent draft of the request of the user on 2026-11-18. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Chat history

The chat input is in [Chat](chat.md). A zendo session is in [zendo session](zendo-session.md).

**Two histories.** zendo keeps the messages that the user sends from the chat input. Each zendo
session has a history of its own: the messages of its current thread. The repository has one
common history. zendo keeps both in `.zendo/db.json`, so they stay after a restart. The common
history keeps the last 1000 messages.

**A new thread.** `ctrl-n` starts a new thread in the zendo session. Then zendo moves the history of
the session into the common history and puts all the messages in the order of the time when the
user sent them. The new thread starts with an empty history of its own.

**The arrows.** `up` on the first row of the chat input shows the message before; `down` on the
last row shows the next message. After the newest message, `down` puts back the text that the user
wrote before the first `up`. A zendo session goes through the common history and its own history,
in the order of time (Design: a session sees the common history as it is now, also the messages
that another session moved into it after the session started). While the suggestions of `@` or `/`
are open, the arrows choose a suggestion.

**What a message keeps.** The history keeps the text of the message without its chips (Design).
A message that is only chips does not go into the history. A message that is the same as the last
message of the session goes in once (Design).

## Detailed requirements

| ID         | Pattern    | Requirement | Source |
| ---------- | ---------- | ----------- | ------ |
| HISTORY-1  | Event      | When the user sends a message from the chat input, the system shall add its text, with the time, to the history of the active zendo session. | User, 2026-11-18 |
| HISTORY-2  | Event      | When the user starts a new thread with `ctrl-n` in a zendo session, the system shall move the history of that session into the common history, in the order of time, and start the session with an empty history of its own. | User, 2026-11-18 |
| HISTORY-3  | Ubiquitous | The system shall keep the history of each zendo session and the common history in `.zendo/db.json`, and keep only the last 1000 messages of the common history. | User, 2026-11-18 |
| HISTORY-4  | Ubiquitous | The system shall give each zendo session the common history and its own history, in the order of time. | User, 2026-11-18 |
| HISTORY-5  | Event      | When the user presses `up` on the first row of the chat input, the system shall show the message before in the history of the session; when the user presses `down` on the last row, the next message. | User, 2026-11-18 |
| HISTORY-6  | Event      | When the user presses `down` on the newest message of the history, the system shall put back the text of the chat input from before the first `up`. | Design |
| HISTORY-7  | Ubiquitous | The system shall keep the text of a message without its chips, and shall not add a message that has no text or that is the same as the last message of the session. | Design |

Tests: `up_and_down_go_through_the_messages_of_the_session` (HISTORY-1, HISTORY-5, HISTORY-6),
`ctrl_n_merges_the_session_into_the_common_history_by_time` (HISTORY-2, HISTORY-4) and
`the_history_comes_back_after_a_restart` (HISTORY-3) in `tests/chat_history.rs`; the unit tests of
the history in `src/setup/history_tests.rs` (HISTORY-2, HISTORY-3, HISTORY-4, HISTORY-7) and of
the arrows in `src/app/composer_tests.rs` (HISTORY-5, HISTORY-6).

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Shall a session see the messages that another session moved into the common history after the session started, or only the common history as it was at the start of its thread? | HISTORY-4 | User |
| Shall the history keep the chips of a message, with their text and files, so that `up` brings them back? | HISTORY-7 | User |
