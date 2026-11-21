---
status: "draft"
date: "2026-11-18"
note: "The user chose the decision in chat on 2026-11-18. This record is a draft and is not reviewed by the user."
sources:
  - "The user, 2026-11-18: a list of what zendo does that an endpoint security product counts as an attack, with a cleanup for each; light model only through `claude -p`; the update check once a day; a native macOS path, built but not tested on a Mac."
  - "ADR 0018: CrowdStrike Falcon killed zendo and its zsh on 2026-11-18."
  - "ADR 0010 (the light model through the API) and ADR 0016 (the background processes from the process tree), which this record changes in part."
  - "../../zendo/session-names/session-names-api.md (NAMES-15, NAMES-16, NAMES-20, NAMES-22)"
  - "../../zendo/background.md (BG-10, BG-13, BG-14)"
  - "../../zendo/cli/cli.md (CMD-4, CMD-20)"
---

# 0023 zendo does nothing that looks like an attack

## Context

CrowdStrike Falcon killed zendo two times on 2026-11-18 (ADR 0018). An endpoint security product
judges a program by what it does, not by why. Besides the startup files of the shells, zendo did
five more things that such a product counts as an attack or as reconnaissance:

| What zendo did | How it looks |
| -------------- | ------------ |
| Ran `ant auth print-credentials --access-token`, took the token, and sent it in its own HTTPS requests to `api.anthropic.com`; read `ANTHROPIC_API_KEY` and `OPENAI_API_KEY` and sent them to the address of `ANTHROPIC_BASE_URL` | Theft of the credentials of another program |
| Fell back to `ps -A` on macOS always, and on Linux without the `children` files | A list of every process of the computer |
| Read the command lines of all processes under zendo each second, also when the agent started nothing | A steady scan of processes |
| Sent signal 0 to each known process group each second | Probes of other processes |
| Ran `git ls-remote` on GitHub at each start, with `StrictHostKeyChecking=accept-new` for ssh | Network at start, and a new host key accepted without the user |

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Keep the behavior, ask the users to allow zendo in their product | No change of the code | The user cannot change the policy of the company; zendo dies |
| Make each feature opt-in | Old behavior for those who want it | Two paths to keep and test; the defaults lose the features |
| Do each thing another way, which a product does not count as an attack | One path; the features stay | A light model only for Claude; on Linux without `children` files, no background line; a macOS path not run on a Mac yet |

## Decision

The user chose the third option on 2026-11-18.

- **The light model.** zendo asks only through `claude -p`, with the login of the user (NAMES-15,
  NAMES-21). It reads no credential of another program, uses no API key of the environment, and
  sends no request of its own to a model API (NAMES-22). `ant` is not used (NAMES-20 retired). Codex and
  other agents ask in a throwaway session (NAMES-16).
- **No `ps`.** On Linux zendo reads its tree only from `/proc`; a kernel without the `children`
  files gives no background line. On macOS zendo reads its tree from libproc:
  `proc_listchildpids`, `proc_pidinfo` and `sysctl` `KERN_PROCARGS2` (BG-14).
- **Scans only around commands.** zendo reads its tree while a command step runs, for 10 s after the
  start of the last command step, and while the background line shows a process (BG-13).
- **No probe signals.** A group runs while its leader runs: `/proc/<pgid>` on Linux,
  `proc_listpgrppids` on macOS (BG-10). zendo sends a signal only when the user stops a group, at
  the quit, and by force 5 s after a stop (BG-3, BG-4, BG-9).
- **The update check once a day.** zendo asks GitHub at most once in 24 h and keeps the answer, also
  when GitHub did not answer (CMD-20). ssh accepts no new host key.

## Consequences

- ADR 0010 holds only for the way through `claude -p`. A user without `claude` gets the names from
  a throwaway session of the agent.
- ADR 0016 holds without its `ps` fallback and without signal 0. A group whose leader ended while
  other processes of it left the tree of zendo counts as ended.
- A background process that a command step starts after a pause of more than 10 s, while the line
  is empty, shows only at the next command step.
- The macOS path is built for `aarch64-apple-darwin`, but not run on a Mac yet.
- The update suggestion can come up to a day late.

## Verification

- `zendo_reads_no_credential_of_another_program` (NAMES-22) and the tests of `claude -p` in
  `src/agent/api/`, and the mock `claude` of `tests/common/claude.rs` for the names, the titles and
  the key hints.
- `the_tree_is_read_only_around_commands` (BG-13), `a_group_runs_while_its_leader_runs` (BG-10,
  BG-14), and the tests of `tests/background.rs`.
- `the_update_check_asks_github_once_a_day` (CMD-20) and `ssh_accepts_no_new_host_key`.
- `cargo check --target aarch64-apple-darwin` and `cargo clippy --target aarch64-apple-darwin`.
- On the machine of the user, auditd shows no SIGKILL from Falcon to zendo.
