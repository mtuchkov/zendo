---
status: "draft"
revision: "1"
date: "2026-11-14"
parent: "cli.md"
prev: "setup.md"
next: "doctor.md"
level: "what zendo init writes in a repository"
note: "Agent draft from the requests of the user on 2026-11-14. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. The code does not have this command yet: zendo doctor does these writes."
---

# Init

`zendo init` prepares the repository where the user runs it. The user runs it one time in each
repository, and again when [Doctor](doctor.md) reports that the state of the repository is older
than zendo. A second run changes nothing. See [Commands](cli.md) for the other commands.

**What it writes.** All three are files of the repository, so the user sees them in `git status`
and commits them:

- `project-structure.toml`, from the zendo template, when the repository has none (STRUCT-16).
- The zendo section about `project-structure.toml` in `CLAUDE.md` and `AGENTS.md`, between
  `<!-- zendo -->` and `<!-- /zendo -->` (STRUCT-17).
- The zendo lines of `.gitignore`, so that git ignores `.zendo/` (STRUCT-18).

[Project structure](../project-structure.md) has these requirements. Init tells the user each
change that it makes. It writes nothing on the machine and does not call the agent.

## Detailed requirements

| ID     | Pattern | Requirement                                                                                                                                                              | Source           |
| ------ | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------- |
| CMD-16 | Event   | When the user runs `zendo init` in a repository, the system shall bring the state of the repository to the version of zendo (STRUCT-16 to STRUCT-18) and tell the user each change. | User, 2026-11-14 |

## Open points

| Open question                                                              | Affects | Answered by |
| -------------------------------------------------------------------------- | ------- | ----------- |
| What shall `zendo init` do outside a repository: an error, or nothing?     | CMD-16  | User        |
