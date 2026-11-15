---
status: "draft"
revision: "1"
date: "2026-11-14"
parent: "cli.md"
prev: "doctor.md"
level: "what zendo upgrade builds and repairs, the version, and the update suggestion"
note: "Agent draft from the requests of the user on 2026-11-14, split from commands.md. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Upgrade

`zendo upgrade` brings zendo and its state to the newest version. It builds the newest zendo, and
then runs `zendo doctor --fix` with the new program (see [Doctor](doctor.md)). So the new program
repairs the state of the machine, and the state of the repository when the user runs the upgrade in
one. See [Commands](cli.md) for the other commands.

**One repository at most.** The state of the machine is there one time, and the upgrade repairs
it. Each repository has its own state. In the other repositories of the user, the state stays as
it is until `zendo doctor --fix` runs there. The open point in [Commands](cli.md) is about this.

**No agent call.** `zendo doctor --fix` does not call the agent and asks no question, so an
upgrade does not need a Claude login and does not stop for an answer.

**Builds from a clone.** When the user runs the installer or `zendo upgrade` from a clone of zendo,
zendo builds that clone and not the main branch of GitHub.

**Version and update suggestion.** `zendo --version` shows the version and the commit. While zendo
starts, it looks for a newer version in the background (CMD-4). When there is one, the top bar
shows a hint, and `U` upgrades zendo and starts it again.

## Detailed requirements

| ID     | Pattern  | Requirement                                                                                                                              | Source                             |
| ------ | -------- | ---------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| CMD-8  | Event    | When the user runs `zendo upgrade`, the system shall build the newest zendo and then run `zendo doctor --fix` with the new program.     | User, 2026-11-11; User, 2026-11-14 |
| CMD-9  | Optional | Where the installer or `zendo upgrade` runs from a clone of zendo, the system shall build that clone.                                    | User, 2026-11-11                   |
| CMD-10 | Event    | When the user runs `zendo --version`, the system shall show the version and the commit of the build.                                     | User, 2026-11-11                   |
| CMD-11 | Event    | When a newer version exists, the system shall show a hint in the top bar, and `U` shall upgrade zendo and start it again.                | User, 2026-11-11                   |

Tests that verify detailed requirements: `an_upgrade_ends_with_doctor_fix` (CMD-8),
`this_checkout_is_the_zendo_repo` (CMD-9),
`short_and_long_commits_match` (CMD-10).

Revision 1 changed CMD-8: before, the upgrade ran `zendo setup` with the new program.
