---
status: "draft"
date: "2026-11-11"
sources:
  - "../../zendo/zendo.md (ZENDO-7)"
  - "The request of the user: \"zendo need a local db state file\", 2026-11-11"
---

# 0003 Keep the state of zendo in the repository

## Context

zendo keeps state for each repository: the layout, the pane widths, the file tree, the Claude
threads, and for each file the visits, the delta base and the recap. The state must survive a
restart, must not go into git, and must not mix two repositories.

## Options

| Option                                              | Benefit                                                                    | Cost or constraint                                                     |
| --------------------------------------------------- | -------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| A JSON file in `.zendo/` in the repository, ignored by git | The state stays with the repository. Easy to read and to delete.     | Each clone has its own state.                                          |
| One file for each repository in the data folder of zendo | The repository stays clean.                                            | The state breaks when the repository moves.                            |
| A database such as SQLite                           | Fast for much data.                                                        | A dependency for a small amount of data; not easy to read.             |

## Decision

zendo keeps its state in `.zendo/db.json` in the root of the repository. The folder `.zendo` has its own
`.gitignore`, so git does not show it.

## Consequences

- A second clone of the same project starts with no state.
- The user can delete `.zendo` to reset zendo for the repository.

## Verification

`state_stays_in_the_repository` (not written yet), see [zendo](../../zendo/zendo.md).

## Decision record

Not reviewed by the user.
