---
status: "draft"
date: "2026-11-18"
note: "The user chose the decision in chat on 2026-11-18. This record is a draft and is not reviewed by the user."
sources:
  - "The user, 2026-11-18: end a command when the terminal is free and quiet for 300 ms; load no script into the shell."
  - "auditd on the machine of the user, 2026-11-18 15:41:29 and 21:27:29: falcon-sensor-bpf sent SIGKILL to zendo and to the zsh that zendo had just started."
  - "herdr ee5bfb5: starts the shell of a pane as it is, and reads the foreground process group from /proc."
  - "../../zendo/terminal/terminal-output.md (TERM-26, TERM-27, TERM-34, TERM-55)"
---

# 0018 The shells of the terminals start as they are

## Context

zendo loaded a script for the OSC 133 marks into each shell that it started (TERM-34): for zsh it
wrote `.zshenv` and `.zshrc` into `.zendo/shell/zsh` and set `ZDOTDIR` to that folder; for bash it
used `--rcfile`; for the POSIX shells it set `ENV`. The script read the files of the user and added
the marks. The marks gave the exact limits of the output and the exit code of each command.

On 2026-11-18 CrowdStrike Falcon killed zendo two times. Each time it sent SIGKILL to zendo and to
the zsh that zendo had just started, in the same millisecond. A program that sends a shell to
startup files in a hidden folder looks like a change of the shell configuration (MITRE ATT&CK
T1546.004). herdr, a terminal multiplexer that runs on the same machine, starts each shell as it is.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Keep the script | Exact limits and the exit code in every shell. | Falcon kills zendo; the user cannot work. |
| The script only when the user turns it on | Exact limits for users without an endpoint security product. | Two paths to keep and test; the default still gives no exit code. |
| No script: the terminal is free and quiet for 300 ms | No change to the start of the shell; the same path for every shell. | No exit code; a loop of the shell that prints nothing and starts no program can end too early. |

## Decision

The user chose the third option on 2026-11-18.

- zendo shall start the shell of a terminal as it is: no startup file of zendo, and no change to
  `ZDOTDIR`, `ENV` or `BASH_ENV` (TERM-55).
- zendo shall end a command when no program of the command runs in the terminal and the terminal
  was quiet for 300 ms, and shall show the exit code as unknown (TERM-27).
- The rule of the free quiet terminal shall hold also while the shell sends OSC 133 marks. The
  end mark ends a command earlier and gives its exit code (TERM-26); a new prompt ends a command
  whose end mark got lost (TERM-74); zendo shall not show the marks (TERM-35). So marks that are not
  complete, as `vte.sh` sends them under a prompt of the user, do not hold a command forever.
- zendo shall start the shell without `VTE_VERSION`: its terminal is not a VTE terminal (TERM-76).

## Consequences

- TERM-34 is retired. `src/session/marks.rs` is removed, and zendo writes no file to
  `.zendo/shell`.
- The result of a command shows the exit code as unknown, unless the user has a shell integration
  that sends the marks.
- A loop of the shell that prints nothing for more than 300 ms and starts no program ends early.
- The parser of the output without marks must handle the prompt of each shell. fish ends its prompt
  with a carriage return, so the last line of the output is not always the prompt.

## Verification

- `bash_starts_as_it_is`, `zsh_starts_as_it_is` and `fish_starts_as_it_is` check that the shell
  reads only the files of the user, sees no `ZDOTDIR` and no `ENV`, and that zendo writes no file of
  a shell.
- `alt_o_puts_the_result_into_the_input_and_does_not_send_it` checks the marks of a `.bashrc` of the
  user.
- On the machine of the user, auditd shows no SIGKILL from Falcon to zendo when terminals open.
