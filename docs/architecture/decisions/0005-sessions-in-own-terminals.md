---
status: "draft"
date: "2026-11-12"
sources:
  - "../../zendo/sessions.md"
  - "The request of the user for more than one Claude Code session at a time, without a herdr dependency, 2026-11-12"
  - "herdr (Apache-2.0), read for its design: https://github.com/ogulcancelik/herdr"
---

# 0005 Run Claude Code sessions in terminals that zendo owns

## Context

Today `ctrl-o` gives the terminal to `claude --resume <id>` and waits until it exits
(`src/tui/mod.rs`). The user can go back to zendo only when Claude Code exits, so the session stops.
The user wants sessions that keep running in the background, one for each of the five agent
sessions (`F6` to `F10`), with their states visible in zendo (SESSIONS-1 to SESSIONS-5). zendo must not depend on another program for
this (SESSIONS-3).

herdr does this with a server process that owns one pseudo-terminal (PTY) for each agent, a terminal
emulator for each PTY, and agent states from Claude Code hooks and from screen patterns.

## Options

| Option                                                   | Benefit                                                                                   | Cost or constraint                                                                                         |
| -------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| A PTY and a terminal emulator for each session, in zendo | One program. zendo knows each state and draws each screen. Works in each terminal.        | New crates for the PTY and the emulator. Raw input and resize to get right. Sessions end when zendo quits. |
| The same, behind a server process, as herdr does          | Sessions survive the quit of zendo and a lost SSH connection.                             | A second process, a socket protocol and its versions. Much more code. The conversation survives anyway.   |
| Open the session in tmux or herdr                         | Very little code.                                                                         | A dependency that the user does not want (SESSIONS-3). No states in zendo.                                 |
| Keep the hand-off and add nothing                         | No work.                                                                                  | No background sessions.                                                                                     |

## Decision

Proposal: zendo runs each session in a PTY that it opens itself, and feeds the output of the PTY to a
terminal emulator in memory. The emulator keeps the screen of the session while the user is away;
the session view draws that screen. No server process: a session lives while zendo runs, and
`claude --resume` keeps the conversation.

For the states, zendo starts Claude Code with hooks in a settings argument of that run only. The
hooks of the events "prompt sent", "tool use", "permission request", "notification" and "stop"
write the state to a file of that session in `.zendo/`, and zendo reads it on each tick. Claude Code merges these hooks with the settings of the user, so zendo
does not change the settings files of the user (SESSIONS-18). Where no hook reports, zendo uses the
output of the PTY (SESSIONS-17).

The session module shall own the PTYs, the emulators and the states; the rest of zendo shall see
only commands (open, leave, write input, resize, stop) and events (state, screen changed, exited),
as the agent module does for ACP.

## Consequences

- New dependencies: `portable-pty` (MIT) for the PTY, and `vt100` (MIT) for the terminal emulator.
  zendo draws the screen of `vt100` into the ratatui buffer itself, so no widget crate pins the
  version of ratatui.
- `handoff()` in `src/tui/mod.rs` goes away; the input thread no longer pauses.
- A thread has one writer at a time: the ACP connection or the Claude Code session (SESSIONS-4).
  After a Claude Code session ends, zendo opens the agent session again (SESSIONS-13).
- The hooks depend on the hook events of Claude Code; a change of them is a change of zendo, with a
  check by `zendo doctor`.
- Sessions stop when zendo quits (SESSIONS-6, SESSIONS-19).

## Verification

The acceptance tests of [Sessions](../../zendo/sessions.md) with the mock command line, in
`cargo test` (`tests/sessions.rs`, with `zendo-mock-cli`); a manual check with real Claude Code at
80 × 24 and 200 × 50.

## Decision record

Not reviewed by the user.
