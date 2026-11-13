---
status: "draft"
revision: "1"
date: "2026-11-12"
parent: "zendo.md"
prev: "sessions.md"
next: "terminal/terminal.md"
level: "the chat input: lines, chips, attachments and @ file suggestions"
note: "Agent draft, split from claude.md. Not reviewed by the user."
---

# Chat

The chat input is where the user writes to the agent. The connection, threads and edits are in
[Claude](claude.md).

**The chat input** is an editor: the arrows and the word keys move in it, every word can change.
`alt-enter` (or `shift-enter`, `ctrl-j`) adds a line; the status bar names `alt-enter`, because
a terminal without the kitty keyboard protocol sends `shift-enter` as `enter`. A `\` at the end of the line and `enter`
also add a line, in every terminal. The input grows up to a quarter of the panel, then scrolls.

**Chips.** A selection added to the chat is a chip with its first 15 chars, and `+N` when it has N more lines; it brings the selected
text as context. A file chip shows the last 4 chars of its folder and the first 10 chars of its
name, as `ates/specificat`. The arrows step over a chip as one unit, and the chip at the cursor
shows in the accent color; `backspace` and `delete` remove it. `@` and the start of a name show the
4 best files under the input; the arrows choose, `tab` or `enter` adds the file as a chip. `ctrl-t`
does the same for templates: the chip tells the agent to write a new document from the template, for
example a specification from the notes the user names with `@`.

**The transcript.** `tab` in the chat input moves the keys to the transcript, and `tab` there to
the next pane. The cursor and the selection keys of read mode work in the transcript. `y` copies the
selection, or the message under the cursor; `a` adds it to the chat input as a chip with `↩` in
its own color. `esc` or `i` goes back to the input.

**The queue.** While the agent answers, `enter` puts the question in a queue under the answer; the
next question of the queue goes when the turn ends, as in Claude Code and Codex. `esc` stops the
answer and puts the queued questions back into the input. While the Claude Code session of the
agent session runs, the question goes to it (see [Sessions](sessions.md)).

**Slash commands.** zendo passes the slash commands of the agent through. The agent sends its
list (ACP `available_commands_update`). `/` and the start of a name at the start of the input show
the 4 best commands with their descriptions; the arrows choose, `tab` (or `enter` on a name that is
not complete) takes one. The command goes to the agent as the user typed it.

**Attachments.** When the user drops files on the terminal, or pastes their paths, the chat input
gets one chip for each file: `[image name]` for an image, `[file name]` for any other file. `ctrl-v`
in the chat pastes an image from the clipboard as an `[image …]` chip. zendo sends an image as image
data when the agent takes images, and every other file as a link to it.

## Detailed requirements

| ID        | Pattern    | Requirement | Source |
| --------- | ---------- | ----------- | ------ |
| CLAUDE-15 | Ubiquitous | The system shall let the user move and edit anywhere in the chat input, and show it on up to a quarter of the panel height. | User, 2026-11-11 |
| CLAUDE-30 | Event      | When the user presses `alt-enter`, `shift-enter` or `ctrl-j` in the chat input, the system shall add a line, and the status bar shall name `alt-enter`. | User, 2026-11-12 |
| CLAUDE-16 | Event      | When the user drops or pastes the paths of existing files into the chat input, the system shall show one chip for each file and send the files with the question. | User, 2026-11-12 |
| CLAUDE-17 | Event      | When the user pastes an image from the clipboard into the chat input, the system shall save it and show it as an `[image …]` chip. | User, 2026-11-12 |
| CLAUDE-29 | Event      | When the user presses `tab` in the chat input, the system shall move the keys to the transcript, where the cursor and the selection keys of read mode work, `y` copies, and `a` adds the text to the input as a chip of its own color. | User, 2026-11-12 |
| CLAUDE-33 | Event      | When the user sends a question while the agent answers, the system shall show it in a queue under the answer, and send it when the turn ends. | User, 2026-11-12 |
| CLAUDE-34 | Event      | When the user presses `esc` while the agent answers, the system shall stop the answer and put the queued questions back into the chat input. | User, 2026-11-12 |
| CLAUDE-28 | Event      | When the user types `/` at the start of the chat input, the system shall show the slash commands of the agent that match, and send a command to the agent as the user typed it. | User, 2026-11-12 |
| CLAUDE-18 | Optional   | Where the agent takes images, the system shall send an attached image as image data; otherwise as a link to the file.         | User, 2026-11-12 |
| CLAUDE-19 | Event      | When the user presses `enter` after a `\` at the end of the chat input, the system shall replace the `\` with a new line.    | User, 2026-11-12 |
| CLAUDE-20 | Ubiquitous | The system shall label a selection chip with the first 15 chars of the selection, and `+N` when it has N more lines.                                 | User, 2026-11-12 |
| CLAUDE-21 | Ubiquitous | The system shall label a file chip with the last 4 chars of its folder and the first 10 chars of its name.         | User, 2026-11-12 |
| CLAUDE-22 | Event      | When the user types `@` and a part of a file name in the chat input, the system shall show the 4 best matching files under the input, also for a name with a typo, and add the chosen file as a chip. | User, 2026-11-12 |
| CLAUDE-23 | Event      | When the user presses an arrow, `backspace` or `delete` next to a chip, the system shall move over or remove the whole chip. | User, 2026-11-12 |
| CLAUDE-24 | Ubiquitous | The system shall show the messages of the user on a block of their own color, and the messages of the agent on the panel, with inline code that stands out from the panel. | User, 2026-11-12 |
| CLAUDE-25 | Event      | When the user presses `ctrl-t` in the chat input, the system shall suggest only templates, as `@` suggests files, and add the chosen one as a chip that tells the agent to write a new document from it. | User, 2026-11-12 |

Tests that verify detailed requirements: `dropped_files_are_paths_and_nothing_else` (CLAUDE-16),
`images_and_files_go_with_the_prompt` (mock agent; CLAUDE-16, CLAUDE-18),
`a_selection_chip_says_its_first_chars` (CLAUDE-20), `a_file_chip_says_its_folder_tail_and_name_head`
(CLAUDE-21), `files_match_by_start_part_and_typo` (CLAUDE-22), `user_and_agent_messages_look_different` (CLAUDE-24), `templates_are_named_ones_and_those_of_templates_folders`,
`ctrl_t_suggests_only_templates` (CLAUDE-25), `a_name_that_starts_with_the_query_comes_first` and
`slash_commands_come_from_the_agent_and_go_to_it_as_typed` (CLAUDE-28),
`alt_enter_is_a_new_line_and_the_status_bar_says_so` (CLAUDE-15, CLAUDE-30),
`the_cursor_moves_and_selects_in_the_transcript` and `tab_goes_to_the_transcript_to_select_copy_and_add` (CLAUDE-29), `at_adds_a_file_chip_that_moves_and_goes_as_one`
(CLAUDE-22, CLAUDE-23), `a_question_while_the_agent_answers_waits_in_the_queue` (CLAUDE-33) and
`esc_stops_and_puts_the_queue_back_in_the_input` (CLAUDE-34) in `tests/chat_queue.rs`.
