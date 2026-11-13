---
status: "draft"
revision: "1"
date: "2026-11-11"
parent: "zendo.md"
prev: "panes.md"
next: "write.md"
level: "the note pane: read mode and write mode"
note: "Agent rewrite of the reference of zendo 0.9.1. Not reviewed by the user."
---

# Notes

The note pane shows one markdown file: a note of the notes folder or any file of the file tree. It
has two modes. In read mode, zendo shows the file formatted for the width of the pane, with a cursor
line. In write mode, the user edits the text, and zendo saves it after a pause of 1.5 s; there is no save key.

A note is a markdown file in the notes folder, `docs/notes` by default. The notes list and search
are in [Notes list and search](notes-list.md).

In read mode, a blinking block cursor stands on one char of a bright cursor line. `h j k l` move it,
`ctrl-shift-h j k l` or `ctrl-shift` and the arrows (or `H J K L`, or Shift and the arrows) select;
`alt-shift-h` and `alt-shift-l` (or `alt-shift` and an arrow) select a word, also in write mode and
the chat. Every key comes from the keymap
(see [Panes](panes.md)); `alt-?` (or `f1`) shows the keys of the pane. Write mode is in
[Write mode](write.md).

`ctrl-shift-s` or `alt-s` asks the agent to write an idea into the note: see [Say it here](say.md).

Tables show with solid borders and fit the pane. The body rows of a table alternate two backgrounds
of the same tone: the page color and a color a small step from it. Code blocks and front matter show with syntax
colors: `sh`, `rust`, `java`, `python`, `go`, `yaml` and `json` are on; other languages are plugins.

## Architecturally significant requirements

| ID     | Pattern    | Requirement                                                                                                                                                    | Source           |
| ------ | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| NOTE-1 | Ubiquitous | The system shall join the lines of a paragraph, a list item or a quote that the file wraps by hand, and wrap them again to the width of the pane.              | User, 2026-11-11 |
| NOTE-2 | Ubiquitous | The system shall show headings and hard line breaks as the file has them, tables with solid borders that fit the pane, and code and front matter with syntax colors. | User, 2026-11-11 |
| NOTE-3 | Ubiquitous | The system shall save the text of the user without a save command.                                                                                             | Design           |

### Acceptance tests

**`hard_wrapped_paragraphs_and_items_are_rewrapped`** (unit test; verifies NOTE-1)

- Given a paragraph and a list item that the file wraps at 40 columns.
- When zendo shows them in a pane of 80 columns.
- Then each shows as lines of up to 80 columns, with the words in the same order.

**`front_matter_and_code_stay_as_they_are`** (unit test; verifies NOTE-2)

- Given a file with front matter and a code block.
- When zendo shows it.
- Then the front matter and the code show line by line, as in the file.

**`writing_saves_after_a_pause`** (tests/write_keys.rs; verifies NOTE-3)

- Given a note in write mode.
- When the user types a line and stops typing.
- Then the file on disk has the line, before the user presses `esc`.

## Detailed requirements

| ID      | Pattern    | Requirement                                                                                                                         | Source           |
| ------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| NOTE-4  | State      | While the note pane has focus in read mode, the system shall show a cursor line.                                                    | User, 2026-11-11 |
| NOTE-5  | Event      | When the user presses `a` in read mode, the system shall send the selection, or the paragraph under the cursor, to Claude with its line numbers. | User, 2026-11-11 |
| NOTE-6  | Event      | When the user presses `y` in read mode, the system shall copy the selection to the clipboard.                                       | Design           |
| NOTE-16 | State      | While the note pane has focus in read mode, the system shall show a blinking block cursor on one char of a bright cursor line.     | User, 2026-11-11 |
| NOTE-17 | Event      | When the user moves the cursor with Shift (`ctrl-shift-h j k l`, `H J K L`, Shift and the arrows), the system shall select the text char by char.        | User, 2026-11-11 |
| NOTE-18 | Event      | When the user adds the selection to the chat (`ctrl-shift-a` or `alt-a`, or `+` in read mode), the system shall put a chip with the first 15 chars of the selection, and `+N` for N more lines, into the chat input and send the text as context. | User, 2026-11-11 |
| NOTE-21 | Optional   | Where a `.sublime-syntax` file is in `~/.zendo/syntaxes/`, or a language is in `[highlight] languages`, the system shall highlight that language. | User, 2026-11-11 |
| NOTE-34 | Ubiquitous | The system shall show the body rows of a table with alternate backgrounds of the same tone: the page color, and the page color mixed with 5 % of the text color. | User, 2026-11-12 |

Tests that verify detailed requirements: `bold_italic_underline_and_tables_render` (NOTE-2),
`the_default_languages_are_on` (NOTE-2), `a_language_is_on_by_config_or_by_plugin_file` (NOTE-21),
`table_rows_alternate_backgrounds` (NOTE-34).
