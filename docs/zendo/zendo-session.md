---
status: "draft"
revision: "1"
date: "2026-11-14"
parent: "zendo.md"
prev: "back-to-the-agent.md"
next: "chat.md"
level: "what a zendo session is, its id, and what zendo tells the agent of a session"
note: "Agent draft, split from sessions.md on 2026-11-14 to keep it within two pages. The definition of the zendo session is new text. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# zendo session

A *zendo session* is one of the five agent sessions of a repository ([Threads](threads.md)),
with the Claude Code session that zendo runs for it ([Sessions](sessions.md)): one chat thread and
at most one Claude Code process. Its *id* is the number of the agent session, 1 to 5. The chat and
the Claude Code session of one agent session have the same id.

**Why the agent must know it.** Several zendo sessions can work in one repository at the same time.
Each one changes code only in its own git worktree, `.worktrees/<id>`, so that the sessions do not
change the files of each other.

**The MCP server.** zendo starts the chat and the Claude Code session of each zendo session with the
MCP server `zendo`. The instructions of the server name the id and carry the rules of the worktree,
in XML tags, so the agent has them at the start without a tool call. The tool `session` returns the
same id. The text of the instructions is part of the zendo program. Only sessions that zendo runs
get it.

**The instructions of the user.** [`zendo setup`](cli/setup.md) and
[`zendo doctor --fix`](cli/doctor.md) write a short zendo section into the instructions of the user
for each agent: a session that zendo runs follows the instructions of the MCP server `zendo`. The
section is the one exception to SESSIONS-18. It has a condition, so it changes nothing for a session
that zendo does not run. zendo makes no folder for an agent that the user does not have.

**The instructions of the project** have no rules for zendo sessions. Their zendo section is about
`project-structure.toml` only ([Project structure](project-structure.md)).

## Detailed requirements

| ID          | Pattern    | Requirement | Source |
| ----------- | ---------- | ----------- | ------ |
| SESSIONS-21 | Ubiquitous | The system shall start each Claude Code session and the chat of each agent session with the MCP server `zendo` of its session, whose instructions name the id of that zendo session and the rules of its worktree, and whose tool `session` returns the same id. | User, 2026-11-13 |
| SESSIONS-22 | Event | When `zendo setup` or `zendo doctor --fix` runs, the system shall write a section into both files of instructions of the user, `CLAUDE.md` of Claude Code and `AGENTS.md` of Codex, between `<!-- zendo -->` and `<!-- /zendo -->`, that tells a session that zendo runs to follow the instructions of the MCP server `zendo`, and replace only that section. | User, 2026-11-14 |

Tests of SESSIONS-21: `tests/mcp.rs` and `src/mcp/mod_tests.rs`. Tests of SESSIONS-22:
`doctor_writes_the_zendo_section_of_the_user` and
`doctor_makes_the_file_but_no_folder_for_the_user_section` in `src/setup/doctor/project_tests.rs`.

The identifiers keep the topic `SESSIONS`: they came from [Sessions](sessions.md), revisions 6 to 9.

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| The id is the number of the agent session, not of the thread. Two zendo processes in one repository both have a session 1, and a new thread in an agent session gets the worktree of the old one. Should the id belong to the thread? | SESSIONS-21 | User |
