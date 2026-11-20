---
status: "approved"
approved: "2026-11-20"
revision: "1"
date: "2026-11-19"
parent: "zendo.md"
prev: "claude.md"
next: "agent-profiles.md"
level: "the environment that zendo gives to the agents it starts: its sources, their order, `.envrc`, the worktrees"
note: "Agent draft from the decisions of the user on 2026-11-19 in chat. Approved by the user on 2026-11-20."
---

# Agent environment

An agent finds its account and its settings through environment variables: Claude Code through
`CLAUDE_CONFIG_DIR`, Codex through `CODEX_HOME`. zendo must start every agent with the account that
the user chose, also when the agent works in a worktree. The choice of an account is a profile:
see [Agent profiles](agent-profiles.md).

## The agent environment

zendo starts these agent processes:

- the agent of the chat ([Claude](claude.md));
- each Claude Code or Codex session in a terminal of zendo ([Sessions](sessions.md));
- each call of the name model ([Session names](session-names/session-names.md));
- the login check and the agent checks of `zendo doctor` and `zendo setup`.

zendo builds one agent environment when it starts, and gives it to each of these processes. The
agent environment has three sources, from the lowest priority to the highest:

1. The environment of the zendo process: `CLAUDE_CONFIG_DIR=… zendo` works.
2. The `.envrc` file at the root of the main checkout of the repository.
3. The selected profile ([Agent profiles](agent-profiles.md)).

A variable of a higher source replaces the same variable of a lower one.

**Worktrees.** zendo reads `.envrc` only from the main checkout, not from the folder of the agent.
A worktree often has no `.envrc`, because the file is usually ignored by git. The commands that the
agent runs in a worktree get the variables because the agent process passes its environment on.
A hook of the shell of the user that loads `.envrc` does not take part: it does not run in the
commands of an agent.

**`.envrc` is read, not run.** Design (User, 2026-11-19): zendo reads `.envrc` itself and uses no
other program for it. It takes only the lines `export NAME=value` and `NAME=value`; a value can be
in single or double quotes, and zendo takes it as it is, without the expansion of `$` or `~`. zendo
skips empty lines and comments. It skips any other line and shows it once. So a cloned repository
cannot run code when zendo opens it.

**System CA.** When the npm registry works only with the CA certificates of the system, `zendo
setup` saves `system_ca = true` in `~/.zendo/config.toml`. This is a fact of the machine, not of a
profile, so it applies with every profile and without one.

## Architecturally significant requirements

| ID    | Pattern    | Requirement | Source |
| ----- | ---------- | ----------- | ------ |
| ENV-1 | Ubiquitous | The system shall start the agent of the chat, each agent session in a terminal, each call of the name model, and each agent check of `zendo doctor` and `zendo setup` with the agent environment. | User, 2026-11-19 |
| ENV-2 | Ubiquitous | The agent environment shall be the environment of the zendo process, with the variables of the `.envrc` of the main checkout over it, and the variables of the selected profile over both. | User, 2026-11-19 |
| ENV-3 | Ubiquitous | The system shall read `.envrc` from the root of the main checkout of the repository, also when the agent works in a worktree and when git ignores `.envrc`. | User, 2026-11-19 |
| ENV-4 | Ubiquitous | The system shall take from `.envrc` only the lines `export NAME=value` and `NAME=value`, and shall not run the file or any other program to read it. | User, 2026-11-19 |

### Acceptance tests

**`agent_env_reaches_every_agent`** (mock agent; verifies ENV-1, ENV-2, ENV-3)

- Given a repository with a worktree in `.worktrees/1`, a `.envrc` in the main checkout that git
  ignores, with `CLAUDE_CONFIG_DIR=A` and `FROM_ENVRC=1`;
  the profile `work` with `CLAUDE_CONFIG_DIR=B`; `FROM_START=1` in the environment of zendo; and
  fake agent programs that write their environment to a file.
- When zendo starts with `--profile work`, and starts the chat agent, a session in the worktree, a
  call of the name model, and the login check.
- Then each file has `CLAUDE_CONFIG_DIR=B`, `FROM_ENVRC=1` and `FROM_START=1`. Without a
  profile, each file has `CLAUDE_CONFIG_DIR=A`.

**`envrc_is_read_not_run`** (unit test; verifies ENV-4, ENV-7)

- Given a `.envrc` with `export A=1`, `B='two words'`, `C="$HOME"`, a comment, and the line
  `touch marker`.
- When zendo builds the agent environment.
- Then the environment has `A=1`, `B=two words` and `C=$HOME` as written, the file `marker` does
  not exist, and zendo shows once that line 5 of `.envrc` was skipped.

## Detailed requirements

| ID    | Pattern    | Requirement | Source |
| ----- | ---------- | ----------- | ------ |
| ENV-5 | Optional   | Where `~/.zendo/config.toml` has `system_ca = true`, the system shall add `--use-system-ca` to `NODE_OPTIONS` of the agent environment with every profile and without one, and to the environment of npm. | Design |
| ENV-6 | Ubiquitous | The system shall keep the agent environment while zendo runs: a change of `.envrc` or of a profile shall apply after the next start of zendo. | Design |
| ENV-7 | Unwanted   | If a line of `.envrc` is not empty, not a comment, and not a variable line, then the system shall skip the line and shall show the file and the line number once. | Design |
