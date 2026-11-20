---
status: "approved"
approved: "2026-11-19"
revision: "1"
date: "2026-11-14"
parent: "cli.md"
next: "init.md"
level: "what the installer and zendo setup do on the machine"
note: "Agent draft, split from commands.md on 2026-11-14. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. The flags: `--claude` of doctor is `--agent`, and `--agent CMD` is `--agent-cmd CMD`, on the request of the user on 2026-11-18 (approved by the user on 2026-11-18). CMD-21 (2026-11-19, from the fix of an empty XDG_DATA_HOME) is a proposal of the agent, approved by the user on 2026-11-19."
---

# Setup

`zendo setup` prepares the machine. The installer runs it, and the user can run it again at any
time: a second run changes nothing. See [Commands](cli.md) for the other commands.

**What it writes.** The pinned Claude Code adapter in the data folder of zendo, when it is not
there. The config of the user, `~/.zendo/config.toml`: the agent and the notes repo
(`--agent-cmd`, `--notes-repo`, see [Claude](../claude.md) and [Notes repo](../notes-repo.md)). The
zendo section in the instructions of the user (SESSIONS-22, see [zendo session](../zendo-session.md)). It
writes nothing in the repository.

**Then it checks.** After the writes, setup runs the checks of [Doctor](doctor.md). Outside a
repository it does not report the repository checks as failures.

**Then the fonts.** When the list of recommended fonts changed since the last time, setup names
`zendo setup fonts` at its end (FONT-18, see [Font tip](../fonts-tip.md)).

**Then the keys.** At its end, setup offers the [keymap wizard](keymap-wizard.md) to a person in a
terminal. When the installer or an agent runs setup, it asks nothing and ends with an instruction
for the agent to ask the user to run `zendo doctor --keys` (WIZARD-15, WIZARD-16).

**Certificates.** Company proxies often replace the TLS certificate. When the adapter install fails
with a certificate error, setup tries again with the certificate store of the system, and keeps that
setting when it works.

**The installer.** `install.sh` is POSIX `sh` and runs piped, so it never waits for input (CMD-3).
It builds zendo, puts it on the `PATH`, and runs `zendo setup`.

## Detailed requirements

| ID     | Pattern    | Requirement                                                                                                                                                           | Source           |
| ------ | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| CMD-5  | Event      | When `zendo setup` runs, the system shall install the pinned adapter in the data folder of zendo, if it is not there.                                                 | User, 2026-11-11 |
| CMD-6  | Unwanted   | If the adapter install fails with a certificate error, then the system shall try again with the certificate store of the system, and keep that setting when it works. | User, 2026-11-11 |
| CMD-12 | Unwanted   | If `zendo setup` runs outside a repository, then the system shall not report the repository checks as failures.                                                       | Design           |
| CMD-13 | Ubiquitous | The installer shall put zendo on the `PATH` and tell the user how to get the new `PATH` in the open shell.                                                            | Design           |
| CMD-21 | Unwanted   | If `XDG_CONFIG_HOME`, `XDG_DATA_HOME` or `XDG_CACHE_HOME` is empty or is not an absolute path, then the system shall use the default folder in the home folder of the user in its place. | Design; XDG Base Directory Specification |

Tests that verify detailed requirements: `detects_the_npm_tls_error_from_the_field` (CMD-6),
`system_ca_is_appended_once` (CMD-6), `an_empty_or_relative_xdg_folder_counts_as_not_set` (CMD-21).
