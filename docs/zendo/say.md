---
status: "draft"
revision: "1"
date: "2026-11-12"
parent: "zendo.md"
prev: "write.md"
next: "notes-list.md"
level: "say it here: the agent turns an idea into text at a place in the note"
note: "Agent draft. Not reviewed by the user."
---

# Say it here

`ctrl-shift-s` or `alt-s` means "here is what I want to say", in read mode and in
write mode: an arrow shows in the empty space left of the line, and the
chat input starts with a chip of the place, `write here :386`, with the number of the line. The
user writes the idea after the chip, and the agent gets the line and the column of the cursor, the
paragraph above and the paragraph below; it proposes connected text for that place. With a
selection, the chip is `rewrite this :386`, and the agent writes the selection again, with the
paragraphs around it. A chip sent alone asks the agent to write what fits there, or to write the
selection again clearer and shorter. The arrow goes away when the answer ends, or with
`esc` in the chat or in the note.

The agent gets the place (the file and the line), the quoted paragraphs, and the task: connect the
idea with the text around it, keep the style and the level of detail of the document, and make
one edit that changes nothing else. zendo allows the edit at once, as
every edit (see [Claude](claude.md)). The note pane is in [Notes](notes.md).

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| NOTE-25 | Event      | When the user presses `ctrl-shift-s` or `alt-s` in read mode or write mode, the system shall show an arrow left of the line of the cursor, start the chat input with the chip `write here :<line>`, and send the line and the column of the cursor, the paragraph above and the paragraph below with the question. | User, 2026-11-12 |
| NOTE-26 | Event      | When the user presses `ctrl-shift-s` or `alt-s` with a selection, the system shall start the chat input with the chip `rewrite this :<line>`, and send the selection and the paragraphs around it. | User, 2026-11-12 |
| NOTE-33 | Event      | When the user sends the chip of say here without more text, the system shall send a default question: write what fits there, or write the selection again clearer and shorter. | User, 2026-11-12 |
| NOTE-27 | Event      | When the answer ends, or the user presses `esc` in the chat or in the note, the system shall remove the arrow. | User, 2026-11-12 |

Tests that verify detailed requirements: `paragraphs_around_a_line` (NOTE-25),
`the_context_quotes_the_text_around_the_mark` (NOTE-25, NOTE-26), `say_marks_the_place_until_the_answer_ends`, `say_here_is_a_chip_with_the_line_and_sends_a_default_question` (NOTE-25, NOTE-33)
(NOTE-25, NOTE-27).
