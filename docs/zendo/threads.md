---
status: "approved"
approved: "2026-11-17"
revision: "3"
date: "2026-11-17"
parent: "zendo.md"
prev: "questions.md"
next: "sessions.md"
level: "the threads of the chat: the five agent sessions, how the user chooses one, and how zendo keeps and resumes them"
note: "Split from claude.md, revision 4, by topic. Agent rewrite of the reference of zendo 0.9.1. Revision 2: the chat input of each agent session (CLAUDE-43 to CLAUDE-47, ADR 0011), asked by the user on 2026-11-17. The user approved the document on 2026-11-17. Revision 3: the sessions count from 0, session 0 in the main checkout (the user, 2026-11-18)."
---

# Threads

The connection to Claude, what zendo sends and what Claude may do are in [Claude](claude.md). This
document is a sibling of it, under [zendo](zendo.md).

**Threads.** zendo keeps five agent sessions for the repository, each one thread for all files.
`F6` to `F10` (or `alt` and the F key, where the terminal takes the F key) choose session 0 to 4, and
`alt-]` goes to the next one, after the last through the terminals of [Terminal](terminal/terminal.md),
then to the first; `alt-[` (`alt-}` on macOS) goes back. Buttons in one row above the header of the
chat, or on its right edge, show the sessions, each with its key and its name
([Session names](session-names/session-names.md)): the active one in the accent color, the others in the second
accent, as the badges of the modes; a click chooses one. While a terminal shows in the chat pane, all
of them are in the second accent, and the terminal button is in the accent color (TERM-15).

**The input of a session.** Each agent session has its own chat input, also after a restart.
The chip window and the choice of the open document ([Chips](chat-chips.md)) belong to it, but do
not stay after a restart. Design: [ADR 0011](../architecture/decisions/0011-chat-input-per-agent-session.md).

**Session IDs and resume.** Each thread is a Claude Code session, so the user can continue it in
Claude Code. zendo keeps the session ID in its state in the repository from the first question of
the session: a session without a question has nothing to resume, so the next start opens a new
session without a warning. When the chat opens the first time, zendo opens the session: the chat
shows the messages of the last session, and the agent sends its slash commands.

## Detailed requirements

| ID        | Pattern    | Requirement                                                                                                                  | Source           |
| --------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| CLAUDE-5  | Ubiquitous | The system shall keep five agent sessions for the repository, each one thread for all files, and keep their session IDs and the active session in the repository. | User, 2026-11-12 |
| CLAUDE-6  | Event      | When the user asks in a thread that has a session ID, the system shall continue that session and show its earlier messages. | Design           |
| CLAUDE-40 | Event      | When the first question goes to a session, the system shall keep its session ID; the system shall not keep the ID of a session that got no question. | User, 2026-11-14 |
| CLAUDE-27 | Event      | When the chat opens the first time, the system shall open the session of the thread and show the messages of the last session in their order, each question without the context that zendo sent with it. | User, 2026-11-12 |
| CLAUDE-31 | Event      | When the user presses `F6` to `F10`, or `alt` and one of them, the system shall open the chat on that agent session; when the user presses `alt-]`, or `alt-[` (`alt-}` on macOS), on the next or the previous session, after the last the terminals (TERM-41, TERM-49). | User, 2026-11-12; User, 2026-11-13 |
| CLAUDE-32 | Ubiquitous | The system shall show a button for each agent session in one row above the header of the chat, or on its right edge (NAMES-8), named by its F key and its name (NAMES-6): the active one in the accent color, the others in the second accent, and all in the second accent while a terminal shows in the chat pane (TERM-15); a click chooses the session. | User, 2026-11-12; User, 2026-11-14; User, 2026-11-17 |
| CLAUDE-35 | Unwanted   | If the keymap in effect gives the agent sessions other keys than `F6` to `F10`, then the system shall name the agent sessions `0` to `4` on their buttons and in their names. | User, 2026-11-13; User, 2026-11-18 (from 0) |
| CLAUDE-43 | Ubiquitous | The system shall keep a chat input of its own for each agent session: its text, its chips, its files and images, and its context. | User, 2026-11-17 |
| CLAUDE-44 | Event      | When the user chooses another agent session, the system shall show the chat input of that session as the user left it, and keep the input of the session before unchanged. | User, 2026-11-17 |
| CLAUDE-45 | Ubiquitous | The system shall keep the chat input of each agent session in the repository, and show it again in its session after a restart of zendo. | User, 2026-11-17 |
| CLAUDE-46 | Unwanted   | If the file or the image of a chip is gone, then the system shall show the chip in red, also after a restart. | User, 2026-11-17 |
| CLAUDE-47 | Event      | When the name of a pasted image comes (CLAUDE-38), or the queued questions of a thread go back into the input (CLAUDE-34), the system shall change the chat input of the session of that chip or thread, also while another session is active. | Design           |
| CLAUDE-10 | Optional   | Where the agent has a command line with a resume, the system shall continue the thread there when the user presses `ctrl-o`. | Design           |

This document has no architecturally significant requirement of its own: CLAUDE-1 of
[Claude](claude.md) and ASK-1 and ASK-2 of [Questions of the agent](questions.md) cover the
connection that the threads use.

Tests: `one_thread_for_every_file_and_the_file_goes_once` (CLAUDE-5),
`the_chat_shows_the_last_session_when_it_opens` (CLAUDE-27, CLAUDE-6, CLAUDE-40),
`a_session_id_is_kept_only_after_the_first_question` (CLAUDE-40),
`five_agent_sessions_with_f6_to_f10_and_alt_n` (CLAUDE-5, CLAUDE-31, CLAUDE-32) and
`alt_and_an_f_key_choose_a_session_too` (CLAUDE-31) in `tests/chat_flows.rs`;
`shell_is_blocked_and_resume_replays` (CLAUDE-6) in `tests/acp_mock.rs`;
`other_session_keys_name_the_buttons_0_to_4_and_the_terminal_by_its_mark` (CLAUDE-35, TERM-48) in
`tests/keymap_hints.rs`; `each_session_keeps_its_own_chat_input` (CLAUDE-43, CLAUDE-44) and
`the_input_of_a_session_comes_back_after_a_restart` (CLAUDE-45, CLAUDE-46) in
`tests/session_input.rs`; `drafts_go_to_the_state_and_come_back` (CLAUDE-45) and
`background_changes_go_to_the_input_of_their_session` (CLAUDE-47) in `src/app/chat_pane_tests.rs`.
