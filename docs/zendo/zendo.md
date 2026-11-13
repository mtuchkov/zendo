---
status: "draft"
revision: "1"
date: "2026-11-11"
children:
  - "panes.md"
  - "notes.md"
  - "write.md"
  - "say.md"
  - "notes-list.md"
  - "notes-repo.md"
  - "length.md"
  - "files.md"
  - "work.md"
  - "claude.md"
  - "sessions.md"
  - "handoff.md"
  - "cli-messages.md"
  - "chat.md"
  - "terminal/terminal.md"
  - "attention.md"
  - "project-structure.md"
  - "commands.md"
level: "what zendo is, its parts, and the qualities that hold for all parts"
note: "Agent rewrite of the reference of zendo 0.9.1. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# zendo

Sources: the reference of zendo 0.9.1 (`docs/REFERENCE.md` at commit `6170f8d`), the decisions of the
user on 2026-11-11, and the code of zendo 0.9.3.

| Document                                     | Topic                                                         |
| -------------------------------------------- | ------------------------------------------------------------- |
| [Panes and layouts](panes.md)                | The panes, the two layouts, resizing, hiding, mouse selection |
| [Notes](notes.md)                            | The note pane: read mode and write mode                       |
| [Write mode](write.md)                       | The live view, hard wrap, the cursor between modes            |
| [Say it here](say.md)                        | `ctrl-shift-s`: the agent writes an idea at a place in the note |
| [Notes list and search](notes-list.md)       | Titles, tags, links, the notes list, search                   |
| [Notes repo](notes-repo.md)                  | Notes in a private git repo, in sync between machines         |
| [Length](length.md)                          | The word limit, the red area, the split with Claude            |
| [File tree](files.md)                        | The files of the repository and their views                   |
| [Work, delta and recaps](work.md)            | Visits that count, the delta pane, recaps                     |
| [Claude](claude.md)                          | The connection to Claude, threads, edits                      |
| [Sessions](sessions.md)                      | Claude Code sessions in the background, more than one at a time |
| [Hand-off during a turn](handoff.md)         | `ctrl-o` while the chat answers: wait, or stop and continue    |
| [Claude Code messages in the chat](cli-messages.md) | The prompts and answers of a running Claude Code session in its chat |
| [Chat](chat.md)                              | The chat input: chips, attachments, `@` file suggestions        |
| [Terminal](terminal/terminal.md)             | Shell blocks in the chat, the `F5` and `[>]` terminals in the chat pane, their output for the agent |
| [Attention](attention.md)                    | The bell and the blinking button when a turn ends; `[sound]`   |
| [Project structure](project-structure.md)    | `project-structure.toml`: use and repair                      |
| [Commands](commands.md)                      | Install, `zendo setup`, `zendo doctor`, `zendo upgrade`          |

## Scope

In scope: reading, writing and finding the markdown files of one git repository in a terminal;
work with Claude on the open file; the installation, the checks and the upgrades of zendo.

Out of scope, not planned yet: theme files of the user (zendo has four themes), fuzzy search of
titles, git commits from zendo, mouse clicks that move the cursor of the editor.

## How zendo works

The user starts `zendo` in a repository. zendo starts the Claude Code adapter as an Agent Client
Protocol (ACP) agent and checks that Claude answers before it opens the screen. The screen shows the
file tree, the open file, the chat with Claude and the delta of the file. The user reads and writes
markdown files of the repository. When the user asks, Claude gets the file, the selection and the
rules of the project. Claude edits the files without a question to the user.

zendo counts the visits where the user really worked on a file. It shows what changed since the last
such visit, and asks Claude for a short recap of each changed file while the user does not use zendo.
zendo keeps its state in the repository, in a folder that git ignores.

## Main design choices

- zendo is a Rust program with ratatui. See [ADR 0002](../architecture/decisions/0002-rust-and-ratatui.md).
- zendo talks to Claude through ACP and the Claude Code adapter, with the Claude Code login of the
  user. See [ADR 0001](../architecture/decisions/0001-claude-through-acp.md).
- zendo keeps its state of a repository in that repository. See
  [ADR 0003](../architecture/decisions/0003-state-in-the-repository.md).
- The files of the user are plain markdown. zendo adds no format of its own to them.

## Shared inputs of the acceptance tests

- **Mock agent**: `zendo-mock-agent`, an ACP agent with scripted answers. It proposes edits, requests
  shell commands and answers recap and repair prompts.
- **Test repository**: a temporary folder that each test makes, with `docs/`, a `.gitignore` and,
  where the test needs it, `project-structure.toml`.
- **Terminal sizes** for terminal checks: 80 × 24 and 200 × 50.

## Architecturally significant requirements

| ID     | Pattern    | Requirement                                                                                                                        | Source           |
| ------ | ---------- | ---------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| ZENDO-1 | Ubiquitous | The system shall keep the notes of the user as markdown files in the repository of the user, or in the private notes repo of the user. | User, 2026-11-11 |
| ZENDO-2 | Unwanted   | If Claude does not answer at the start, then the system shall not open the screen, and shall show the failed step and its fix.     | User, 2026-11-11 |
| ZENDO-3 | Optional   | Where the user starts zendo with `--offline`, the system shall open without Claude.                                                 | Design           |
| ZENDO-5 | Unwanted   | If every part that zendo needs is installed, then `zendo setup`, `zendo doctor` and the start of zendo shall not use the network.      | User, 2026-11-11 |
| ZENDO-6 | Optional   | Where a repository has `project-structure.toml`, the system shall use it in that repository.                                       | User, 2026-11-11 |
| ZENDO-7 | Ubiquitous | The system shall keep its state of a repository in that repository, in a folder that git ignores.                                  | User, 2026-11-11 |

### Acceptance tests

**`new_note_is_a_markdown_file_in_the_notes_folder`** (unit test, not written yet; verifies ZENDO-1)

- Given a test repository without a notes folder.
- When the user saves a new note.
- Then the notes folder holds one markdown file with the text of the note, and nothing else changed.

**`start_check_fails_without_claude`** (mock agent, not written yet; verifies ZENDO-2)

- Given an agent command that exits at once.
- When the user starts zendo.
- Then zendo does not open the screen, prints the failed step and its fix, and exits with an error.

**`offline_start_needs_no_agent`** (terminal check, not written yet; verifies ZENDO-3)

- Given no agent.
- When the user starts `zendo --offline`.
- Then the screen opens, and a question to Claude shows that Claude is off in this run.

**`setup_needs_no_network_when_the_adapter_is_installed`** (unit test, not written yet; verifies ZENDO-5)

- Given the pinned adapter in the data folder of zendo, and no network.
- When the user runs `zendo setup`.
- Then every check passes.

**`structure_view_lists_main_materials_in_file_order`** (unit test; verifies ZENDO-6)

- Given a test repository with `project-structure.toml` and main materials.
- When zendo opens the file tree.
- Then the tree shows the main materials first, in the order of the file.

**`state_stays_in_the_repository`** (unit test, not written yet; verifies ZENDO-7)

- Given a test repository.
- When the user changes the layout and closes zendo.
- Then the state is in the `.zendo` folder of the repository, and git does not show that folder.

Retired in revision 1: ZENDO-4 (only the user, an allowed edit, or the title step changes a file), because Claude and the terminal of zendo also change files (User, 2026-11-12). Retired identifiers are never reused.

The specification is complete when the acceptance tests of every architecturally significant
requirement in this document and its children pass.

## Milestones

No milestone document exists yet. zendo 0.9.3 implements this specification, except the behavior that
the tests marked "not written yet" verify: that behavior exists, but no test checks it.
