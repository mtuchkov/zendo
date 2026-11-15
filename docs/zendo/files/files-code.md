---
status: "implemented"
approved: "2026-11-13"
revision: "1"
date: "2026-11-14"
parent: "files.md"
prev: "files-actions.md"
next: "files-worktree.md"
level: "how a file that is not markdown shows in the note pane"
note: "Split from files.md revision 2 (user, 2026-11-14). No requirement changed."
---

# File tree: code files

Parent: [File tree](files.md).

A file that is not markdown opens in the note pane as code, in the colors of its language, 120 chars
wide (`alt-=` and `alt--` change it), wrapped, with line numbers. A wrapped line has its number on
its first row only. Write mode shows code the same way, does not break long lines, and keeps the
indent on `enter`.

The text view of the tree, which lists these files, is in [Views](files-views.md).

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-21 | Event | When the user opens a file that is not markdown, the system shall show it as code in the colors of its language, for every language that zendo knows. | User, 2026-11-12 |
| FILES-22 | Ubiquitous | The system shall show a code file 120 chars wide by default, wrapped, with the number of each line on its first row only, and let `alt-=` and `alt--` change the width. | User, 2026-11-12 |
| FILES-23 | Ubiquitous | The system shall show a code file in write mode as in read mode, shall not break its long lines, and shall keep the indent of the line on `enter`. | User, 2026-11-12 |

Tests that verify detailed requirements: `a_whole_file_uses_every_language_that_zendo_knows` and
`tests/tree_text.rs` (FILES-21 to FILES-23), `a_long_line_wraps_and_each_row_knows_where_it_starts`
(FILES-22).
