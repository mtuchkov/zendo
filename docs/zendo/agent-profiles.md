---
status: "implemented"
approved: "2026-11-20"
revision: "3"
date: "2026-11-20"
parent: "zendo.md"
prev: "agent-env.md"
next: "questions.md"
level: "the profiles of the agent environment: their tables, their selection by flag or by folder, and the threads that they own"
note: "Agent draft from the decisions of the user on 2026-11-19 in chat. Approved by the user on 2026-11-20. Revision 2: each profile keeps threads of its own in place of the refusal of PROFILE-4 (the user, 2026-11-20). Revision 3: PROFILE-8, the name of the profile in the top line (the user, 2026-11-20)."
---

# Agent profiles

A profile is a named set of environment variables for the agents, for example the config folders
of a work account. The selected profile is the highest source of the
[agent environment](agent-env.md).

## Profiles and their selection

A profile is a table of `~/.zendo/config.toml`. The table `[profile_dirs]` maps a folder to a
profile:

```toml
[profiles.work]
CLAUDE_CONFIG_DIR = "~/.claude-work"
CODEX_HOME = "~/.codex-work"

[profile_dirs]
"~/src/work" = "work"
```

`--profile <name>` selects a profile. Without it, `[profile_dirs]` selects the profile of the
nearest folder that contains the root of the main checkout. Without a match, no profile applies.
The main checkout decides, so a worktree outside the repository gets the profile of its
repository.

The config holds variables for the agents only in profiles. `--agent` selects the kind of the agent
(Claude or Codex); the profile selects the variables, and the two are independent.

**On the screen.** The name of the profile stands at the left end of the top line, before the
branch and the path: a badge in the blue of the theme, as the name of the pane in the status bar.
Without a profile, the top line has no badge.

**Threads.** The sessions of an agent are stored under its config folder, so a thread made with
one profile cannot be resumed with another. Each profile has threads of its own in each agent
session: after a return to `work`, the thread of `work` goes on.

## Architecturally significant requirements

| ID        | Pattern  | Requirement | Source |
| --------- | -------- | ----------- | ------ |
| PROFILE-1 | Event    | When the user starts zendo with `--profile <name>`, the system shall apply the table `[profiles.<name>]` of `~/.zendo/config.toml`. | User, 2026-11-19 |
| PROFILE-2 | Event    | When the user starts zendo without `--profile`, the system shall apply the profile that `[profile_dirs]` maps to the nearest folder that contains the root of the main checkout, and no profile if no folder matches. | User, 2026-11-19 |
| PROFILE-3 | Unwanted | If `~/.zendo/config.toml` has no profile with the name that `--profile` or `[profile_dirs]` gives, then zendo shall not start and shall name the profile and the config file. | Design |
| PROFILE-4 | Ubiquitous | The system shall keep the threads and the sessions of each profile apart, and shall resume a thread only with the profile that it started with. | User, 2026-11-20 |

### Acceptance tests

**`profile_dirs_select_nearest`** (unit test; verifies PROFILE-1, PROFILE-2)

- Given `[profile_dirs]` with `<tmp>/src = "home"` and `<tmp>/src/work = "work"`, and a repository
  at `<tmp>/src/work/app` with a worktree at `<tmp>/wt`.
- When zendo starts in the worktree, then in `<tmp>/other`, then in the worktree with
  `--profile home`.
- Then the profile is `work`, then none, then `home`.

**`unknown_profile_stops_start`** (unit test; verifies PROFILE-3)

- Given a config without `[profiles.home]`.
- When zendo starts with `--profile home`.
- Then zendo exits with an error that names `home` and `~/.zendo/config.toml`.

**`threads_are_kept_per_profile`** (unit test; verifies PROFILE-4, PROFILE-6)

- Given a thread of agent session 1 that started with the profile `work`, and a thread that zendo
  recorded before profiles.
- When zendo runs without a profile, and then with `--profile work`.
- Then agent session 1 has the old thread without a profile, and the thread of `work` with it.

## Detailed requirements

| ID        | Pattern    | Requirement | Source |
| --------- | ---------- | ----------- | ------ |
| PROFILE-5 | Ubiquitous | `~/.zendo/config.toml` shall hold environment variables for the agents only in profiles. | User, 2026-11-19 |
| PROFILE-6 | Ubiquitous | A thread that zendo recorded without a profile shall count as a thread without a profile. | PROFILE-4 |
| PROFILE-7 | Event      | When the user runs `zendo doctor` or `zendo setup` with `--profile <name>`, the command shall check the agents with that profile. | ENV-1 |
| PROFILE-8 | State      | While a profile is selected, the system shall show its name at the left end of the top line, before the branch, as a badge in the blue of the theme. | User, 2026-11-20 |
