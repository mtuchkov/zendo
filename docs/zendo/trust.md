---
status: "implemented"
approved: "2026-11-20"
revision: "1"
date: "2026-11-20"
parent: "zendo.md"
prev: "agent-profiles.md"
next: "questions.md"
level: "the trust of the user in the folder where zendo works: the question, the list of the trusted folders, what zendo does not do without the trust"
note: "Agent draft from the decision of the user on 2026-11-20 in chat; approved by the user on 2026-11-20 in chat, with two changes: `zendo doctor` and `zendo setup` ask too, and zendo asks about every new folder. Decisions of the agent that the user approved on 2026-11-20 in chat: no saved answer for `zendo doctor` without `--fix` (CMD-14) and for the home folder."
---

# Trusted folders

The files of a repository can make a program that zendo starts run code of the repository: a
variable of its `.envrc` changes how the agents and the terminals start programs, and an agent
loads settings, hooks and tool servers from its folder. So zendo uses a folder in these ways
only after the user trusts it, as Claude Code and Codex do.

## The trust

**Trusted folder.** A folder is trusted when it, or a folder above it, is in the list
`trusted_dirs` of `~/.zendo/config.toml`. The trust of a folder covers every folder in it: the
trust of `~/src` covers each repository in `~/src`. The user can write the list by hand; an
entry that is no full path trusts nothing.

```toml
trusted_dirs = ["~/src/zendo", "~/src/work"]
```

**The location.** zendo asks about every new location. In a repository, the location is the root
of the main checkout, whose `.envrc` zendo reads ([Agent environment](agent-env.md)), and the
root of the worktree where zendo runs, when it is outside the main checkout. Outside a
repository, the location is the folder where the user starts zendo.

**What needs the trust.** In a location that is not trusted, zendo does not read `.envrc`, and
does not start an agent in the folders of the location.

**The question.** `zendo`, `zendo notes`, `zendo doctor` and `zendo setup` read `.envrc` or start
an agent. In a terminal, in a location that is not trusted, each of them asks first:

```text
zendo: do you trust the files in /home/u/src/app?
  zendo gives the variables of its .envrc to the agents and the terminals, and starts agents in
  this folder. Both can run code of the folder.
  Trust this folder and the folders in it? [y/N]
```

With `y`, zendo adds the folders of the location to `trusted_dirs` and goes on. With any other
answer, zendo exits and changes nothing. In two cases the answer `y` counts for that run only,
and zendo saves nothing: `zendo doctor` without `--fix` writes no file (CMD-14); and the home
folder and the folders above it hold every other folder of the user, so their trust would end
the question for all of them. The user can write such a folder into `trusted_dirs` by hand.

**Without a terminal** zendo cannot ask: the installer runs `zendo setup` so. `zendo doctor` and
`zendo setup` then go on without the location: without `.envrc`, with the
programs of their checks in the data folder of zendo, and without a task for an agent in the
location. They say that the folder is not trusted, and how to trust it. `zendo` and `zendo
notes` stop.

**The config file.** zendo changes only the key `trusted_dirs` of `~/.zendo/config.toml`; all
other content of the file stays as it is. If zendo cannot write the file, then the answer counts
for this run, and zendo asks again at the next start.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TRUST-1 | State      | While a location is not trusted, the system shall not read its `.envrc` and shall not start an agent in a folder of the location. | User, 2026-11-20 |
| TRUST-2 | Event      | When the user starts `zendo`, `zendo notes`, `zendo doctor` or `zendo setup` in a terminal, in a location that is not trusted, the system shall ask whether the user trusts the folder, before it reads `.envrc` and before it starts an agent. | User, 2026-11-20 |
| TRUST-3 | Ubiquitous | A folder shall count as trusted when it, or a folder above it, is in `trusted_dirs` of `~/.zendo/config.toml`. | User, 2026-11-20 |
| TRUST-4 | Unwanted   | If the user does not answer `y` to the question, then zendo shall exit, and shall not change `trusted_dirs`. | User, 2026-11-20 |

### Acceptance tests

**`an_untrusted_repository_gives_nothing_to_the_agents`** (mock agent; verifies TRUST-1, TRUST-3,
TRUST-6)

- Given a repository with a `.envrc` that sets `FROM_ENVRC=1`, a config whose `trusted_dirs` does
  not hold it, and no terminal.
- When `zendo doctor` runs in the repository, then again after `trusted_dirs` gets the folder
  above the repository.
- Then the first run names the folder as not trusted, and its agent starts outside the
  repository, without `FROM_ENVRC`. The agent of the second run starts in the repository, with
  `FROM_ENVRC=1`.

**`zendo_asks_before_it_trusts`** (unit test; verifies TRUST-2, TRUST-4, TRUST-5)

- Given a repository that is not trusted, with a worktree outside its main checkout.
- When zendo starts in the worktree and the user answers `n`, and then starts again and the user
  answers `y`.
- Then the first start stops and the config file is as before. The second start goes on, and
  `trusted_dirs` holds the root of the main checkout and the root of the worktree, with the other
  content of the file as before.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TRUST-5 | Event      | When the user answers `y`, the system shall add the root of the main checkout to `trusted_dirs`, and the root of the worktree when it is outside the main checkout, and shall keep all other content of the config file; it shall not save the home folder or a folder above it, and `zendo doctor` without `--fix` shall write nothing (CMD-14). | TRUST-2 |
| TRUST-6 | Unwanted   | If a location is not trusted and zendo has no terminal to ask in, then `zendo doctor` and `zendo setup` shall go on without the location and shall name the folder and the two ways to trust it, and every other command shall stop. | Design |
| TRUST-7 | State      | While zendo runs outside a repository, the folder where the user started zendo shall be the location that needs the trust. | User, 2026-11-20 |
| TRUST-8 | Unwanted   | If zendo cannot write the config file after the answer `y`, then the answer shall count for this run only. | FAIL-1 |
