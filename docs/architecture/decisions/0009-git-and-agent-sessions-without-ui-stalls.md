---
status: "draft"
date: "2026-11-17"
sources:
  - "Issue #5: the UI freezes on a switch of the agent session, and `claude` processes pile up."
  - "The user, 2026-11-17: move the git of the tree off the UI thread, show a git error, and capture the decisions as an ADR, not as new requirement identifiers."
  - "../../zendo/files/files-worktree.md (FILES-27, FILES-32)"
  - "ADR 0001 (Claude through ACP)"
---

# 0009 Git and agent sessions do not stall the UI

## Context

Issue #5 reported two faults that a switch of the agent session showed:

- `git()` in `src/setup/sync/git.rs` waited for git to end before it read the output of git. An
  output larger than the pipe buffer (64 KiB on Linux) blocked git, so each call ran to its limit
  of 5 s. The worktree changed view (FILES-27) ran `git diff` and `git ls-files` on the UI thread
  on each switch of the session (FILES-32) and every 5 s, so the UI froze.
- zendo opened a new ACP session when the user forgot a session or chose another model, and never
  closed the old one. Each open session kept its `claude` process.

In the same views, a failure of git, for example a repository without `main`, showed as "No changes
against main.", which is wrong.

No specification covers the pipes of git or the life of an ACP session, and the agent shall not
create requirement identifiers. This record holds the decisions; the tests name it.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Read the pipes of git on threads while git runs | No deadlock for any output size; keeps the limit. | Two short threads per call. |
| Write the output of git to temporary files | No threads. | Files to clean; disk writes on each scan. |
| Run git for the worktree changed view on a thread; the tick takes the answer | The UI never waits for git. | The view shows the rows a short time later. |
| Keep git on the UI thread, now that it is fast | No change. | A slow disk or a large repository still freezes the UI. |
| Close a replaced or forgotten ACP session, when the agent can | The agent ends the process of the session. | Only agents that advertise close; one more round trip. |
| Leave sessions open until zendo exits | No change. | Processes pile up during a long run. |

## Decision

Proposal, not yet accepted:

- `git()` shall read the standard output and the standard error of git while git runs, and shall
  keep the time limit of the call.
- The file tree shall run git for the worktree changed view on a thread of its own, on a switch of
  the agent session and on each periodic scan. Until git answers, the view shall keep the rows of
  the same worktree, or say "Reading the changes against main…" when it has none.
- If git fails, then the worktree changed view shall show "Cannot read the changes against main:"
  and the error of git, not an empty list. A branch without a commit in common with `main` is such
  an error.
- When the user forgets an ACP session or a new model replaces it, zendo shall send
  `session/close` for the old session, where the agent advertises `session_capabilities.close`.
  zendo shall wait for the close before it handles the next command of the agent link, so that a
  session that opens again with the same key loads after the close.

## Consequences

- `worktree::changes` returns the error of git. `Tree` holds the pending answer and the error;
  `Tree::poll`, called from `App::tick`, takes the answer. A scan of another view or of another
  worktree drops the pending answer, so a late answer does not fill the wrong view. A reveal of a
  file while git reads, and the first rows of a worktree, select the row of the file when the
  answer comes.
- The worktree view (every text file) stays on the UI thread: it reads the disk, not git.
- The recap of a worktree still runs `worktree::diff` on the UI thread, at most every 10 minutes.
  A later change can move it to a thread too.
- An agent without close keeps the old behavior: its sessions stay open until zendo exits.

## Verification

- `a_large_output_does_not_hang_git` in `src/setup/sync/mod_tests.rs`: about 460 KB of output return
  before the limit.
- `the_changed_view_reads_git_off_the_ui_thread` and `a_git_error_shows_in_the_changed_view` in
  `tests/worktree.rs`: a switch of the session returns before git answers, and an error of git
  shows in the view.
- `the_changed_view_selects_the_open_file_when_git_answers` and
  `a_reveal_while_git_reads_selects_the_file_in_the_new_rows` in `tests/worktree.rs`: the selection
  goes to the open file when git answers.
- `replaced_and_forgotten_sessions_are_closed` in `tests/acp_mock.rs`: the mock agent gets
  `session/close` for a replaced and for a forgotten session.

## Decision record

Draft by the agent, 2026-11-17. It waits for the review of the user.
