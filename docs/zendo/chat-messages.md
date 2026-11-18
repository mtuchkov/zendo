---
status: "draft"
revision: "1"
date: "2026-11-17"
parent: "zendo.md"
prev: "chat-chips.md"
next: "terminal/terminal.md"
level: "how the chips of a question go to the agent and to Claude Code, and how the transcript shows them"
note: "Agent draft of the requests of the user on 2026-11-17, with the answers of the user to the open questions in the chat on 2026-11-17, and the escapes of CLAUDE-61 asked by the user on 2026-11-17. Not reviewed yet."
---

# Chips in a question

The chat input and its chips are in [Chat](chat.md); the chip window is in
[Chips and the open document](chat-chips.md). This document is a sibling of them, under
[zendo](zendo.md).

**What goes.** A question goes to the agent, or to the Claude Code session of the agent session
(SESSIONS-4), as the user wrote it, and each chip stays at its place. A chip that brings a text
goes as `⟦`, its text, and `⟧`. The text is what the chip window shows: the line that names its
source, such as `Text from this chat:` or `Selected text from docs/a.md (about lines 3–5):`, and
the quoted text. For example, `yes ⟦Text from this chat:` / `> Start now⟧`. A chip that brings only
a file goes as its label, with the file attached (CLAUDE-16, CLAUDE-18); to Claude Code it goes as
`@` and the path of the file, and Claude Code reads the file. The context that is not a chip, such
as the delta of a note, goes before the question as before.

**Escapes.** `⟦` and `⟧` mark the chips, so a text that has them escapes them. zendo escapes the
text that the user pastes into the chat input, as a chip or as text, and the text that the user
adds to the chat (`add_to_chat`: a selection of a note, a line, a message of the transcript): a
`⟦` becomes `\⟦`, a `⟧` becomes `\⟧`, and a run of `\` right before a mark or at the end of the
text doubles. To read a chip back, a run of n `\` before a mark reads as (n − 1) / 2 `\` and the
mark as text when n is odd, and as n / 2 `\` and a real mark when n is even (Design). An escaped mark is text: the chat input, the transcript and the agent see it as `\⟦`; the
chip window shows the text without the escapes. Text that the user types is not escaped. The text
of a pasted chip names its source as `Text that the user pasted:`, not by its label (Design).

**What the transcript shows.** The transcript shows the question as the user wrote it in the chat
input, with its chips as chips in their colors. A double-click on a chip in the transcript opens
the chip window, as in the chat input. A question of an earlier session (CLAUDE-27) comes back
from the agent with its chips in `⟦` and `⟧`; the transcript shows each one as a chip with a label
made from its text (CLAUDE-20, CLAUDE-41), and `↩` for text from the chat (Design). When zendo
does not know what a chip in the transcript brings, for example the file of a chip of an earlier
session, the chip window says so.

## Architecturally significant requirements

This document has no architecturally significant requirement of its own. CLAUDE-59 and CLAUDE-60
change the form of what goes to the agent, within CLAUDE-1 and CLAUDE-26 of [Claude](claude.md)
and SESSIONS-4 of [Sessions](sessions.md).

## Detailed requirements

| ID        | Pattern    | Requirement | Source |
| --------- | ---------- | ----------- | ------ |
| CLAUDE-59 | Ubiquitous | The system shall send each chip that brings a text at its place in the question, as `⟦`, the text of the chip with the line of its source, and `⟧`, to the agent and to Claude Code. | User, 2026-11-17 |
| CLAUDE-60 | Ubiquitous | The system shall send a chip that brings only a file at its place: to the agent as its label with the file attached, and to Claude Code as `@` and the path of the file. | User, 2026-11-17 |
| CLAUDE-61 | Event      | When the user pastes text into the chat input, or adds text to the chat, the system shall escape each `⟦` and `⟧` in it as `\⟦` and `\⟧`, and show and send an escaped mark as text, not as a chip. | User, 2026-11-17 |
| CLAUDE-62 | Ubiquitous | The system shall show a question in the transcript as the user wrote it in the chat input, with its chips as chips in their colors. | User, 2026-11-17 |
| CLAUDE-63 | Event      | When the user double-clicks a chip in the transcript, the system shall show the chip window of that chip (CLAUDE-48 to CLAUDE-53). | User, 2026-11-17 |
| CLAUDE-64 | Event      | When a question of an earlier session comes back (CLAUDE-27), the system shall show each `⟦…⟧` in it as a chip with a label made from its text, and `↩` for text from the chat. | Design |
| CLAUDE-65 | Unwanted   | If the system does not know what a chip in the transcript brings, then the chip window shall say so. | Design |

Tests that verify detailed requirements: `a_text_chip_goes_at_its_place_in_brackets` (mock agent;
CLAUDE-59, CLAUDE-61), `marks_in_a_pasted_or_added_text_are_escaped` (CLAUDE-61), `a_file_chip_goes_as_its_label_with_the_file` (mock agent; CLAUDE-60) in
`tests/chat_flows.rs`; `claude_code_gets_the_chips_at_their_places` (CLAUDE-59, CLAUDE-60) in
`tests/sessions.rs`; `escapes_go_and_come_back` (CLAUDE-61, CLAUDE-64) in `src/app/chips_tests.rs`; `the_transcript_shows_chips_and_opens_them` (CLAUDE-62, CLAUDE-63, CLAUDE-65)
and `a_replayed_question_shows_its_chips` (mock agent; CLAUDE-64) in `tests/chip_view.rs`.

## Open points

None. The user checked on 2026-11-17 that Claude Code reads the file of an `@path` that comes in a
paste of more than one line (CLAUDE-60).
