---
status: "draft"
revision: "1"
date: "2026-11-17"
parent: "zendo.md"
prev: "chat.md"
next: "terminal/terminal.md"
level: "what the user sees of a chip and of the open document before a question goes: the chip window, images, the choice of the open document"
note: "Agent draft of the requests of the user on 2026-11-17, with the answers of the user to the open questions in the chat on 2026-11-17. Not reviewed yet."
---

# Chips and the open document

The chat input and its chips are in [Chat](chat.md); each agent session has its own input
([Threads](threads.md), CLAUDE-43). This document is a sibling of them, under [zendo](zendo.md).

**The chip window.** A double-click on a text chip or a file chip opens a window with what the
chip brings. For a text chip, the window shows the text that goes as context: the quoted text in
the normal color, and the line that names its source (`Selected text from …`) in the dim color.
For a file chip, it shows the text of the file; a file that is not text shows its size and the
words `not a text file`. The window has the color of the chip, dimmed, as its background, so the
user sees which chip it shows. Its top line has the label of the chip on the left and `[ esc close ]`
on the right. Its bottom right corner is at the top right corner of the chat input, it is as wide
as the input, and it grows up with its text, up to the top of the transcript. The arrows, `pageup`,
`pagedown`, `home`, `end` and the wheel scroll the text; `esc` closes the window. Other keys and
clicks do nothing while the window is open. One click on a chip changes nothing (Design).

**Images.** A double-click on an image chip opens the image in the image program of the system:
`open` on macOS, `xdg-open` on Linux. A terminal cannot show the image itself.

**The open document.** When the next question takes the open document (CLAUDE-26), the line above
the chat input shows `[x] open document`. A click on the line unselects it: `[ ] open document`.
Then the question goes without the file, its project rules, its path and the current line, as if no
document were open. Another click selects it again. The choice stays until the text of the document
changes or the user opens another document in the session; then the line shows `[x]` again. The
line does not show when the question would not take the document anyway: a question across all
notes, a new note, a slash command, or a document that went with one of the last 20 questions.
There is no key for the choice (User, 2026-11-17).

## Architecturally significant requirements

This document has no architecturally significant requirement of its own. CLAUDE-55 changes what
goes to the agent, within CLAUDE-26 of [Claude](claude.md).

## Detailed requirements

| ID        | Pattern    | Requirement | Source |
| --------- | ---------- | ----------- | ------ |
| CLAUDE-48 | Event      | When the user double-clicks a text chip or a file chip in the chat input, the system shall show a window with the text of the chip: the context of a text chip, or the text of the file of a file chip. | User, 2026-11-17 |
| CLAUDE-49 | Ubiquitous | The system shall show the chip window with the color of the chip, dimmed, as its background, the label of the chip on its top line, and its bottom right corner at the top right corner of the chat input. | User, 2026-11-17 |
| CLAUDE-50 | State      | While the chip window is open, the system shall scroll its text with the arrows, `pageup`, `pagedown`, `home`, `end` and the wheel, and ignore the other keys and clicks. | User, 2026-11-17 |
| CLAUDE-51 | Event      | When the user presses `esc` while the chip window is open, the system shall close the window and change nothing else. | User, 2026-11-17 |
| CLAUDE-52 | Event      | When the user double-clicks an image chip in the chat input, the system shall open the image in the image program of the system. | User, 2026-11-17 |
| CLAUDE-53 | Unwanted   | If the file of a chip is gone, or the image program does not start, then the system shall say so in the flash line and open no window. | Design |
| CLAUDE-54 | State      | While the next question would take the open document (CLAUDE-26), the system shall show `[x] open document` on the line above the chat input, or `[ ] open document` when the user unselected it. | User, 2026-11-17 |
| CLAUDE-55 | Event      | When the user clicks the line of the open document, the system shall switch between `[x]` and `[ ]`; while it shows `[ ]`, the system shall send the question without the file, its project rules, its path and the current line. | User, 2026-11-17 |
| CLAUDE-56 | Event      | When the text of the open document changes, or the user opens another document in the session, the system shall select the open document again. | User, 2026-11-17 |

Tests that verify detailed requirements: `a_double_click_on_a_chip_shows_its_text_over_the_input`
(CLAUDE-48 to CLAUDE-51), `a_file_chip_shows_the_text_of_its_file` (CLAUDE-48),
`a_double_click_on_an_image_chip_opens_it_outside` (CLAUDE-52, CLAUDE-53) and
`the_open_document_can_stay_out_of_the_question` (mock agent; CLAUDE-54 to CLAUDE-56) in
`tests/chip_view.rs`; `two_clicks_on_one_chip_are_a_double_click` (CLAUDE-48) in
`src/app/chip_view_tests.rs`.

## Open points

None. The user answered the open questions in the chat on 2026-11-17: the image opens outside,
the choice has no key, the line is above the input, an unselected document sends no path, and
other keys do nothing while the chip window is open.
