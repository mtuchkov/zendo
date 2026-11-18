---
status: "draft"
revision: "2"
date: "2026-11-17"
parent: "files.md"
prev: "files-views.md"
next: "files-code.md"
level: "the actions on a file in the file tree: filter, rename, move, delete, new, add to the chat"
note: "Split from files.md revision 2 (user, 2026-11-14). Revision 2 (agent draft, not reviewed): the filter moves from `/` to `` ` ``, `/` opens the find panel (FIND-20), and the filter gets the regex button (FILES-59)."
---

# File tree: actions

Parent: [File tree](files.md).

Keys in the file tree: `enter` opens a file; `a` adds the file to the chat; `/` opens the find panel
([Find: modes](../find-modes.md)).

`` ` `` filters the files by name: the tree becomes a flat list of the files whose path has every typed
word, `enter` opens the selected file, and `esc` shows the tree again with the match selected. The
filter line has the button `[.*]` at its end: a click on it or `alt-r` makes the typed text a
regular expression for the path, as in the find panel ([Find: regex](../find-regex.md)).

`f2` renames the selected file in its folder, `m` moves it to another folder, and `delete` deletes
it after a `y`. `f3` makes a new file and `f4` a new folder; their field starts with the folder of
the selection, and `tab` completes a folder name. The question and its field are on a line under
the status bar.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-11 | Event | When the user types after `` ` `` in the file tree, the system shall show only the files whose path has every typed word, without regard to letter case. | User, 2026-11-12; key: User, 2026-11-17 |
| FILES-59 | Event | When the user clicks `[.*]` at the end of the filter line or presses `alt-r` in the filter of the file tree, the system shall match the typed text as a regular expression in the path, without regard to letter case, or as words again. | User, 2026-11-17 |
| FILES-12 | Event | When the user presses `delete` on a file in the file tree, the system shall ask, and delete the file only after `y`. | User, 2026-11-12 |
| FILES-13 | Event | When the user presses `f2` on a file in the file tree and types a name, the system shall rename the file in its folder, keep the extension when the name has none. | User, 2026-11-12 |
| FILES-14 | Event | When the user presses `m` on a file in the file tree and types a folder of the repository, the system shall move the file there, make the folder when it is not there. | User, 2026-11-12 |
| FILES-15 | Event | When the user presses `a` on a file in the file tree, the system shall add the file to the chat as an attachment and keep the focus in the tree. | User, 2026-11-12 |
| FILES-16 | Ubiquitous | The system shall ask the questions of a delete, a rename, a move and a new file or folder on a line under the status bar. | User, 2026-11-12 |
| FILES-18 | Event | When the user presses `f3` or `f4` in the file tree, the system shall ask for the path of a new file or folder, start the field with the folder of the selection, and complete a folder name on `tab`. | User, 2026-11-12 |
| FILES-19 | Event | When the user makes a new markdown file in a folder whose entry names a `template`, the system shall start the file from the template, with the title from the file name. | User, 2026-11-12 |

Tests that verify detailed requirements: `find_filters_files_by_name` (FILES-11), `the_tree_filter_takes_a_regex` (FILES-59),
`a_new_name_stays_in_the_folder_and_keeps_the_extension` (FILES-13),
`a_new_file_starts_from_the_template_of_its_folder` (FILES-19), and the flows in
`tests/ui_flows.rs` (FILES-11, FILES-14 to FILES-16, FILES-18).
