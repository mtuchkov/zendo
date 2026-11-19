---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
level: "what zendo does when a dependency fails, for every feature"
note: "Agent draft of the failure default that AGENTS.md asks for, from the defects that reached main from 2026-11-12 to 2026-11-17 and the behavior of zendo 0.113. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Failure default

Sources: the rules "Failure requirements" in `AGENTS.md` (2026-11-18); the fixes `8e36ffd` (a config
with a typo was read as the defaults, and the next save wrote the defaults over it), `03cf764` (a
new note went to another checkout) and `402bfe2` (the "offline" status stayed after the network came
back); the code of zendo 0.113.

Parent: [zendo](zendo.md).

## Terms

- A **dependency** is a file, a program or a service that zendo reads or calls: the config, the
  keymap, `project-structure.toml`, a note, git, the notes repo, the agent, the API, the shell.
- A dependency **fails** when it is missing, is not valid, changes during use, or gives no answer.
- **User data** is a file that the user can edit: a note, the config, the keymap, a file of the
  repository.

## The default

zendo stays up when a dependency fails. It shows the failure once, says what fixes it, and does not
change user data because of the failure. When the dependency works again, the failure goes from the
screen. A write of user data is complete or does not happen: the file never stays half written.

A specification states a failure requirement only where its feature does something different from
this default. Examples of such differences today: when Claude does not answer at the start, zendo
does not open the screen (ZENDO-2); a line of the config with an error costs only its own setting
(CMD-17).

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| FAIL-1 | Unwanted   | If a dependency fails, then the system shall keep working, shall show the failure once, and shall not change user data because of the failure. | Design |
| FAIL-2 | Unwanted   | If a write of user data is not completed, then the file shall keep the content that it had before the write, and the system shall show the failure. | Design |
| FAIL-3 | Event      | When the system writes user data, the system shall change only the content that the action of the user names. | Design |

### Acceptance tests

**`a_structure_that_does_not_parse_leaves_its_reason`** (unit test; verifies FAIL-1)

- Given a test repository with a `project-structure.toml` that does not parse.
- When zendo reads the file again.
- Then zendo keeps working, the flash line says that the file does not parse and names
  `zendo doctor --fix`, and the file is not changed.

**`a_failed_write_keeps_the_file`** (unit test, not written yet; verifies FAIL-2)

- Given a config, a keymap and a document with front matter, and a write that stops after half of
  the new content.
- When zendo saves a setting, a key, and the front matter.
- Then each file has the content that it had before the save, and the flash line names the file.

**`a_saved_setting_changes_one_line`** (unit test; verifies FAIL-3)

- Given a config with comments and a line with an error.
- When zendo saves one setting.
- Then only the line of that setting changes.

## Detailed requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| FAIL-4 | Event      | When a dependency that failed works again, the system shall remove its failure from the screen. | Design |
| FAIL-5 | Unwanted   | If a dependency that holds user data is not valid, then the system shall not write values that it did not read from the dependency in place of its content. | Design |
| FAIL-6 | Ubiquitous | The message of a failure shall name the dependency and, where one exists, the command or the key that fixes it. | Design |

Tests that verify detailed requirements: `offline_clears_when_the_network_comes_back` (FAIL-4),
`a_saved_setting_changes_one_line` (FAIL-5).

## Where zendo does not meet the default yet

FAIL-2: these writes of user data replace the file in place, so a write that stops can leave the
file half written. The note save (`docs/notes.rs`) and the font install already write a temporary
file and rename it.

- the config: `setup/config/mod.rs`;
- the keymap: `setup/config/keys_write.rs`;
- the title and the front matter that the agent puts into a document: `agent/work/title.rs`,
  `agent/work/meta.rs`;
- the fixes of `zendo doctor --fix`: `setup/doctor/fix.rs`.
