---
status: "implemented"
approved: "2026-11-13"
revision: "3"
date: "2026-11-13"
parent: "zendo.md"
prev: "float.md"
next: "notes-repo.md"
level: "the find panel: recent files, search by path and by text, the preview"
note: "Revision 2 was approved by the user on 2026-11-13 and implemented. Revision 3 adds FIND-18 (the notes of the notes repo), approved by the user on 2026-11-13."
---

# Find: recent files and search

The find panel opens a file of the repository. `alt-/` opens it in every pane and every mode, also
in write mode, where `/` types, and over the full screen of a Claude Code session or a terminal. `/` in read mode opens the same panel. The panel stands over the
screen. It has the query field at the top, the list on the left, and the preview of the selected
item on the right.

## Recent files

With an empty query, the list shows the recent files: the files that the user opened in the note
pane in this repository, the most recent first. The open file is not in the list, so the first row
is the file before it, and `alt-/` `enter` goes back to that file. Each row shows the title of the
file (the file name when it has no `# ` title), its path in dim colors, and when the user opened it:
the time today, the day this week, the date before.

A file counts as opened when it shows in the note pane: from the file tree, the notes list, the find
panel, a `[[link]]`, `]` and `[`, and a new note from the [float](float.md). The preview of the file
tree (`l`) and the preview of the find panel do not count. zendo keeps the recent files for the
repository in its state, so they are there after a restart. A file that is gone does not show; a
rename in zendo keeps the file in its place in the list.

## Search

When the user types, the list shows first the files whose path has every typed word, recent files
first, and then the markdown files of the repository whose text has the query, each with up to two lines of
the text.
A query that starts with `#` finds a tag, and one that starts with `?` asks the agent about all
notes, as in [Notes list and search](notes-list.md). `/` in the notes list keeps its own search: the text of
the notes only, without the recent files and the preview.

## The preview

When the selection moves in the list, the preview shows the selected file as read mode shows it,
from the top. For a match in the text, the preview shows the line of the match near the top, with
the match marked. The preview is read only; the focus stays in the query field.

Design: where the panel is narrower than 100 chars, the preview stands under the list.

## Keys

`up` and `down`, `ctrl-j` and `ctrl-k`, `tab` and `shift-tab` move the selection; letters type into
the query. `enter` opens the selected file in the note pane in read mode, at the line of the match.
`esc` closes the panel and changes nothing. These keys are fixed, as in the search box today.

## Architecturally significant requirements

| ID     | Pattern    | Requirement                                                                                                                                      | Source           |
| ------ | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------- |
| FIND-1 | Event      | When the user presses `alt-/` in any pane or mode, or `/` in read mode, the system shall open the find panel.                                    | User, 2026-11-13 |
| FIND-2 | State      | While the query is empty, the system shall list the files that the user opened in the note pane in this repository, the most recent first, without the open file. | User, 2026-11-13 |
| FIND-3 | Event      | When the selection moves in the list, the system shall show the selected file in the preview of the panel, as read mode shows it.                | User, 2026-11-13 |
| FIND-4 | Ubiquitous | The system shall keep the recent files for the repository in its state, so that a restart keeps them.                                           | Design           |

### Acceptance tests

**`alt_slash_opens_find_from_every_pane`** (unit test; verifies FIND-1)

- Given a repository with notes, a note open in write mode.
- When the user presses `alt-/` in write mode, in the chat, in the file tree, and `/` in read mode.
- Then each time the find panel opens with an empty query, and in write mode no `/` is typed.

**`an_empty_query_lists_the_recent_files`** (unit test; verifies FIND-2)

- Given the user opened `a.md`, then `b.md`, then `c.md`.
- When the user presses `alt-/`.
- Then the list shows `b.md` and `a.md`, in this order, and `b.md` is selected.

**`moving_in_the_list_shows_the_preview`** (integration test on a test terminal of 140×40; verifies FIND-3)

- Given the recent files `b.md` and `a.md`.
- When the user opens the panel and presses `down`.
- Then the preview shows the title and the first lines of `a.md`, formatted as in read mode.

**`the_recent_files_stay_after_a_restart`** (unit test; verifies FIND-4)

- Given the user opened `a.md`, then `b.md`, and quit.
- When the user starts zendo again, opens `c.md` and presses `alt-/`.
- Then the list shows `b.md`, then `a.md`.

## Detailed requirements

| ID      | Pattern    | Requirement                                                                                                                                           | Source           |
| ------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| FIND-5  | Ubiquitous | The system shall show in each row of the recent files the title or the file name, the path, and when the user opened the file.                        | Design           |
| FIND-6  | Event      | When a file shows in the note pane, other than in the preview of the file tree or of the find panel, the system shall make it the most recent file.   | Design           |
| FIND-7  | Unwanted   | If a recent file is not on disk, then the system shall not show it.                                                                                   | Design           |
| FIND-8  | Event      | When the user renames or moves a file in zendo, the system shall keep it in its place in the recent files with the new path.                          | Design           |
| FIND-9  | Event      | When the user types a query, the system shall list the files whose path has every typed word, recent files first, and then the markdown files of the repository whose text has the query, without regard to letter case. | User, 2026-11-13 |
| FIND-10 | Event      | When the selected item is a match in the text, the system shall show the line of the match near the top of the preview, with the match marked.        | Design           |
| FIND-11 | Event      | When the user presses `enter`, the system shall open the selected file in the note pane in read mode, at the line of the match.                       | Design           |
| FIND-12 | Event      | When the user presses `esc`, the system shall close the panel and keep the file, the pane, the mode and the cursor as they were.                      | Design           |
| FIND-13 | State      | While the panel is narrower than 100 chars, the system shall show the preview under the list.                                                         | Design           |
| FIND-14 | Ubiquitous | The system shall keep NOTE-13 and NOTE-14 in the find panel: `#` finds a tag, and `?` asks the agent about all notes.                                 | NOTE-13, NOTE-14 |
| FIND-15 | State      | While a Claude Code session or a terminal is on the full screen, the system shall open the find panel over it when the user presses `alt-/`, and not send the key to the program. | User, 2026-11-13 |
| FIND-17 | Event      | When the user presses `/` in the notes list, the system shall search the text of the notes only, as NOTE-12, and not open the find panel. | User, 2026-11-13 |
| FIND-16 | Unwanted   | If a markdown file is in a hidden folder or git ignores it, then the system shall not search its text (FILES-1).                                       | FILES-1          |
| FIND-18 | State      | While the notes repo is on, the system shall search the notes of the repository in the notes repo by path and by text, as the markdown files of the repository (FIND-9, FILES-25). | User, 2026-11-13 |

Tests that verify detailed requirements: `the_row_shows_title_path_and_time` (FIND-5),
`a_tree_preview_is_not_recent` and `a_note_from_the_float_is_recent` (FIND-6), `a_deleted_file_is_not_recent` (FIND-7),
`a_rename_keeps_the_recent_place` (FIND-8), `a_query_finds_paths_then_text` (FIND-9),
`the_preview_shows_the_match_and_enter_opens_at_it` (FIND-10, FIND-11), `esc_changes_nothing` (FIND-12),
`a_narrow_panel_shows_the_preview_under_the_list` (FIND-13), the tests of `src/docs/search_tests.rs` (FIND-14),
`alt_slash_opens_find_over_a_full_screen_session` in `tests/terminal.rs` (FIND-15), `ignored_files_are_not_searched`
(FIND-16), `slash_in_the_notes_list_searches_the_notes` (FIND-17), and
`a_query_finds_the_notes_of_the_notes_repo` (FIND-18).

