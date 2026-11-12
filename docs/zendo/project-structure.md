---
status: "draft"
revision: "1"
date: "2026-11-11"
parent: "zendo.md"
prev: "chat.md"
next: "commands.md"
level: "how zendo uses project-structure.toml, and how zendo doctor repairs it"
note: "Agent rewrite of the reference of zendo 0.9.1. Not reviewed by the user."
---

# Project structure

`project-structure.toml` sits in the root of a repository. It is optional. It describes the folders
and the key files for three readers: people read `about`, zendo shows the `main` entries as the
structure view of the file tree, and Claude gets the `claude` rules. The format is in
[resources/project-structure.md](../../resources/project-structure.md), which is part of the zendo
program.

`zendo doctor` checks the file. When zendo does not understand it, zendo first repairs what needs no
judgement, for example other key names and paths that start with `./`. Then it gives the rest to
Claude, with the instructions in
[resources/fix-project-structure.md](../../resources/fix-project-structure.md).

## Architecturally significant requirements

| ID        | Pattern  | Requirement                                                                                                                                         | Source           |
| --------- | -------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| STRUCT-1  | Event    | When the user asks Claude about a file, the system shall send the project rules and the rules of each entry that covers the file, from the most general to the most specific. | User, 2026-11-11 |
| STRUCT-2  | Unwanted | If zendo does not understand the file, then `zendo doctor` shall repair it by itself first and give only the rest to Claude.                          | User, 2026-11-11 |
| STRUCT-3  | State    | While Claude repairs the file, the system shall allow edits of `project-structure.toml` only, and reject each other edit and each command.          | Design           |

### Acceptance tests

**`parses_and_finds_rules_from_general_to_specific`** (unit test; verifies STRUCT-1)

- Given a file with project rules, a rule for `docs` and a rule for `docs/design`.
- When zendo collects the rules for `docs/design/a.md`.
- Then the project rules come first, then `docs`, then `docs/design`.

**`zendo_fixes_key_names_and_paths_by_itself`** (mock agent; verifies STRUCT-2)

- Given a file with `description` in place of `about` and a path that starts with `./`.
- When the user runs `zendo doctor --fix`.
- Then the file uses `about` and the clean path, keeps its comments, and Claude gets no prompt.

**`claude_fixes_the_rest_and_may_change_only_that_file`** (mock agent; verifies STRUCT-2, STRUCT-3)

- Given a file with a problem that needs judgement.
- When the user runs `zendo doctor --fix`, and the mock agent tries to edit `README.md` and then the
  file.
- Then the edit of `README.md` is rejected, the edit of the file is applied, and zendo checks the file again.

## Detailed requirements

| ID        | Pattern    | Requirement                                                                                                                       | Source           |
| --------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| STRUCT-4  | Event      | When `project-structure.toml` changes, the system shall read it again without a restart.                                         | Design           |
| STRUCT-5  | Ubiquitous | `zendo doctor` shall check that the file parses, has no unknown keys, has clean and unique paths, and has `main` paths that exist and have no `*`. | Design |
| STRUCT-6  | Unwanted   | If zendo does not understand the file and `zendo doctor` runs on a terminal, then it shall ask the user to repair the file.        | User, 2026-11-11 |
| STRUCT-7  | Event      | When the user runs `zendo doctor --fix`, the system shall repair the file without a question.                                     | User, 2026-11-11 |
| STRUCT-8  | Unwanted   | If `zendo doctor` runs without `--fix` and without a terminal, then the system shall not change the file.                         | Design           |
| STRUCT-9  | Ubiquitous | The system shall keep the comments and the layout of the file when it repairs the file by itself.                                | Design           |
| STRUCT-10 | Event      | Before a repair, the system shall keep a copy of the file. After it, the system shall show the diff and check the file again.     | Design           |
| STRUCT-11 | Optional   | Where the repository has no `project-structure.toml` but has `project-structure.md`, `zendo doctor --fix` shall make the TOML file from it with Claude. | User, 2026-11-11 |
| STRUCT-12 | Ubiquitous | The system shall keep the instructions for Claude for the repair in the zendo program, not in the repository of the user.          | User, 2026-11-11 |
| STRUCT-13 | Optional   | Where an entry names a `template`, the system shall use it for new documents in that folder, and `zendo doctor` shall report a template that is not there. | User, 2026-11-12 |
| STRUCT-14 | Unwanted   | If two entries have the same `label`, not counting case, then `zendo doctor` shall report the second entry.                       | User, 2026-11-12 |

Tests that verify detailed requirements: `checks_find_what_zendo_does_not_understand` (STRUCT-5),
`without_fix_and_without_a_terminal_nothing_changes` (STRUCT-8),
`quick_fix_renames_keys_and_paths_and_keeps_comments` (STRUCT-9),
`a_missing_file_is_created_from_project_structure_md` (STRUCT-11), `a_folder_names_its_template`,
`a_missing_template_is_an_issue` (STRUCT-13), `a_label_in_two_entries_is_an_issue` (STRUCT-14).
