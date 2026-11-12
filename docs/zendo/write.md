---
status: "draft"
revision: "1"
date: "2026-11-12"
parent: "zendo.md"
prev: "notes.md"
next: "say.md"
level: "write mode: the live view, hard wrap, the cursor between modes, edit keys"
note: "Agent draft, split from notes.md. Not reviewed by the user."
---

# Write mode

Write mode looks as read mode does: the same colors, headings, lists, quotes, code and tables. Only
the source line with the cursor shows as it is in the file, so the user edits the real text; in a
table, only the row with the cursor shows its source, and the rest of the table keeps its borders.
Write mode joins and wraps a paragraph as read mode does; only the cursor line stands alone. The
head of the note (the title, the date and the tags, the recap) stays where read mode shows it, so
the text does not move when the mode changes.

zendo breaks a line at a word when it gets longer than the text column (76 chars by default; `width`
in the config, and `alt-=` and `alt--` for a repository). A list item goes on under
its marker and a quote with `>`, so read mode joins the lines again. Headings, table rows, code and
front matter keep their length.

The cursor stays on the same char when the mode changes, at the same height on the screen: zendo
matches the chars of the formatted text with the chars of the source and skips the markup.

`ctrl-shift-h j k l`, `ctrl-shift` and the arrows, and Shift and the arrows select (`alt-shift` and
an arrow: a word); `ctrl-shift-s` says it here, `ctrl-shift-a` adds the
selection or the line to the chat, `ctrl-shift-c` or `ctrl-g` (in any terminal) saves and opens the chat; `ctrl-c`, `ctrl-x` and `ctrl-v` copy, cut and
paste with the system clipboard; `ctrl-b`, `ctrl-i` and `ctrl-u` make the selection or the word
bold, italic or underlined. When the user leaves write mode, and the file has no `# ` title, or it
is a doc outside the notes folder without front matter, the agent writes the missing part in a
background session without tools; zendo adds only that part and changes nothing else. The note pane
and read mode are in [Notes](notes.md).

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| NOTE-19 | Event      | When the user presses `ctrl-c`, `ctrl-x` or `ctrl-v` in write mode, the system shall copy, cut or paste with the system clipboard. | User, 2026-11-11 |
| NOTE-20 | Event      | When the user presses `ctrl-b`, `ctrl-i` or `ctrl-u` in write mode, the system shall mark the selection, or the word at the cursor, bold, italic or underlined, and take the mark away when it is there. | User, 2026-11-11 |
| NOTE-22 | Event      | When the user leaves write mode in a file that has no `# ` title, the system shall ask the agent for a title and add it as the first line. | User, 2026-11-12 |
| NOTE-23 | Event      | When the user leaves write mode in a doc outside the notes folder that has no front matter, the system shall ask the agent for front matter and add it at the top. | User, 2026-11-12 |
| NOTE-28 | Event      | When the user switches between read mode and write mode, the system shall keep the cursor on the same char of the text, at the same height on the screen. | User, 2026-11-12 |
| NOTE-29 | Event      | When the user types a line that gets longer than the text column, the system shall break it at a word, under the list marker or with the `>` of a quote; headings, table rows, code and front matter keep their length. | User, 2026-11-12 |
| NOTE-30 | State      | While write mode is on, the system shall show the note as read mode does, and show only the source line with the cursor as source; in a table, only that row. | User, 2026-11-12 |
| NOTE-31 | State      | While write mode is on, the system shall wrap the text and show the head of the note (title, date, recap) as read mode does; only the cursor line differs. | User, 2026-11-12 |

NOTE-9 (focus dimming) is retired: write mode now has the colors of read mode.

Tests that verify detailed requirements: `select_copy_cut_and_type_over` (NOTE-19),
`bold_italic_underline_toggle` (NOTE-20), `title_and_front_matter_go_in_only_when_missing` (NOTE-22,
NOTE-23), `the_same_char_in_both_views`, `hidden_lines_do_not_shift_the_cursor`,
`a_row_with_a_trailing_space`, `a_link_url_does_not_lose_the_way`,
`markup_and_decoration_are_skipped` (NOTE-28), `a_long_line_breaks_at_a_word`,
`list_items_and_quotes_go_on_under_their_marker`, `headings_tables_and_code_keep_their_length`,
`a_line_does_not_start_with_a_block_marker`, `typing_past_the_column_breaks_the_line` (NOTE-29),
`write_mode_keeps_the_char_and_shows_only_the_cursor_line_as_source` (NOTE-28, NOTE-30),
`write_mode_wraps_as_read_mode_and_keeps_the_head` (NOTE-31).
