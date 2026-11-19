---
status: "draft"
date: "2026-11-19"
note: "The user chose the scope and the clipboard crate in chat on 2026-11-19. This record is an agent draft and is not reviewed by the user."
sources:
  - "../../zendo/io.md (IO-1 to IO-8)"
  - "The user, 2026-11-19: IO off the UI thread for all IO in one branch; 100 ms for a key and a frame; reads of the config and the state at the start; a slow-tick log in debug builds; arboard for the read and the copy of the clipboard."
  - "Measurements of 2026-11-19: fsync of 440 KB on /home took 8.5 ms at rest and up to 3.6 s during a build; the main thread of zendo waited in `Journal::beat` → `atomic::write` → `sync_all` for 1 s to 1.5 s about every 1.5 s; `wl-paste --list-types` never ended on GNOME Shell 49 (no data-control protocol)."
  - "ADR 0009 (git on a thread for the worktree changed view) and ADR 0017 (find on a worker thread), which this record extends to all IO."
---

# 0024 IO on worker threads

## Context

zendo did IO on the UI thread in many places: the heartbeat of the board with fsync each second,
the save of `.zendo/db.json`, the walk of the file tree, the clipboard programs, `/proc`, git. A
slow disk or a program that does not end froze the screen for seconds. ADR 0009 and ADR 0017 moved
two of them to threads, each in its own way. [Input and output](../../zendo/io.md) now asks that
all IO runs off the UI thread (IO-1) and that keys and frames take at most 100 ms (IO-2).

The decision needed: one mechanism for all IO, the clipboard library, the sync policy, and how to
find IO that a code check cannot see.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| One module `io` with a writer thread and a job runner; the tick takes the answers | One pattern; writes coalesce by path (IO-4); tests can hold a write (IO-2). | Each feature changes from a call to a request and an answer. |
| `tokio::task::spawn_blocking` for each IO call | No own threads. | Unit tests of the app run without a runtime; no coalescing of writes. |
| A thread of its own for each feature, as ADR 0009 and ADR 0017 did | Already works for git and find. | Many patterns; each feature solves limits, errors and stale answers again. |
| Keep IO on the UI thread and make it faster (no fsync, smaller files) | Small change. | A slow disk or a hung program still freezes the screen; breaks IO-1. |
| arboard for the clipboard | Measured on GNOME 49: reads through XWayland in 2–9 ms, a copy stays after zendo exits; native on macOS; no external programs. | X11 reads wait up to 4 s for each format; Wayland reads have no limit. Needs a worker and a limit anyway. |
| `wl-paste`, `xclip`, `wl-copy`, `osascript` with a time limit | No new crate. | `wl-paste` never ends on GNOME; one program for each desktop. |

## Decision

Proposal, not yet accepted:

- The module `worker` shall hold the means; the UI thread shall only send a request or take an
  answer, and one shared wake shall bring the main loop to the answers.
  - `Writer`: one thread; the newest content of each file waits (IO-4); sync only for user data
    (IO-3); a checked write writes a note only over the text that zendo saw last (UNDO-6).
  - `Task`: one operation with a time limit (IO-5); a new start drops the older answer.
  - `Memo`: facts that a frame shows (a branch, a file that exists, the text at HEAD), computed
    again while asked.
  - `Serial`: changes of files of the user, one at a time, in order.
- The disk work of the documents shall live in `docs::disk`, not in the modules of the UI thread;
  the file actions of the tree shall return an `Effect::Io` job (ADR 0008).
- What needs an answer shall wait for it ("wait, then act"): a jump to a match, a merge, the close
  of the float, a question after a save; after 350 ms the status bar says "reading" (IO-6).
- Programs (sessions, terminals) shall start on a task; their input waits until they run.
- The clipboard shall go through arboard, with `wayland-data-control`, on a task, limit 2 s.
- A debug build shall log each tick, key or frame over 100 ms with its slowest phase (IO-8).
- `ui_code_does_no_io` shall check the modules of the UI thread; the IO before the first frame and
  after the last one is named with its reason (IO-7).

## Consequences

- Each feature that reads gets a pending state, and each that writes gets a failure message.
- The answers of the agent link, the PTY readers, git (ADR 0009) and find (ADR 0017) keep their
  threads; they report through the same tick.
- `src/text/clipboard.rs` loses the programs; `arboard` joins the dependencies.
- A crash right after a write of an own file can lose the newest content of that file; the next
  write repairs it. User data keeps the sync.
- `tests/user_data_writes.rs` keeps its list `OWN`; the new check adds a list of the reads at the
  start.

## Verification

- `keys_answer_while_a_write_waits`, `paste_answers_while_the_clipboard_does_not`,
  `ui_code_does_no_io` and `a_slow_tick_is_logged_with_its_phase` in `cargo test`.
- The terminal check `keys_answer_during_a_full_build` of IO-2, with the sampling scripts of
  2026-11-19: no stall of the main thread of 100 ms or more.

## Decision record

Agent draft of 2026-11-19, from the choices of the user in chat. Not reviewed.
