---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "zendo.md"
level: "the list of the documents of the specification, with the topic of each"
note: "Split from zendo.md on 2026-11-21, on the request of the user, to keep each document within two pages. It holds no requirement."
---

# Documents of the specification

Parent: [zendo](zendo.md). Each document of this list is a child of [zendo](zendo.md) and states one
part of zendo.

| Document                                     | Topic                                                         |
| -------------------------------------------- | ------------------------------------------------------------- |
| [Panes and layouts](panes/panes.md)          | The panes, the two layouts, resizing, hiding; text selection; keys, hints and help |
| [Notes](notes.md)                            | The note pane: read mode and write mode                       |
| [Write mode](write.md)                       | The live view, hard wrap, the cursor between modes            |
| [Say it here](say.md)                        | `ctrl-shift-s`: the agent writes an idea at a place in the note |
| [Notes list and search](notes-list.md)       | Titles, tags, links, the notes list, search                   |
| [Note to spec](note-to-spec.md)              | `[ → spec ]` or `s`: the agent turns a note into specifications |
| [New note in a float](float.md)              | `alt-n`: a new note in a float over the open file, without distraction |
| [Title of a new note](float-title.md)        | The name model writes the title of the note in the float      |
| [Find](find.md)                              | `alt-/`: recent files, search by path and text, the preview   |
| [Find: modes](find-modes.md)                 | Notes, Docs, Code and All modes; `tab`; the `/` key           |
| [Find: tags and IDs](find-ids.md)            | `#`: tags and requirement IDs in docs and code                |
| [Find: background](find-background.md)       | search off the UI thread with the ripgrep crates; All merge   |
| [Find: regex](find-regex.md)                 | `[.*]` and `alt-r` in the find panel and the tree filter      |
| [Notes repo](notes-repo.md)                  | Notes in a private git repo, in sync between machines         |
| [Length](length.md)                          | The word limit, the red area, the split with Claude            |
| [File tree](files/files.md)                  | The files of the repository and their views                   |
| [Work, delta and recaps](work.md)            | Visits that count, the delta pane, recaps                     |
| [Claude](claude.md)                          | The connection to Claude, edits                               |
| [Questions of the agent](questions.md)       | Which tool calls zendo allows, the quick action in the chat, a command in a terminal |
| [Threads](threads.md)                        | The five agent sessions, their keys and buttons, session IDs and resume |
| [Sessions](sessions.md)                      | Claude Code sessions in the background, more than one at a time |
| [Hand-off during a turn](handoff.md)         | `ctrl-o` while the chat answers: wait, or stop and continue    |
| [Claude Code messages in the chat](cli-messages.md) | The prompts and answers of a running Claude Code session in its chat |
| [Activity of a Claude Code session](cli-activity.md) | The status line, steps, checklist and questions of a Claude Code session in its chat, from its hooks |
| [Codex in its terminal](codex-cli.md) | `ctrl-o` with Codex: `codex resume` in a terminal of zendo, its state from the terminal |
| [Back to the agent](back-to-the-agent.md) | A quiet Claude Code session that the user left goes back to the chat agent through ACP |
| [zendo session](zendo-session.md)            | What a zendo session is, its id, and what zendo tells its agent |
| [Message board](board.md)                    | The emergency channel between zendo sessions: channels, questions, tasks, limits |
| [Posts](board-posts.md)                      | Posts about shared resources to all sessions of the machine   |
| [Leases](board-leases.md)                    | The lease of a post, its age, the restart, and the tombstone of a dead post |
| [Watch the board](board-watch.md)            | `zendo board watch`: the messages of a session during its turn, for the Monitor of Claude Code |
| [Chat](chat.md)                              | The chat input: chips, attachments, `@` file suggestions        |
| [Chips and the open document](chat-chips.md) | The chip window, images, the choice of the open document      |
| [Chips in a question](chat-messages.md)      | How chips go to the agent and to Claude Code, and show in the transcript |
| [Chat history](chat-history.md)              | The messages of the user per zendo session, the common history, `up` and `down` |
| [Activity of the agent](activity.md)         | The status line, steps, questions and commands of a turn in the chat |
| [Checklist](checklist.md)                    | The plan of the agent at the top of the transcript, closed and open |
| [Transcript](transcript.md)                  | The time of each message; the question of the turn at the top of the transcript |
| [A status line that is alive](live-status.md) | The thought, the waits of the agent, and the pulse in the status line |
| [Interactivity of the chat](interactivity.md) | Steer a turn, open a step, the next question, the mode |
| [Background processes](background.md)        | That background processes of the agent still run, which ones, their end, their stop |
| [Terminal](terminal/terminal.md)             | Shell blocks in the chat, the `F5` and `[>]` terminals in the chat pane, their output for the agent |
| [Attention](attention.md)                    | The bell and the blinking button when a turn ends; `[sound]`   |
| [Project structure](project-structure.md)    | `project-structure.toml`: use and repair                      |
| [Commands](cli/cli.md)                       | Install, `zendo setup`, `zendo init`, `zendo doctor`, `zendo upgrade` |
| [Landing page](site.md)                      | The page in `site/`: the pixel garden, the demo, the lofi music  |
| [Lofi music](lofi.md)                        | Calm music that zendo makes itself: `[music]` and `alt-m`        |
| [Usage counts](usage.md)                     | The actions of the user, by pane and trigger, for the choice of key hints |
| [Tips](tips.md)                              | The tip menu at start and the tip in the status bar              |
| [Failure default](failures.md)               | What zendo does when a dependency fails, for every feature       |
| [Responsiveness of actions](responsiveness.md) | The time that the user waits for the result of each action, measured in one place |
| [Telemetry log](telemetry.md)                | The events of the actions in rolling files in the data folder, for a later analytics service |
| [Internals of the TUI](internals/internals.md) | The core and the runtime of the TUI: one specification for each chapter of the zendo book |
