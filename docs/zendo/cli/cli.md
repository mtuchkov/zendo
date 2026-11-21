---
status: "draft"
revision: "2"
date: "2026-11-14"
parent: "../zendo.md"
prev: "../project-structure.md"
next: "../site.md"
children:
  - "setup.md"
  - "init.md"
  - "doctor.md"
  - "keymap-wizard.md"
  - "upgrade.md"
level: "the commands around zendo and the job of each one: setup, init, doctor, upgrade"
note: "Draft. Revision 2 splits commands.md into this parent and one child for each command, from the requests of the user on 2026-11-14. The user approved CMD-17, CMD-18 and CMD-19 on 2026-11-15; the rest is not reviewed by the user, and its requirement IDs are proposals until the user reviews this document."
---

# Commands

The user installs zendo with one line, `curl … | sh`. The installer builds zendo, puts it on the
`PATH` and runs `zendo setup`. The steps for each case are in
[INSTALLATION.md](../../../INSTALLATION.md).

Each command has one job. zendo keeps state in two places: on the machine (the adapter, the config
of the user, the zendo section in the instructions of the user) and in each repository
(`project-structure.toml`, the zendo section in the instructions of the project, the zendo lines of
`.gitignore`).

| Command                              | Job                                                                 | Writes                                              |
| ------------------------------------ | ------------------------------------------------------------------- | --------------------------------------------------- |
| [`zendo setup`](setup.md)            | Prepares this machine. The installer runs it.                       | The state of the machine.                           |
| [`zendo init`](init.md)              | Prepares this repository. The user runs it one time in each repository. | The state of the repository.                    |
| [`zendo doctor`](doctor.md)          | Checks every step and reports.                                      | Nothing.                                            |
| [`zendo doctor --fix`](doctor.md)    | Repairs what the checks found, on the machine and in this repository. | Only what it reports.                             |
| [`zendo doctor --keys`](keymap-wizard.md) | Opens the wizard that moves the keys that the OS or the terminal takes. | The keymap file of the user, after a confirmation. |
| [`zendo upgrade`](upgrade.md)        | Builds the newest zendo, then runs `zendo doctor --fix` with it.    | The program, then what `zendo doctor --fix` repairs. |
| [`zendo --version`](upgrade.md)      | Shows the version and the commit.                                   | Nothing.                                            |

**Checks, in their order:** Node.js, the adapter, the repository and its notes folder,
`project-structure.toml`, the ACP handshake, the Claude Code login. `zendo setup`, `zendo doctor`
and `zendo doctor --fix` run the same checks. For each failed check, the command shows what failed
and the fix.

**Network.** `zendo setup` and `zendo doctor --fix` use the network only to install a missing
adapter. `zendo upgrade` and the update suggestion use GitHub; the update suggestion asks GitHub at
most once a day (CMD-20, ADR 0023). Nothing else uses the network.

## Architecturally significant requirements

| ID    | Pattern    | Requirement                                                                                                           | Source           |
| ----- | ---------- | --------------------------------------------------------------------------------------------------------------------- | ---------------- |
| CMD-1 | Unwanted   | If the pinned adapter is installed, then `zendo setup` shall not use the network.                                     | ZENDO-5          |
| CMD-2 | Unwanted   | If a check fails, then the command shall show what failed and the fix, and exit with an error.                        | User, 2026-11-11 |
| CMD-3 | Ubiquitous | The installer shall never wait for input.                                                                             | Design           |
| CMD-4 | State      | While zendo starts, the system shall look for a newer version in the background, and shall not make the start slower. | User, 2026-11-11 |
| CMD-17 | Unwanted  | If a line of `~/.zendo/config.toml` has an error, then zendo shall still read every other setting in the file. | User, 2026-11-15 |
| CMD-18 | Unwanted  | If a line of `~/.zendo/config.toml` has an error, then `zendo doctor` shall show the line number and the error. | User, 2026-11-15 |
| CMD-19 | Event     | When zendo saves a setting, zendo shall change only the line of that setting, or add one line when the setting is missing, and shall keep every other line as it is. | User, 2026-11-15 |
| CMD-20 | Event     | When the last check for a newer version is younger than 24 h, the system shall take its answer and shall not use the network; when GitHub did not answer that check, the system shall show no update until the next check. | User, 2026-11-18; ADR 0023 |

CMD-14, that `zendo doctor` writes no file, is in [Doctor](doctor.md).

### Acceptance tests

**`setup_needs_no_network_when_the_adapter_is_installed`** (unit test, not written yet; verifies
CMD-1). See [zendo](../zendo.md).

**`npm_errors_keep_the_reason_and_the_log`** (unit test; verifies CMD-2)

- Given an adapter install that fails with an npm error.
- When `zendo setup` reports it.
- Then the report shows the reason of npm and the path of its log.

**`installer_runs_without_a_terminal`** (terminal check, not written yet; verifies CMD-3)

- Given a machine without zendo, and SSH that needs a password.
- When the user pipes the installer into `sh`.
- Then the installer finishes, or fails with a fix, and asks nothing.

**`a_line_with_an_error_does_not_hide_the_others`** (unit test; verifies CMD-17, CMD-18)

- Given a `config.toml` with `width = "wide"` on line 2, a table header with a typo, and good
  settings before and after them.
- When zendo reads the file.
- Then zendo has every good setting, and the problems name line 2 and the line of the header.

**`a_saved_setting_changes_one_line`** (unit test; verifies CMD-19)

- Given a `config.toml` with comments, a `[notes]` table, and a line with an error.
- When the sound button saves `sound = false`, once with the line `sound = true` in the file and
  once without it.
- Then only that line changes, or one line comes before the first table; every other byte stays.

**`update_check_does_not_block_the_start`** (unit test, not written yet; verifies CMD-4)

- Given a network that does not answer.
- When zendo starts.
- Then the screen opens as fast as without the update check.

## Detailed requirements

The detailed requirements are in the child documents: CMD-5, CMD-6, CMD-12 and CMD-13 in
[Setup](setup.md), CMD-16 in [Init](init.md), CMD-7 and CMD-15 in [Doctor](doctor.md), and CMD-8 to
CMD-11 in [Upgrade](upgrade.md).

## Open points

| Open question                                                                                                              | Affects        | Answered by |
| -------------------------------------------------------------------------------------------------------------------------- | -------------- | ----------- |
| `zendo upgrade` repairs one repository at most. When zendo starts in a repository whose state is older than the program, should it repair the state, ask first, or only report it? | CMD-8, CMD-15 | User        |
