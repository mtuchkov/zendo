---
status: "draft"
revision: "1"
date: "2026-11-11"
parent: "zendo.md"
prev: "project-structure.md"
level: "the commands around zendo: install, setup, doctor, upgrade, version, update suggestion"
note: "Agent rewrite of the reference of zendo 0.9.1. Not reviewed by the user."
---

# Commands

The user installs zendo with one line, `curl … | sh`. The installer builds zendo,
puts it on the `PATH` and runs `zendo setup`. The steps for each case are in
[INSTALLATION.md](../../INSTALLATION.md).

| Command                         | What it does                                                                  |
| ------------------------------- | ----------------------------------------------------------------------------- |
| `zendo setup`                    | Installs the pinned Claude Code adapter once, then checks every step.         |
| `zendo doctor [--live] [--fix]`  | Checks every step. `--live` sends one short prompt. `--fix` repairs `project-structure.toml`. |
| `zendo upgrade [--force]`        | Builds the newest zendo, then runs `zendo setup` with it.                       |
| `zendo --version`                | Shows the version and the commit.                                             |

**Checks, in their order:** Node.js, the adapter, the repository and its notes folder,
`project-structure.toml`, the ACP handshake, the Claude Code login. For each failed check, the command shows
what failed and the fix.

**Network.** `zendo setup` uses the network only to install a missing adapter. `zendo upgrade` and the
update suggestion use GitHub. Nothing else uses the network.

**Certificates.** Company proxies often replace the TLS certificate. When the adapter install fails
with a certificate error, setup tries again with the certificate store of the system, and keeps that
setting when it works.

**Builds from a clone.** When the user runs the installer or `zendo upgrade` from a clone of zendo,
zendo builds that clone and not the main branch of GitHub.

## Architecturally significant requirements

| ID    | Pattern  | Requirement                                                                                                                          | Source           |
| ----- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ | ---------------- |
| CMD-1 | Unwanted | If the pinned adapter is installed, then `zendo setup` shall not use the network.                                                     | ZENDO-5           |
| CMD-2 | Unwanted | If a check fails, then the command shall show what failed and the fix, and exit with an error.                                      | User, 2026-11-11 |
| CMD-3 | Ubiquitous | The installer shall never wait for input.                                                                                          | Design           |
| CMD-4 | State    | While zendo starts, the system shall look for a newer version in the background, and shall not make the start slower.                 | User, 2026-11-11 |

### Acceptance tests

**`setup_needs_no_network_when_the_adapter_is_installed`** (unit test, not written yet; verifies
CMD-1). See [zendo](zendo.md).

**`npm_errors_keep_the_reason_and_the_log`** (unit test; verifies CMD-2)

- Given an adapter install that fails with an npm error.
- When `zendo setup` reports it.
- Then the report shows the reason of npm and the path of its log.

**`installer_runs_without_a_terminal`** (terminal check, not written yet; verifies CMD-3)

- Given a machine without zendo, and SSH that needs a password.
- When the user pipes the installer into `sh`.
- Then the installer finishes, or fails with a fix, and asks nothing.

**`update_check_does_not_block_the_start`** (unit test, not written yet; verifies CMD-4)

- Given a network that does not answer.
- When zendo starts.
- Then the screen opens as fast as without the update check.

## Detailed requirements

| ID     | Pattern    | Requirement                                                                                                                       | Source           |
| ------ | ---------- | --------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| CMD-5  | Event      | When `zendo setup` runs, the system shall install the pinned adapter in the data folder of zendo, if it is not there.             | User, 2026-11-11 |
| CMD-6  | Unwanted   | If the adapter install fails with a certificate error, then the system shall try again with the certificate store of the system, and keep that setting when it works. | User, 2026-11-11 |
| CMD-7  | Event      | When the user runs `zendo doctor --live`, the system shall send one short prompt to Claude and show the answer.                   | Design           |
| CMD-8  | Event      | When the user runs `zendo upgrade`, the system shall build the newest zendo and then run `zendo setup` with the new program.        | User, 2026-11-11 |
| CMD-9  | Optional   | Where the installer or `zendo upgrade` runs from a clone of zendo, the system shall build that clone.                              | User, 2026-11-11 |
| CMD-10 | Event      | When the user runs `zendo --version`, the system shall show the version and the commit of the build.                             | User, 2026-11-11 |
| CMD-11 | Event      | When a newer version exists, the system shall show a hint in the top bar, and `U` shall upgrade zendo and start it again.            | User, 2026-11-11 |
| CMD-12 | Unwanted   | If `zendo setup` runs outside a repository, then the system shall not report the repository checks as failures.                  | Design           |
| CMD-13 | Ubiquitous | The installer shall put zendo on the `PATH` and tell the user how to get the new `PATH` in the open shell.                        | Design           |

Tests that verify detailed requirements: `detects_the_npm_tls_error_from_the_field` (CMD-6),
`system_ca_is_appended_once` (CMD-6), `this_checkout_is_the_zendo_repo` (CMD-9),
`short_and_long_commits_match` (CMD-10).
