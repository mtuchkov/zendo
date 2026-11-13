---
status: "draft"
revision: "1"
date: "2026-11-11"
parent: "zendo.md"
prev: "find.md"
next: "length.md"
level: "where the notes live when the user has a private notes repo, and how zendo keeps it in sync"
note: "Agent draft of the design that the user decided on 2026-11-11. Not reviewed by the user."
---

# Notes repo

By default, zendo keeps the notes of a repository in that repository, in `docs/notes`. The user can
instead keep all notes in one private git repository, the notes repo. zendo then keeps one folder for
each machine and each repository root on that machine, and shows only the notes of that folder. A
file in each folder records the machine, the path and the git origin of the repository.

```text
<machine>/<path of the repository root from home>/   the notes of that repository on that machine
<machine>/~/                                         the scratchpad, outside a repository
```

zendo commits the folder of the open repository at a set interval, 15 seconds by default, and
pushes in the background. Only one machine writes to each folder, so a conflict is rare.

Two tasks need judgement, and Claude does them, with edits allowed only in the notes repo and in the
listed notes: `zendo notes move` brings the notes of `docs/notes` into the notes repo, and
`zendo notes consolidate` merges the folders of one project from all machines and paths, found by
the git origin. The instructions for Claude are part of the zendo program.

The user turns the notes repo on with `zendo setup --notes-repo <folder or git URL>`, and off with
`zendo setup --notes-repo off`. `--notes DIR` overrides the notes repo for one run.

## Architecturally significant requirements

| ID         | Pattern  | Requirement                                                                                                                                        | Source           |
| ---------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| NOTESREP-1 | Optional | Where the user turned the notes repo on, the system shall keep the notes of a repository root in the folder of this machine and that root, and show only those notes. | User, 2026-11-11 |
| NOTESREP-2 | State    | While the notes repo is on, the system shall commit the changed notes at a set interval and push them, without a delay of the start or of the screen. | User, 2026-11-11 |
| NOTESREP-3 | Unwanted | If the pull of the notes repo has a conflict, then the system shall stop the sync, keep the files as they are, and show the problem.                   | Design           |
| NOTESREP-4 | Event    | When the user runs `zendo notes consolidate`, the system shall give Claude the notes of every folder with the same git origin, and allow edits only in the folder of this machine and in those notes. | User, 2026-11-11 |

### Acceptance tests

**`location_is_machine_then_path_from_home`** (unit test; verifies NOTESREP-1)

- Given a notes repo, the machine `mbp` and the home `/Users/m`.
- When zendo finds the folder of `/Users/m/src/zendo`, of `/Users/m`, and of `/opt/work`.
- Then the folders are `mbp/src/zendo`, `mbp/~` and `mbp/_/opt/work`.

**`syncer_commits_on_its_own_and_at_exit`** (unit test; verifies NOTESREP-2)

- Given a notes repo with a remote, and the sync on.
- When the user writes a note, waits for the interval, writes another note and closes zendo.
- Then the remote has both notes, in commits named after the machine and the folder.

**`a_conflict_stops_and_keeps_the_files`** (unit test; verifies NOTESREP-3)

- Given two clones of the notes repo that changed the same note.
- When the second clone pulls.
- Then the sync reports a conflict, the note keeps the text of that clone, and no rebase stays open.

**`consolidate_merges_the_folders_of_one_origin`** (mock agent; verifies NOTESREP-4)

- Given the folders of one project on two machines, with a note of the same name, and a folder of
  another project.
- When the user runs `zendo notes consolidate`.
- Then the note of this machine holds the text of both, the other folder of the project is gone, and
  the folder of the other project stays.

## Detailed requirements

| ID          | Pattern    | Requirement                                                                                                                         | Source           |
| ----------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| NOTESREP-5  | Ubiquitous | The system shall write in each folder the name of the machine, the path of the repository root, and its git origin.                 | Design           |
| NOTESREP-6  | Ubiquitous | The system shall use a name of the machine that does not change with the network, and keep it in the configuration.                 | Design           |
| NOTESREP-7  | Ubiquitous | The system shall commit and push every 15 seconds by default, and use the interval of the configuration when the user sets one.     | User, 2026-11-11 |
| NOTESREP-8  | Event      | When the user closes zendo, the system shall commit the changed notes and try to push them for a short time.                       | Design           |
| NOTESREP-9  | Unwanted   | If the network is not available, then the system shall keep the commits and push them at the next interval.                        | Design           |
| NOTESREP-10 | Event      | When the user turns the notes repo on in a repository with notes in `docs/notes`, the system shall let Claude move them into the notes repo. | User, 2026-11-11 |
| NOTESREP-11 | Ubiquitous | The system shall delete a listed note that Claude emptied, and only such a note.                                                    | Design           |
| NOTESREP-12 | Unwanted   | If the repository still has notes in `docs/notes` while the notes repo is on, then the system shall tell the user about `zendo notes move`. | Design      |
| NOTESREP-13 | Ubiquitous | `zendo doctor` shall check that the notes repo is a git repository, has a remote, and has no open rebase.                           | Design           |
| NOTESREP-14 | Unwanted   | If zendo runs outside a repository, then the system shall show in the file tree only the notes, and shall not walk the home folder. | Design           |

Tests that verify detailed requirements: `two_machines_sync_their_own_folders` (NOTESREP-5,
NOTESREP-9), `move_puts_the_notes_of_the_repo_into_the_folder_of_this_machine` (NOTESREP-10,
NOTESREP-11), `without_a_notes_repo_the_commands_say_how_to_turn_it_on`,
`a_new_empty_notes_repo_is_in_sync`.

See [ADR 0004](../architecture/decisions/0004-notes-in-a-private-git-repo.md).
