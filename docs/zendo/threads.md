---
status: "draft"
revision: "1"
date: "2026-11-14"
parent: "zendo.md"
prev: "claude.md"
next: "sessions.md"
level: "the threads of the chat: the five agent sessions, how the user chooses one, and how zendo keeps and resumes them"
note: "Split from claude.md, revision 4, by topic. Agent rewrite of the reference of zendo 0.9.1. Not reviewed by the user."
---

# Threads

The connection to Claude, what zendo sends and what Claude may do are in [Claude](claude.md). This
document is a sibling of it, under [zendo](zendo.md).

**Threads.** zendo keeps five agent sessions for the repository, each one thread for all files.
`F6` to `F10` (or `alt` and the F key, where the terminal takes the F key) choose a session, and
`alt-]` goes to the next one, after the last through the terminals of [Terminal](terminal/terminal.md),
then to the first; `alt-[` (`alt-}` on macOS) goes back. Buttons in one row above the header of the
chat, or on its right edge, show the sessions, each with its key and its name
([Session names](session-names/session-names.md)): the active one in the accent color, the others in the second
accent, as the badges of the modes; a click chooses one.

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
| CLAUDE-32 | Ubiquitous | The system shall show a button for each agent session in one row above the header of the chat, or on its right edge (NAMES-8), named by its F key and its name (NAMES-6): the active one in the accent color, the others in the second accent; a click chooses the session. | User, 2026-11-12; User, 2026-11-14 |
| CLAUDE-35 | Unwanted   | If the keymap in effect gives the agent sessions other keys than `F6` to `F10`, then the system shall name the agent sessions `1` to `5` on their buttons and in their names. | User, 2026-11-13 |
| CLAUDE-10 | Optional   | Where the agent has a command line with a resume, the system shall continue the thread there when the user presses `ctrl-o`. | Design           |

This document has no architecturally significant requirement of its own: CLAUDE-1 and CLAUDE-3 of
[Claude](claude.md) cover the connection that the threads use.

Tests: `one_thread_for_every_file_and_the_file_goes_once` (CLAUDE-5),
`the_chat_shows_the_last_session_when_it_opens` (CLAUDE-27, CLAUDE-6, CLAUDE-40),
`a_session_id_is_kept_only_after_the_first_question` (CLAUDE-40),
`five_agent_sessions_with_f6_to_f10_and_alt_n` (CLAUDE-5, CLAUDE-31, CLAUDE-32) and
`alt_and_an_f_key_choose_a_session_too` (CLAUDE-31) in `tests/chat_flows.rs`;
`shell_is_blocked_and_resume_replays` (CLAUDE-6) in `tests/acp_mock.rs`;
`other_session_keys_name_the_buttons_1_to_5_and_the_terminal_by_its_mark` (CLAUDE-35, TERM-48) in
`tests/keymap_hints.rs`.
