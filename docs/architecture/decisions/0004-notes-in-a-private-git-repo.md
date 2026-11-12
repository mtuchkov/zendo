---
status: "draft"
date: "2026-11-11"
sources:
  - "../../zendo/notes-repo.md"
  - "The decisions of the user on 2026-11-11: a private notes repo; one folder for each machine and repo root; consolidation by origin with Claude; a commit every 15 seconds, configurable; Claude moves the old notes."
---

# 0004 Keep the notes in a private git repo, one folder for each machine and repo root

## Context

Notes in `docs/notes` go into the repository of the project, with its reviewers and its history.
Some notes are private, or belong to a person and not to a project. The user wants all notes in
one private repository, in sync between machines, and wants to see only the notes of the current
machine and repository root.

## Options

| Option                                                         | Benefit                                                                                          | Cost or constraint                                                                   |
| -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------ |
| One git repo, one folder for each machine and repo root        | Only one machine writes to a folder, so a conflict is rare. git gives history and works offline. | The notes of one project are in more than one folder; consolidation needs a step.   |
| One git repo, one folder for each project (by git origin)      | One place for the notes of a project.                                                            | Two machines write the same files: conflicts. The user did not want this view.      |
| A file sync service, such as a cloud drive                     | No git steps.                                                                                    | No history, no review, another account; conflicts make copies of files.             |

## Decision

Proposal: the notes repo is a git repository of the user. Each folder is `<machine>/<path of the
repo root from home>`, with a location file that holds the machine, the path and the git origin.
zendo commits and pushes in the background at a set interval. Claude moves old notes into the
notes repo and consolidates the folders of one origin, with edits allowed only in the notes repo.

The sync shall never delay the start of zendo, and shall stop on a conflict without a change of the
files.

## Consequences

- A note is in the notes repo within one interval and one push; at exit zendo pushes once more.
- Consolidation is a step that the user starts. It changes the folders of other machines; the next
  pull on those machines brings the change.
- A machine that renames itself gets a new folder. zendo keeps the machine name in the
  configuration so that a network change does not rename it.

## Verification

The tests in `src/sync.rs` with a bare remote and two clones, and `tests/notes_repo.rs` with the
mock agent.

## Decision record

The user decided the design on 2026-11-11. This record is not reviewed by the user.
