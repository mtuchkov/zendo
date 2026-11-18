---
status: "approved"
approved: "2026-11-17"
revision: "1"
date: "2026-11-17"
parent: "zendo.md"
prev: "chat.md"
next: "chat-messages.md"
level: "what the user sees of a chip and of the open document before a question goes: the chip window, images, the choice of the open document"
note: "Agent draft of the requests of the user on 2026-11-17, with the answers of the user to the open questions in the chat on 2026-11-17. Approved by the user on 2026-11-17."
---

# Chips and the open document

The chat input and its chips are in [Chat](chat.md); each agent session has its own input
([Threads](threads.md), CLAUDE-43). This document is a sibling of them, under [zendo](zendo.md).

**The chip window.** A double-click on a text chip, or on the chip of a text file, opens a window
with what the chip brings: the context of a text chip, with the quoted text in the normal color
and the line that names its source (`Selected text from …`) in the dim color, or the text of the
file. The window has the color of the chip, dimmed, as its background. Its top line has the label
of the chip on the left and `[ esc close ]` on the right. Its bottom right corner is at the top
right corner of the chat input. It is as wide as its longest line, at most as wide as the text
column of the note pane, and grows to the left; it grows up with its text, up to the top of the
transcript. The arrows, `pageup`, `pagedown`, `home`, `end` and the wheel scroll the text; `esc`
closes the window; other keys and clicks do nothing. One click on a chip changes nothing (Design).

**Images and other files.** A double-click on an image chip, or on the chip of a file that is not
text, opens the file in the program of the system for its kind: `open` on macOS, `xdg-open` on
Linux. When the file of a chip is gone, or the program does not start, the chip window shows the
error. A chip whose file is gone is red (CLAUDE-46); a question with it goes without the file.

**The open document.** While a document is open, the line above the chat input shows
`[x] open document`: the question takes the document as CLAUDE-26 says, with its path and the
current line, or the text of a new note that is not saved yet. A click on the line unselects it:
`[ ] open document`, and the question goes without anything of the document. Another click selects
it again. The choice stays until the text of the document changes or the user opens another
document in the session. The line does not show for a question across all notes or a slash
command. There is no key for the choice. The header of the chat names only the agent, and
`ALL NOTES` for a question across all notes.

## Architecturally significant requirements

This document has no architecturally significant requirement of its own. CLAUDE-55 changes what
goes to the agent, within CLAUDE-26 of [Claude](claude.md).

## Detailed requirements

| ID        | Pattern    | Requirement | Source |
| --------- | ---------- | ----------- | ------ |
| CLAUDE-48 | Event      | When the user double-clicks a text chip, or the chip of a text file, in the chat input, the system shall show a window with the text of the chip: the context of a text chip, or the text of the file. | User, 2026-11-17 |
| CLAUDE-49 | Ubiquitous | The system shall show the chip window with the color of the chip, dimmed, as its background, the label of the chip on its top line, its bottom right corner at the top right corner of the chat input, and the width of its longest line, at most the width of the text column of the note pane. | User, 2026-11-17 |
| CLAUDE-50 | State      | While the chip window is open, the system shall scroll its text with the arrows, `pageup`, `pagedown`, `home`, `end` and the wheel, and ignore the other keys and clicks. | User, 2026-11-17 |
| CLAUDE-51 | Event      | When the user presses `esc` while the chip window is open, the system shall close the window and change nothing else. | User, 2026-11-17 |
| CLAUDE-52 | Event      | When the user double-clicks an image chip, or the chip of a file that is not text, in the chat input, the system shall open the file in the program of the system for its kind. | User, 2026-11-17 |
| CLAUDE-53 | Unwanted   | If the file of a chip is gone, or the program of the system does not start, then the system shall show the error in the chip window. | User, 2026-11-17 |
| CLAUDE-54 | State      | While a document is open and the question is not across all notes or a slash command, the system shall show `[x] open document` on the line above the chat input, or `[ ] open document` when the user unselected it. | User, 2026-11-17 |
| CLAUDE-55 | Event      | When the user clicks the line of the open document, the system shall switch between `[x]` and `[ ]`; while it shows `[ ]`, the system shall send the question without the file, its project rules, its path, the current line, and the text of a new note. | User, 2026-11-17 |
| CLAUDE-56 | Event      | When the text of the open document changes, or the user opens another document in the session, the system shall select the open document again. | User, 2026-11-17 |
| CLAUDE-57 | Event      | When the user sends a question with a red chip (CLAUDE-46), the system shall send the question and its label without the gone file. | User, 2026-11-17 |
| CLAUDE-58 | Ubiquitous | The system shall name in the header of the chat the agent, and `ALL NOTES` for a question across all notes, and no open document. | User, 2026-11-17 |

Tests that verify detailed requirements: `a_double_click_on_a_chip_shows_its_text_over_the_input`
(CLAUDE-48 to CLAUDE-51), `a_file_chip_shows_the_text_of_its_file` (CLAUDE-48),
`a_double_click_on_an_image_chip_opens_it_outside` and `a_gone_file_shows_its_error_in_the_chip_window` (CLAUDE-52, CLAUDE-53) and
`the_open_document_can_stay_out_of_the_question` (mock agent; CLAUDE-54 to CLAUDE-56), `a_red_chip_goes_without_its_file` (mock agent; CLAUDE-57), `the_chat_header_does_not_name_the_open_document` (CLAUDE-58) in
`tests/chip_view.rs`; `two_clicks_on_one_chip_are_a_double_click` (CLAUDE-48) in
`src/app/chip_view_tests.rs`.

## Open points

None: the user answered them in the chat on 2026-11-17.
