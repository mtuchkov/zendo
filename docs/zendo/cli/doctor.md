---
status: "approved"
approved: "2026-11-18"
revision: "1"
date: "2026-11-14"
parent: "cli.md"
prev: "init.md"
next: "keymap-wizard.md"
level: "what zendo doctor checks, and what its flags --fix, --agent and --live add"
note: "Agent draft from the requests of the user on 2026-11-14, split from commands.md. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. The flags: `--claude` of doctor is `--agent`, and `--agent CMD` is `--agent-cmd CMD`, on the request of the user on 2026-11-18 (approved by the user on 2026-11-18)."
---

# Doctor

`zendo doctor` checks every step and reports. It is safe to run at any time and in any folder: it
writes no file. See [Commands](cli.md) for the checks and the other commands.

**What it reports.** Each check with its result, and for each failed check what failed and the fix
(CMD-2). When the state of the machine or of the repository is older than zendo, doctor says so and
names the command that repairs it: `zendo doctor --fix`.

**`--fix`.** `zendo doctor --fix` repairs what the checks found. It does the writes of
[Setup](setup.md) on the machine and, in a repository, the writes of [Init](init.md), and it also
repairs by itself what it can in a `project-structure.toml` that zendo does not understand
(STRUCT-2). Each repair has one result for the same input: `--fix` does not call the agent and asks
no question. It reports each file that it changes.

**`--agent`.** `zendo doctor --fix --agent` also gives to the agent what `--fix` cannot repair by
itself in `project-structure.toml` (STRUCT-7, STRUCT-11, see
[Project structure](../project-structure.md)).

It also repairs a broken font recipe in an interactive session of the agent (FONT-25, see
[Font recipe](../fonts-recipe.md)).

**`--live`.** `zendo doctor --live` also sends one short prompt to Claude and shows the answer.

## Architecturally significant requirements

| ID     | Pattern    | Requirement                                                       | Source           |
| ------ | ---------- | ----------------------------------------------------------------- | ---------------- |
| CMD-14 | Ubiquitous | `zendo doctor` without `--fix` shall not create or change a file. | User, 2026-11-14 |

### Acceptance tests

**`doctor_writes_no_file`** (integration test with the mock agent; verifies CMD-14)

- Given a repository without `project-structure.toml`, without the zendo sections and without the
  zendo lines of `.gitignore`, and a home folder without the zendo section of the user.
- When the user runs `zendo doctor`.
- Then no file of the repository and no file of the home folder changes, and the output names
  `zendo doctor --fix`.

## Detailed requirements

| ID     | Pattern | Requirement                                                                                                                                                                                  | Source           |
| ------ | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| CMD-7  | Event   | When the user runs `zendo doctor --live`, the system shall send one short prompt to Claude and show the answer.                                                                              | Design           |
| CMD-15 | Event   | When the user runs `zendo doctor --fix`, the system shall do each repair that needs no agent and no answer of the user, on the machine and in the repository, and report each file that it changes. | User, 2026-11-14 |

Tests that verify detailed requirements: `doctor_fix_repairs_the_repository_and_the_machine` and
`fix_without_claude_does_not_call_the_agent` (CMD-15), in `tests/doctor_fix.rs`.
