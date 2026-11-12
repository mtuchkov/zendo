---
status: "draft"
revision: "1"
date: "2026-11-11"
parent: "zendo.md"
prev: "length.md"
next: "work.md"
level: "the file tree pane: which files it lists, its views, folding, opening a file"
note: "Agent rewrite of the reference of zendo 0.9.1. Not reviewed by the user."
---

# File tree

The file tree lists the markdown files of the repository. It sits on the left of the screen and
opens and closes with `ctrl-shift-b` or `alt-b` in every pane, or `e` in read mode. It has five views:

- **structure**: the main materials of [project-structure.toml](project-structure.md), in the order
  of that file;
- **docs**: the files that match `**/docs/**`;
- **all**: every markdown file;
- **text**: every text file: markdown, code and configuration. A file that is not markdown opens in
  the note pane as code, in the colors of its language;
- **folders**: every folder, also an empty one or one without markdown, and every markdown file.

At the bottom, the tree shows what the selected item is (`about`) and the start of its rules for
Claude.

Keys in the file tree: `j` and `k` move, `enter` opens a file; `+` and `-`, `l` and `h`, or a click
on `[+]` and `[-]` open and close a folder; `v` shows the next view; `a` adds the file to the chat.
`/` filters the files by name: the tree becomes a flat list of the files whose path has every typed
word, `enter` opens the selected file, and `esc` shows the tree again with the match selected.
`f2` renames the selected file in its folder, `m` moves it to another folder, and `delete` deletes it
after a `y`. `f3` makes a new file and `f4` a new folder; their field starts with the folder of the
selection, and `tab` completes a folder name. The question and its field are on a line under the
status bar.

## Architecturally significant requirements

| ID      | Pattern    | Requirement                                                                                                                                       | Source           |
| ------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| FILES-1 | Ubiquitous | The system shall list the markdown files of the repository, and skip the files that git ignores and the hidden folders.                           | User, 2026-11-11 |
| FILES-2 | Ubiquitous | The system shall show the structure view first when the repository has main materials, and the docs view first when it does not.                  | User, 2026-11-11 |
| FILES-3 | Event      | When the user opens a file from the tree, the system shall open it like a note, with read, write, delta, recap and Claude.                         | User, 2026-11-11 |

### Acceptance tests

**`docs_filter_and_gitignore`** (unit test; verifies FILES-1)

- Given a test repository with markdown files in `docs/`, in other folders, in a hidden folder and in
  a folder that `.gitignore` excludes.
- When zendo lists the files.
- Then the list has no file of the hidden folder and no file that `.gitignore` excludes.

**`structure_view_lists_main_materials_in_file_order`** (unit test; verifies FILES-2)

- Given a test repository with `project-structure.toml` that has three main materials.
- When zendo opens the file tree.
- Then the tree shows the three main materials first, in the order of the file.

**`file_from_the_tree_works_like_a_note`** (terminal check, not written yet; verifies FILES-3)

- Given a test repository with `docs/design.md`.
- When the user opens it from the tree, writes a line and asks Claude about it.
- Then the file on disk has the line, and the chat shows the question with the file name.

## Detailed requirements

| ID       | Pattern    | Requirement                                                                                                    | Source           |
| -------- | ---------- | -------------------------------------------------------------------------------------------------------------- | ---------------- |
| FILES-4  | Event      | When the user presses `v` in the tree, the system shall show the next view: structure, docs, all, text, folders. | User, 2026-11-12 |
| FILES-5  | Event      | When the user clicks `[+]` or `[-]`, or presses its key, the system shall open or close the folder.            | User, 2026-11-11 |
| FILES-6  | Ubiquitous | The system shall show a folder that holds only one folder as one row, for example `svc/orders/docs`.          | Design           |
| FILES-7  | Ubiquitous | The system shall mark with `Δ` the files that changed since the last visit that counted.                       | Design           |
| FILES-8  | Ubiquitous | The system shall show the `about` and the start of the Claude rules of the selected item at the bottom.        | User, 2026-11-11 |
| FILES-9  | Event      | When a file of the tree changes on disk, the system shall show the change in the tree without a restart.       | Design           |
| FILES-10 | Event      | When the user changes the view or opens and closes folders, the system shall keep them for the repository.     | Design           |
| FILES-11 | Event      | When the user types after `/` in the file tree, the system shall show only the files whose path has every typed word, without regard to letter case. | User, 2026-11-12 |
| FILES-12 | Event      | When the user presses `delete` on a file in the file tree, the system shall ask, and delete the file only after `y`. | User, 2026-11-12 |
| FILES-13 | Event      | When the user presses `f2` on a file in the file tree and types a name, the system shall rename the file in its folder, keep the extension when the name has none. | User, 2026-11-12 |
| FILES-14 | Event      | When the user presses `m` on a file in the file tree and types a folder of the repository, the system shall move the file there, make the folder when it is not there. | User, 2026-11-12 |
| FILES-15 | Event      | When the user presses `a` on a file in the file tree, the system shall add the file to the chat as an attachment and keep the focus in the tree. | User, 2026-11-12 |
| FILES-16 | Ubiquitous | The system shall ask the questions of a delete, a rename, a move and a new file or folder on a line under the status bar. | User, 2026-11-12 |
| FILES-17 | Ubiquitous | The system shall have a folders view that shows every folder, also an empty one, and every markdown file. | User, 2026-11-12 |
| FILES-18 | Event      | When the user presses `f3` or `f4` in the file tree, the system shall ask for the path of a new file or folder, start the field with the folder of the selection, and complete a folder name on `tab`. | User, 2026-11-12 |
| FILES-19 | Event      | When the user makes a new markdown file in a folder whose entry names a `template`, the system shall start the file from the template, with the title from the file name. | User, 2026-11-12 |
| FILES-20 | Ubiquitous | The system shall have a text view that shows every text file that git does not ignore: a file of at most 2 MB whose start is UTF-8 without a NUL byte. | User, 2026-11-12 |
| FILES-21 | Event      | When the user opens a file that is not markdown, the system shall show it as code in the colors of its language, for every language that zendo knows. | User, 2026-11-12 |

Tests that verify detailed requirements: `rows_join_single_folders_and_collapse` (FILES-5, FILES-6),
`find_filters_files_by_name` (FILES-11),
`a_new_name_stays_in_the_folder_and_keeps_the_extension` (FILES-13),
`the_folders_view_shows_empty_folders` (FILES-17), `the_text_view_shows_every_text_file` (FILES-20),
`a_whole_file_uses_every_language_that_zendo_knows` and `tests/tree_text.rs` (FILES-20, FILES-21), `a_new_file_starts_from_the_template_of_its_folder`
(FILES-19), and the flows in `tests/ui_flows.rs` (FILES-11, FILES-14 to FILES-16, FILES-18).
