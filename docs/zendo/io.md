---
status: "draft"
revision: "1"
date: "2026-11-19"
parent: "zendo.md"
prev: "failures.md"
level: "where zendo does input and output, for every feature: off the UI thread"
note: "Agent draft from the request of the user on 2026-11-19, after the UI froze: the heartbeat of the board ran fsync on the UI thread during builds (up to 3.6 s), and a paste waited for wl-paste, which never ended on GNOME Wayland. Answers of the user on 2026-11-19: 100 ms, the reads at the start, the slow-tick log. Not reviewed by the user; the requirement IDs are proposals."
---

# Input and output

zendo answers each key at once, also when the disk is slow, a program does not answer, or the
network is gone. The **UI thread** is the thread that reads the keys and the mouse and draws the
screen.

## What is IO

IO (input and output) is each operation whose time depends on something outside zendo:

| Kind | Operations | Examples in zendo |
| ---- | ---------- | ----------------- |
| Files | Open, read, write, sync, rename or remove a file; list a folder; read the metadata of a file | The open note, `.zendo/db.json`, the heartbeat and the journal of the board, the walk of the file tree |
| Programs | Start a program, wait for it, read its output | git, `wl-paste`, `xclip`, the command line of the agent that names the sessions |
| System state | Read the files of the system about processes | `/proc` for the background processes |
| Network | Connect, send, receive | None directly today: zendo reaches the network through programs |
| Waits | Wait for a lock or a channel that another thread holds while it does IO | A lock that a worker holds during a write |

A computation on data in memory is not IO.

**Where IO runs.** The UI thread gives each IO operation to a worker thread and takes the answer on
a later tick, until then the screen shows the last known state or says that zendo reads. A failure
follows the [failure default](failures.md). ADR
[0024](../architecture/decisions/0024-io-on-worker-threads.md) holds the mechanism for all IO,
after [0009](../architecture/decisions/0009-git-and-agent-sessions-without-ui-stalls.md) and
[0017](../architecture/decisions/0017-find-search-on-a-worker-thread.md) for git and find. One
exception: at the start, before the
first frame, zendo reads its config and its state on the UI thread, when no screen can freeze yet.

**Writes.** A write of user data stays whole or does not happen (FAIL-2), and only user data waits
for the disk to sync. The own files of zendo, such as the heartbeat of the board and the local
state, do not. When zendo has newer content for a file before the last write ended, only the
newest content waits.

## Architecturally significant requirements

| ID   | Pattern    | Requirement                                                                                                                                     | Source           |
| ---- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| IO-1 | Ubiquitous | The system shall do IO off the UI thread.                                                                                                       | User, 2026-11-19 |
| IO-2 | State      | While an IO operation waits for the disk, a program or the network, the system shall handle each key of the user and draw the screen within 100 ms. | User, 2026-11-19 |

### Acceptance tests

**`keys_answer_while_a_write_waits`** (unit test; verifies IO-1, IO-2)

- Given an app whose writes wait until the test lets them go.
- When the heartbeat of the board is due, and the user presses keys.
- Then each tick and each key returns within 100 ms. After the test lets the writes go, the file
  holds the newest content.

**`paste_answers_while_the_clipboard_does_not`** (mock clipboard program; verifies IO-1, IO-2)

- Given a clipboard program that never ends.
- When the user presses the paste key in the chat.
- Then the key returns within 100 ms, and after the time limit the chat says that the clipboard
  does not answer.

**`ui_code_does_no_io`** (unit test; verifies IO-1)

- Given the source of the modules that run on the UI thread.
- When the check looks for calls of file, program and network IO.
- Then it finds none outside the workers, except the reads before the first frame (IO-7), which
  the check lists with a reason.

**`keys_answer_during_a_full_build`** (terminal check; verifies IO-2)

- Given zendo in a terminal, and `cargo clean -p zendo && cargo nextest run` in another worktree on
  the same disk.
- When a script samples the main thread of zendo each 50 ms during the build and the tests.
- Then the main thread never stays in IO for 100 ms or more.

## Detailed requirements

| ID   | Pattern    | Requirement                                                                                                                       | Source           |
| ---- | ---------- | --------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| IO-3 | Ubiquitous | The system shall sync to the disk only the writes of user data.                                                                   | Design; FAIL-2   |
| IO-4 | Event      | When the system has newer content for a file whose write did not end, the system shall write only the newest content after that write. | Design   |
| IO-5 | Unwanted   | If a read of the clipboard or a program that the system starts does not end within its time limit, then the system shall stop waiting for it and show the failure once. | Design; FAIL-1, FAIL-6 |
| IO-6 | State      | While the answer of an IO operation has not come, the system shall show the last known state of what it reads, or say that it reads. | Design; ADR 0009 |
| IO-7 | Event      | When the system starts, the system shall read its config and its state on the UI thread before it draws the first frame. | User, 2026-11-19 |
| IO-8 | Optional   | Where the system is a debug build, the system shall log each tick longer than 100 ms with the phase of the tick that took the time. | User, 2026-11-19 |

Tests of the detailed requirements: `keys_answer_while_a_write_waits` (IO-4),
`paste_answers_while_the_clipboard_does_not` (IO-5), `ui_code_does_no_io` (IO-7),
`a_slow_tick_is_logged_with_its_phase` (IO-8).
