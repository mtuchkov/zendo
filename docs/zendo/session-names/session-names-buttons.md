---
status: "implemented"
approved: "2026-11-17"
revision: "2"
date: "2026-11-17"
parent: "session-names.md"
prev: "session-names-api.md"
level: "where the name of an agent session shows: the row of buttons, the right edge, the bottom and the top line"
note: "From the requests of the user on 2026-11-14, split from session-names.md in its revision 2 without a change of the requirements. Approved by the user on 2026-11-14. Revision 2 (the user, 2026-11-17): NAMES-11 is retired; only the width of the chat puts the buttons on the right edge (NAMES-8), also while a terminal shows."
---

# Session names: buttons

The name of an agent session ([Session names](session-names.md)) shows after its key.

**Where a name shows.** The buttons of the agent sessions stand in one row above the header of the
chat, each with its key and its name: `F6 parser·  F7  F8 hooks•`. In a narrow chat the names get
shorter. When the row does not fit even then, the buttons stand on the right edge, one under the
other, with their keys alone, above the terminal buttons. The name also shows in the bottom line of the session view and
in the top line (ATTN-10). The terminals (`F5` and `[>]`) get no name, and their buttons stay on
the right edge (TERM-14).

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| NAMES-6  | Ubiquitous | The system shall show the name of an agent session after its key: on its button, in the bottom line of the session view, and in the top line. | User, 2026-11-14 |
| NAMES-8  | Unwanted   | If the buttons of the agent sessions with their names do not fit into one row, then the system shall shorten the names, and then show the buttons on the right edge of the chat, one under the other, with their keys alone. | User, 2026-11-14 |

Tests in `tests/session_names.rs`:
`narrow_chats_shorten_the_names_and_then_put_the_buttons_on_the_side` and
`a_terminal_in_the_chat_pane_keeps_the_buttons_in_the_row` (NAMES-8),
`the_first_agent_message_names_the_session` and `the_bottom_line_shows_the_name` (NAMES-6).
