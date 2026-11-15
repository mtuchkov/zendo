---
status: "implemented"
approved: "2026-11-13"
revision: "3"
date: "2026-11-14"
parent: "../zendo.md"
prev: "../length.md"
next: "../work.md"
children:
  - "files-views.md"
  - "files-actions.md"
  - "files-code.md"
  - "files-worktree.md"
  - "files-worktree-recap.md"
level: "the file tree pane as a whole: which files it lists, that it has views, what opening a file does"
note: "Agent rewrite of the reference of zendo 0.9.1. Revision 2 adds FILES-25 (the notes of the notes repo), approved by the user. Revision 3: split into children by topic (user, 2026-11-14); no requirement changed. The children files-worktree.md and files-worktree-recap.md are drafts."
---

# File tree

The file tree lists the markdown files of the repository. While the [notes repo](../notes-repo.md) is
on, the notes of the repository live outside it, so the structure view shows them as a `notes`
folder at the top, or the docs view when the repository has no main materials. The tree sits on the
left of the screen and opens and closes with `ctrl-shift-b` or `alt-b` in every pane, or `e` in read
mode.

The tree has five views: structure, docs, all, text and folders. `v` shows the next view. `enter`
opens the selected file in the note pane, where it works like a note.

The children of this document hold the details:

| Document                              | What it specifies                                                        |
| ------------------------------------- | ------------------------------------------------------------------------ |
| [Views](files-views.md)               | The five views, folding, the marks, the bottom of the tree               |
| [Actions](files-actions.md)           | The filter, rename, move, delete, new file and folder, add to the chat   |
| [Code files](files-code.md)           | How a file that is not markdown shows in the note pane                   |
| [Worktree views](files-worktree.md)   | Draft: the worktree of the agent session and its changed files           |
| [Worktree recap](files-worktree-recap.md) | Draft: the recap of the changes of the worktree, and when it refreshes |

## Architecturally significant requirements

| ID      | Pattern    | Requirement                                                                                                                                       | Source           |
| ------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| FILES-1 | Ubiquitous | The system shall list the markdown files of the repository, and skip the files that git ignores and the hidden folders.                           | User, 2026-11-11 |
| FILES-2 | Ubiquitous | The system shall show the structure view first when the repository has main materials, and the docs view first when it does not.                  | User, 2026-11-11 |
| FILES-3 | Event      | When the user opens a file from the tree, the system shall open it like a note, with read, write, delta, recap and Claude.                         | User, 2026-11-11 |
| FILES-25 | State     | While the notes repo is on, the system shall show the notes of the repository from the notes repo as a `notes` folder at the top of the structure view, or of the docs view when the repository has no main materials. | User, 2026-11-13 |

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

**`the_tree_shows_the_notes_of_the_notes_repo`** (unit test; verifies FILES-25)

- Given a test repository with `project-structure.toml` and `docs/design.md`, and a notes repo with
  one note of this repository.
- When zendo opens the file tree in the structure view.
- Then the first row is the `notes` folder, it holds the note, and opening the note opens the file
  in the notes repo.
- And when the test repository has no `project-structure.toml`, the docs view shows the same
  `notes` folder first.
