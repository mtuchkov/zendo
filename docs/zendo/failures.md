---
status: "draft"
revision: "2"
date: "2026-11-21"
parent: "zendo.md"
level: "what zendo does when a dependency fails, for every feature"
note: "Draft of the failure default that AGENTS.md asks for, from the defects that reached main from 2026-11-12 to 2026-11-17 and the behavior of zendo 0.113. Updated with the atomic write of zendo 0.113.6. Revision 1 was approved by the user on 2026-11-18. Revision 2, draft from the decision of the user of 2026-11-21 that the zendo book takes priority: a write of user data is in place, with a backup (ADR 0029); FAIL-2 changed, FAIL-7 is new, the section Design follows the book. Revision 2 is not reviewed by the user. The code of zendo 0.126 implements revision 1: a temporary file and a rename."
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
screen. A write of user data that fails leaves the file with the content that it had: zendo restores
the file from the backup of the write. If zendo itself stops during a write, the backup stays beside
the file and keeps that content.

A specification states a failure requirement only where its feature does something different from
this default. Examples of such differences today: when Claude does not answer at the start, zendo
does not open the screen (ZENDO-2); a line of the config with an error costs only its own setting
(CMD-17).

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| FAIL-1 | Unwanted   | If a dependency fails, then the system shall keep working, shall show the failure once, and shall not change user data because of the failure. | Design |
| FAIL-2 | Unwanted   | If a write of user data fails, then the file shall keep the content that it had before the write, and the system shall show the failure. | Design; User, 2026-11-21 |
| FAIL-3 | Event      | When the system writes user data, the system shall change only the content that the action of the user names. | Design |

### Acceptance tests

**`a_structure_that_does_not_parse_leaves_its_reason`** (unit test; verifies FAIL-1)

- Given a test repository with a `project-structure.toml` that does not parse.
- When zendo reads the file again.
- Then zendo keeps working, the flash line says that the file does not parse and names
  `zendo doctor --fix`, and the file is not changed.

**`a_failed_write_keeps_the_file`** (unit test; verifies FAIL-2)

- Given a config file, and a write that stops after half of the new content with an error.
- When zendo writes the file.
- Then the error comes back, the file has the content that it had before, and no other file
  stays in the folder.

**`user_data_goes_through_the_atomic_write`** (unit test; verifies FAIL-2)

- Given the source code of zendo.
- When the test looks for the calls that write a file.
- Then each such call is in the one function that writes user data, or in a file that writes a
  file of zendo itself (logs, caches, the state in `.zendo`), with the reason.

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
| FAIL-7 | Unwanted   | If zendo stops during a write of user data, then the backup of the write shall stay beside the file, with the content that the file had before the write. | User, 2026-11-21 |

Tests that verify detailed requirements: `offline_clears_when_the_network_comes_back` (FAIL-4),
`a_saved_setting_changes_one_line` (FAIL-5); `a_link_stays_a_link`,
`the_file_keeps_its_permissions` and `a_saved_setting_keeps_a_linked_config_a_link` (FAIL-2,
FAIL-3).

## Design

Every write of user data goes through one function, `zendo::atomic` (Design). It copies the file to
a backup, `name~`, writes the new content in place, and then deletes the backup. After a failure, it
restores the file from the backup. The file keeps its identity: a file that is a link stays a link,
and the owner and the permissions stay. The rules are in
[File writes](internals/file-writes/file-writes.md) (WRITE-5, WRITE-6, WRITE-8), and the decision is
in [ADR 0029](../architecture/decisions/0029-user-data-written-in-place-with-a-backup.md).
