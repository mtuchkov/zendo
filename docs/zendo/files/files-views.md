---
status: "draft"
revision: "4"
date: "2026-11-17"
parent: "files.md"
next: "files-view-column.md"
level: "the views of the file tree, folding, the marks, the bottom of the tree"
note: "Split from files.md revision 2 (user, 2026-11-14). Revision 2 changes FILES-4: the `v` cycle has the two worktree views after folders (user, 2026-11-14); implemented in zendo 0.62.0. Revision 3, from the request of the user on 2026-11-17 and approved by the user on 2026-11-17: the all and folders views go (FILES-4, FILES-17), the folders start closed (FILES-56), `[+]` and `[-]` open and close every folder (FILES-57), and the tree follows the open document (FILES-58); implemented in zendo 0.85.0. Revision 4 (agent draft, not reviewed): FILES-4 ends with the Notes view of files-view-column.md."
---

# File tree: views

Parent: [File tree](files.md).

The file tree has three views of the repository:

- **structure**: the main materials of [project-structure.toml](../project-structure.md), in the
  order of that file;
- **docs**: the files that match `**/docs/**`;
- **text**: every text file: markdown, code and configuration, and every folder, also an empty
  one. The text view shows each extension in the color of its kind of file (prose, code,
  configuration, scripts, web), from the theme. How a file that is not markdown opens is in
  [Code files](files-code.md).

`v` shows the next view, after text the two worktree views, then the Notes view. A strip of
letters on the left edge of the tree shows the views; see [The view strip](files-view-column.md). `j` and `k` move; `+` and `-`, `l` and
`h`, or a click on `[+]` and `[-]` open and close a folder. zendo keeps the view for the repository.

Every folder and main material starts closed. Under the head of the tree, on the right, `[+]` opens
every folder and `[-]` closes every folder; the keys `*` and `_` do the same.

While the tree shows, it follows the open document: it opens the folders above the document and
selects its row. It does this when the tree opens, when the agent session changes, when a file opens
(from the tree, from find, from a search result, also the file that is open), and when anything else
changes the open document. Outside the worktree views, a file of the worktree of the session shows
at its place in the repository.

At the bottom, the tree shows what the selected item is (`about`) and the start of its rules for
Claude.

Two more views show the worktree of the active agent session: worktree and worktree changed; see
[Worktree views](files-worktree.md).

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-4 | Event | When the user presses `v` in the tree, the system shall show the next view: structure, docs, text, worktree, worktree changed, notes. | User, 2026-11-12; User, 2026-11-14; User, 2026-11-17; User, 2026-11-18 |
| FILES-5 | Event | When the user clicks `[+]` or `[-]`, or presses its key, the system shall open or close the folder. | User, 2026-11-11 |
| FILES-6 | Ubiquitous | The system shall show a folder that holds only one folder as one row, for example `svc/orders/docs`. | Design |
| FILES-7 | Ubiquitous | The system shall mark with `Δ` the files that changed since the last visit that counted. | Design |
| FILES-8 | Ubiquitous | The system shall show the `about` and the start of the Claude rules of the selected item at the bottom. | User, 2026-11-11 |
| FILES-9 | Event | When a file of the tree changes on disk, the system shall show the change in the tree without a restart. | Design |
| FILES-10 | Event | When the user changes the view or opens and closes folders, the system shall keep them for the repository. | Design |
| FILES-17 | Ubiquitous | The text view shall show every folder, also an empty one or one without a text file. | User, 2026-11-12; User, 2026-11-17 |
| FILES-20 | Ubiquitous | The system shall have a text view that shows every text file that git does not ignore: a file of at most 2 MB whose start is UTF-8 without a NUL byte. | User, 2026-11-12 |
| FILES-24 | Ubiquitous | The system shall show each file extension in the text view in the color that the theme gives its kind of file. | User, 2026-11-12 |
| FILES-56 | Ubiquitous | The system shall show each folder and each main material of the tree closed until the user or FILES-58 opens it. | User, 2026-11-17 |
| FILES-57 | Event | When the user clicks `[+]` or `[-]` under the head of the tree, or presses its key, the system shall open or close every folder and main material of the view. | User, 2026-11-17 |
| FILES-58 | Event | While the tree shows, when the tree opens, the agent session changes, a file opens, or the open document changes, the system shall open the folders above the open document and select its row. | User, 2026-11-17 |

Tests that verify detailed requirements: `rows_join_single_folders_and_collapse` (FILES-5, FILES-6,
FILES-56), `the_text_view_shows_every_folder` (FILES-17), `structure_view_lists_main_materials_in_file_order`
and `the_worktree_views_follow_text_only_with_a_worktree` (FILES-4), `the_text_view_shows_every_text_file` (FILES-20),
`expand_all_and_collapse_all` (FILES-57), `the_tree_follows_the_open_document` (FILES-58),
`tests/tree_fold.rs` (FILES-56, FILES-57, FILES-58), and `tests/tree_text.rs` (FILES-20, FILES-24).
