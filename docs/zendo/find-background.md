---
status: "implemented"
approved: "2026-11-17"
revision: "1"
date: "2026-11-17"
parent: "zendo.md"
prev: "find-ids.md"
next: "find-regex.md"
level: "the search of the find panel off the UI thread: the list of files, the search in the text, and the merge of All"
note: "Design that the user decided and approved on 2026-11-17. The mechanism and the measurements are in ADR 0011."
---

# Find: search in the background

The find panel is in [Find](find.md), its modes in [Find: modes](find-modes.md).

## The UI does not wait

The search runs off the UI thread. Keys, the cursor of the query and the drawing of the screen do
not wait for it, also in a large repository. Files change often, so zendo keeps no copy of their
text: each search reads the files from the disk.

- **The list of files.** When the panel opens, zendo walks the repository again in the background.
  The search by path uses this list until the panel closes. So a file that the user made since the
  last open is found.
- **The search in the text.** When the user stops typing, zendo searches the text of the files for
  the query in the background, with the search engine of ripgrep. One search gives the results of
  every mode: zendo sorts each file into Notes, Docs or Code by its path. The text of code is
  searched only for a requirement ID ([Find: tags and requirement IDs](find-ids.md)).
- When the user types again, zendo stops the search of the old query and drops its results, so a
  late result of an old query does not show.

Design: the list shows the matches by path at once, and the matches in the text when the search
ends. While the search runs, each mode in the mode row shows a dim `…` after its name; when it
ends, the number of results, for example `Notes 3  Docs 12  Code 0  All 15`.

## All

All mode has no search of its own. Its list is the merge of the lists of Notes, Docs and Code, in
one sort order: the matches by path first, recent files first; then the matches in the text, the
most hits first. A query that starts with `?` is not a search: the agent gets it only in Notes
mode.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FIND-34 | Ubiquitous | The system shall run the walk of the repository and the search in the text of the files off the UI thread, so that keys and the drawing of the screen do not wait for them. | User, 2026-11-17 |
| FIND-35 | Event      | When the query changes, the system shall search the text of the files on the disk once for every mode, and sort the matches into Notes, Docs and Code by the path of the file. | User, 2026-11-17 |
| FIND-36 | Ubiquitous | The system shall keep no copy of the text of the files between two searches. | User, 2026-11-17 |

### Acceptance tests

**`typing_does_not_wait_for_the_search`** (unit test; verifies FIND-34)

- Given a repository with 2,000 docs that each have the word `needle`.
- When the user types `needle` in the find panel.
- Then each key returns in less than 50 ms, the query shows `needle` at once while the search in
  the text runs, and the list shows the results when they come.

**`one_search_fills_every_mode`** (unit test; verifies FIND-35)

- Given a repository with a note, a doc and a code file that each have `find` in the text and not
  in the path.
- When the user types `find` in Docs mode, waits for the search, and presses `tab` three times.
- Then the search in the text ran one time; Docs shows the doc, Code shows nothing (code text is
  searched only for an ID), All shows the note and the doc, and Notes shows the note.

**`a_changed_file_is_found_without_a_new_open`** (unit test; verifies FIND-36)

- Given the panel is open with the query `alpha`, and no file has `alpha`.
- When a doc gets the word `alpha` on the disk, and the user types ` ` and deletes it.
- Then the list shows the doc.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FIND-37 | Event      | When the find panel opens, the system shall walk the repository again, and search by path in that list of files until the panel closes. | User, 2026-11-17 |
| FIND-38 | Event      | When the query changes while a search runs, the system shall stop that search and drop its results. | Design |
| FIND-39 | Unwanted   | If the query starts with `?`, then the system shall not search the text, and shall ask the agent only while the panel is in Notes mode. | Design |
| FIND-40 | Ubiquitous | The system shall make the list of All mode as the merge of the lists of Notes, Docs and Code, in one sort order, with no search of its own. | User, 2026-11-17 |
| FIND-41 | Ubiquitous | The system shall show in the mode row a dim `…` after each mode while the search runs, and the number of results of each mode when it ends. | Design |

Tests that verify detailed requirements: `a_new_file_is_found_after_the_panel_opens_again`
(FIND-37), `a_late_result_of_an_old_query_does_not_show` (FIND-38),
`a_question_is_not_searched` (FIND-39), `all_is_the_merge_of_the_three_modes` (FIND-40),
`the_mode_row_shows_counts_and_running_searches` (FIND-41).
