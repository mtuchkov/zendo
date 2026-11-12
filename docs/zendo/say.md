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
chat starts with "Here is what I want to say here: ". The user writes the idea, and the agent gets
the paragraph above and the paragraph below; it proposes connected text for that place. With a
selection, the chat starts with "I want to say differently here: ", and the agent writes the
selection again, with the paragraphs around it. The arrow goes away when the answer ends, or with
`esc` in the chat or in the note.

The agent gets the place (the file and the line), the quoted paragraphs, and the task: connect the
idea with the text around it, keep the style and the level of detail of the document, and propose
one edit that changes nothing else. The edit comes back as a proposal, as every edit does (see
[Claude](claude.md)). The note pane is in [Notes](notes.md).

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| NOTE-25 | Event      | When the user presses `ctrl-shift-s` or `alt-s` in read mode or write mode, the system shall show an arrow left of the line of the cursor, start the chat input with "Here is what I want to say here: ", and send the line and the column of the cursor, the paragraph above and the paragraph below with the question. | User, 2026-11-12 |
| NOTE-26 | Event      | When the user presses `ctrl-shift-s` or `alt-s` with a selection, the system shall start the chat input with "I want to say differently here: ", and send the selection and the paragraphs around it. | User, 2026-11-12 |
| NOTE-27 | Event      | When the answer ends, or the user presses `esc` in the chat or in the note, the system shall remove the arrow. | User, 2026-11-12 |

Tests that verify detailed requirements: `paragraphs_around_a_line` (NOTE-25),
`the_context_quotes_the_text_around_the_mark` (NOTE-25, NOTE-26), `say_marks_the_place_until_the_answer_ends`
(NOTE-25, NOTE-27).
