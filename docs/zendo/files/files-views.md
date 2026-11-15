---
status: "implemented"
approved: "2026-11-14"
revision: "2"
date: "2026-11-14"
parent: "files.md"
next: "files-actions.md"
level: "the views of the file tree, folding, the marks, the bottom of the tree"
note: "Split from files.md revision 2 (user, 2026-11-14). Revision 2 changes FILES-4: the `v` cycle has the two worktree views after folders (user, 2026-11-14); implemented in zendo 0.60.0."
---

# File tree: views

Parent: [File tree](files.md).

The file tree has five views of the repository:

- **structure**: the main materials of [project-structure.toml](../project-structure.md), in the
  order of that file;
- **docs**: the files that match `**/docs/**`;
- **all**: every markdown file;
- **text**: every text file: markdown, code and configuration. The text view shows each extension
  in the color of its kind of file (prose, code, configuration, scripts, web), from the theme. How
  a file that is not markdown opens is in [Code files](files-code.md);
- **folders**: every folder, also an empty one or one without markdown, and every markdown file.

`v` shows the next view, after folders the two worktree views. `j` and `k` move; `+` and `-`, `l` and `h`, or a click on `[+]` and `[-]`
open and close a folder. zendo keeps the view and the open folders for the repository.

At the bottom, the tree shows what the selected item is (`about`) and the start of its rules for
Claude.

Two more views show the worktree of the active agent session: worktree and worktree changed; see
[Worktree views](files-worktree.md).

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-4 | Event | When the user presses `v` in the tree, the system shall show the next view: structure, docs, all, text, folders, worktree, worktree changed. | User, 2026-11-12; User, 2026-11-14 |
| FILES-5 | Event | When the user clicks `[+]` or `[-]`, or presses its key, the system shall open or close the folder. | User, 2026-11-11 |
| FILES-6 | Ubiquitous | The system shall show a folder that holds only one folder as one row, for example `svc/orders/docs`. | Design |
| FILES-7 | Ubiquitous | The system shall mark with `Δ` the files that changed since the last visit that counted. | Design |
| FILES-8 | Ubiquitous | The system shall show the `about` and the start of the Claude rules of the selected item at the bottom. | User, 2026-11-11 |
| FILES-9 | Event | When a file of the tree changes on disk, the system shall show the change in the tree without a restart. | Design |
| FILES-10 | Event | When the user changes the view or opens and closes folders, the system shall keep them for the repository. | Design |
| FILES-17 | Ubiquitous | The system shall have a folders view that shows every folder, also an empty one, and every markdown file. | User, 2026-11-12 |
| FILES-20 | Ubiquitous | The system shall have a text view that shows every text file that git does not ignore: a file of at most 2 MB whose start is UTF-8 without a NUL byte. | User, 2026-11-12 |
| FILES-24 | Ubiquitous | The system shall show each file extension in the text view in the color that the theme gives its kind of file. | User, 2026-11-12 |

Tests that verify detailed requirements: `rows_join_single_folders_and_collapse` (FILES-5, FILES-6),
`the_folders_view_shows_empty_folders` (FILES-17),
`the_worktree_views_follow_folders_only_with_a_worktree` (FILES-4), `the_text_view_shows_every_text_file` (FILES-20),
and `tests/tree_text.rs` (FILES-20, FILES-24).
