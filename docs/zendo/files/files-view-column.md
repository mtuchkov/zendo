---
status: "implemented"
approved: "2026-11-18"
revision: "1"
date: "2026-11-18"
parent: "files.md"
prev: "files-views.md"
next: "files-actions.md"
level: "the strip of view letters on the left edge of the tree, its hint, and the Notes view"
note: "Design that the user decided and approved on 2026-11-18. FILES-78 and `∆` for `Δ`: the fonts iA Writer Mono and IBM Plex Mono have no `●`, `◆` and `Δ`, so the terminal drew them two cells wide (the user, 2026-11-18). It moves the notes list of notes-list.md into the tree."
---

# File tree: the view strip and the Notes view

Parent: [File tree](files.md). The views are in [File tree: views](files-views.md).

## The view strip

The left edge of the tree pane has a strip of two columns: one letter for each view, one under the
other with an empty row between them, and one empty column after the letters. The first letter
stands at the first row of files, two rows under the head.

| Letter | View |
| ------ | ---- |
| `S`    | structure |
| `D`    | docs |
| `T`    | text |
| `W`    | worktree |
| `∆`    | worktree changed |
| `N`    | notes |

The letter of the active view stands on the accent color; the other letters have the second accent
color. `W` and `∆` show only while the active agent session has a worktree, and `S` only while
`project-structure.toml` has main materials, as in the `v` cycle (FILES-4). A line separates `N`
from the views of the files. Where the pane is too low for the empty rows, the letters stand
without them. A click on a letter shows its view. The head of the tree names the view
and its key, as before: `structure · v`.

```
    ZENDO · structure · v
                     [+] [-]
 S  docs/zendo
      find.md
 D  src

 T

 ─

 N
```

## The hint

When the user presses `v`, the tree shows the next view (FILES-4), and the strip opens into a hint
over the tree: each letter with the name of its view, the active one marked. The tree under the
hint keeps its width; the hint is a box at 80% opacity, one column wider than the names, so the
tree under it shows at 20%. After 2.5 seconds the hint closes into the strip again; a `v` while it is open
starts the 2.5 seconds again. A click on a name shows its view.

```
 S structure     ZENDO · text · v
 D docs
▐T text▌
 W worktree
 ∆ worktree ∆
 ─
 N notes
```

## The Notes view

The Notes view lists the notes by age, under the heads Today, This week and Older, as the notes
list did ([Notes list and search](../notes-list.md)). It has no folders, so it has no `[+]` and
`[-]`, and `+`, `-`, `*` and `_` do nothing. `enter` opens the selected note, `n` writes a new note
in a [float](../float.md), `f2` renames the file to a typed name, and `delete` deletes it after a
`y`. `o` opens the tree in the Notes view; the notes list as a screen of its own goes away.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-68 | Ubiquitous | The system shall show a strip of two columns on the left edge of the tree pane: one letter for each view, one under the other, and an empty column. | User, 2026-11-18 |
| FILES-69 | Ubiquitous | The system shall have a Notes view in the tree that lists the notes by age, with the keys `enter`, `n`, `f2` and `delete`. | User, 2026-11-18 |

### Acceptance tests

**`the_views_stand_in_a_strip`** (unit test; verifies FILES-68)

- Given a repository with `project-structure.toml`, docs and notes, and the tree open.
- When the user draws the screen, then clicks `T`.
- Then the left edge of the tree shows `S`, `D`, `T` and `N`, one under the other, with an empty
  column after them; `S` stands on the accent color; after the click the tree shows the text view
  and `T` stands on the accent color.

**`the_notes_view_lists_the_notes_by_age`** (unit test; verifies FILES-69)

- Given notes of today and of an older day.
- When the user presses `o`, then `enter`.
- Then the tree shows the Notes view with the note of today under Today and the older note under
  Older, without `[+]` and `[-]`; `enter` opens the selected note in the note pane.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-70 | Ubiquitous | The system shall label the views `S`, `D`, `T`, `W`, `∆` and `N`. | User, 2026-11-18 |
| FILES-71 | Ubiquitous | The system shall show the letter of the active view on the accent color and the other letters in the second accent color. | User, 2026-11-18 |
| FILES-72 | State      | While the active agent session is not session 0 and has no worktree, the system shall not show `W` and `∆` (FILES-81, FILES-82 in [Worktree views](files-worktree.md)); while `project-structure.toml` has no main materials, `S`. | Design |
| FILES-73 | Event      | When the user clicks a letter of the strip or a name of the hint, the system shall show that view until the user chooses another or quits (FILES-66). | User, 2026-11-18 |
| FILES-74 | Event      | When the user presses `v` in the tree, the system shall show over the tree, for 2.5 seconds after the last `v`, the hint with the letter and the name of each view, without a change of the width of the tree. | User, 2026-11-18 |
| FILES-75 | Ubiquitous | The system shall name the active view and its key in the head of the tree. | User, 2026-11-18 |
| FILES-76 | Event      | When the user presses `o`, the system shall open the tree in the Notes view, with the focus in the tree. | User, 2026-11-18 |
| FILES-78 | Ubiquitous | The system shall draw in the tree only chars that every font of `zendo setup fonts` has, so that each char takes one cell: `•` for the open file, `§` for a section, `∆` for the worktree changed view. | User, 2026-11-18 |
| FILES-77 | State      | While the tree shows the Notes view, the system shall not show `[+]` and `[-]`, and `+`, `-`, `*` and `_` shall do nothing. | User, 2026-11-18 |

Tests that verify detailed requirements: `the_view_letters` (FILES-70),
`the_active_letter_has_the_accent_color` (FILES-71), `the_worktree_letters_need_a_worktree`
(FILES-72), `a_click_on_a_letter_keeps_the_view` (FILES-73), `v_shows_the_hint_for_2_5_seconds`
(FILES-74), `the_head_names_the_view` (FILES-75), `o_opens_the_notes_view` (FILES-76),
`the_notes_view_has_no_fold_buttons` (FILES-77), `the_tree_draws_only_chars_of_every_font` (FILES-78).
