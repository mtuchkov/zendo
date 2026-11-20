---
status: "implemented"
approved: "2026-11-20"
revision: "3"
date: "2026-11-20"
parent: "zendo.md"
prev: "claude.md"
next: "envrc.md"
level: "the environment that zendo gives to the agents it starts: its sources, their order, the worktrees"
note: "Agent draft from the decisions of the user on 2026-11-19 in chat. Approved by the user on 2026-11-20. Revision 2: the shell of a terminal gets the agent environment too (the user, 2026-11-20). Revision 3, after the code review of 2026-11-20: the reading of `.envrc` (ENV-4, ENV-7) moved to envrc.md; zendo reads `.envrc` only in a trusted folder, and outside a repository too (ENV-10; the user, 2026-11-20); the shell without the system CA in ENV-5 is a fix of the agent. The user approved revision 3 on 2026-11-20 in chat."
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
2. The `.envrc` file at the root of the main checkout of the repository; outside a repository,
   the `.envrc` of the folder where zendo starts.
3. The selected profile ([Agent profiles](agent-profiles.md)).

A variable of a higher source replaces the same variable of a lower one.

**Terminals.** The shell of each terminal of zendo ([Terminals](terminal/terminal.md)) gets the
agent environment too, so an agent that the user starts there by hand uses the same account. The
variables that select the startup files of the shell stay as they are (TERM-55).

**Worktrees.** zendo reads `.envrc` only from the main checkout: a worktree often has none,
because git usually ignores the file. The commands of an agent in a worktree get the variables
from the agent process. zendo finds the main checkout from the files of git, without a program.

**`.envrc`.** zendo reads the file itself and runs nothing of it: see [The `.envrc` file](envrc.md).
It reads `.envrc` only in a folder that the user trusts: see [Trusted folders](trust.md).

**System CA.** When the npm registry works only with the CA certificates of the system, `zendo
setup` saves `system_ca = true` in `~/.zendo/config.toml`. This is a fact of the machine, not of a
profile, so it applies with every profile and without one. The shell of a terminal does not get
the option: an older Node.js of a project does not start with it.

## Architecturally significant requirements

| ID    | Pattern    | Requirement | Source |
| ----- | ---------- | ----------- | ------ |
| ENV-1 | Ubiquitous | The system shall start the agent of the chat, each agent session in a terminal, each call of the name model, each agent check of `zendo doctor` and `zendo setup`, and the shell of each terminal with the agent environment. | User, 2026-11-19; User, 2026-11-20 |
| ENV-2 | Ubiquitous | The agent environment shall be the environment of the zendo process, with the variables of the `.envrc` of the main checkout over it, and the variables of the selected profile over both. | User, 2026-11-19 |
| ENV-3 | Ubiquitous | The system shall read `.envrc` from the root of the main checkout of the repository, also when the agent works in a worktree and when git ignores `.envrc`. | User, 2026-11-19 |

### Acceptance tests

**`agent_env_reaches_every_agent`** (mock agent; verifies ENV-1, ENV-2, ENV-3)

- Given a repository with a worktree outside its main checkout; a `.envrc` in the main checkout
  that git ignores, with `CLAUDE_CONFIG_DIR=A` and `FROM_ENVRC=1`; the profile `work` with
  `CLAUDE_CONFIG_DIR=B`; and fake agent programs that write their environment to a file.
- When zendo starts in the worktree with `--profile work`, and starts the chat agent, a session,
  a call of the name model, and the login check.
- Then each file has `CLAUDE_CONFIG_DIR=B`, `FROM_ENVRC=1`, and the `HOME` of zendo. Without a
  profile, the agent environment has `CLAUDE_CONFIG_DIR=A`.

**`a_terminal_shell_gets_the_agent_env`** (unit test; verifies ENV-1, ENV-5, TERM-55)

- Given an agent environment with `FROM_ENVRC=1`, `ZDOTDIR`, `BASH_ENV`, and the system CA that
  zendo added to `NODE_OPTIONS`.
- When the user opens a terminal of zendo and prints these variables.
- Then the shell prints `1`, `ZDOTDIR` and `BASH_ENV` as zendo itself has them, and
  `NODE_OPTIONS` without the system CA.

## Detailed requirements

| ID    | Pattern    | Requirement | Source |
| ----- | ---------- | ----------- | ------ |
| ENV-5 | Optional   | Where `~/.zendo/config.toml` has `system_ca = true`, the system shall add `--use-system-ca` to `NODE_OPTIONS` of the agent environment with every profile and without one, and to the environment of npm, and not to the environment of the shell of a terminal. | Design |
| ENV-6 | Ubiquitous | The system shall keep the agent environment while zendo runs: a change of `.envrc` or of a profile shall apply after the next start of zendo. | Design |
| ENV-10 | State     | While zendo runs outside a repository, the system shall read `.envrc` from the folder where the user started zendo. | User, 2026-11-20 |
