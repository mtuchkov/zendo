---
status: "approved"
approved: "2026-11-20"
revision: "1"
date: "2026-11-19"
parent: "zendo.md"
prev: "agent-env.md"
next: "questions.md"
level: "the profiles of the agent environment: their tables, their selection by flag or by folder, and the threads that they own"
note: "Agent draft from the decisions of the user on 2026-11-19 in chat. Approved by the user on 2026-11-20."
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

**Threads.** The sessions of an agent are stored under its config folder. A thread made with one
profile cannot be resumed with another. zendo records the profile of each thread and session.

## Architecturally significant requirements

| ID        | Pattern  | Requirement | Source |
| --------- | -------- | ----------- | ------ |
| PROFILE-1 | Event    | When the user starts zendo with `--profile <name>`, the system shall apply the table `[profiles.<name>]` of `~/.zendo/config.toml`. | User, 2026-11-19 |
| PROFILE-2 | Event    | When the user starts zendo without `--profile`, the system shall apply the profile that `[profile_dirs]` maps to the nearest folder that contains the root of the main checkout, and no profile if no folder matches. | User, 2026-11-19 |
| PROFILE-3 | Unwanted | If `~/.zendo/config.toml` has no profile with the name that `--profile` or `[profile_dirs]` gives, then zendo shall not start and shall name the profile and the config file. | Design |
| PROFILE-4 | Unwanted | If the user resumes a thread or a session that started with a different profile than the current one, then the system shall not resume it and shall name the profile of the thread. | Design |

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

**`resume_keeps_profile`** (mock agent; verifies PROFILE-4)

- Given a thread that started with the profile `work`.
- When zendo runs without a profile and the user resumes the thread.
- Then the thread does not resume, and the message names `work`.

## Detailed requirements

| ID        | Pattern    | Requirement | Source |
| --------- | ---------- | ----------- | ------ |
| PROFILE-5 | Ubiquitous | `~/.zendo/config.toml` shall hold environment variables for the agents only in profiles. | User, 2026-11-19 |
| PROFILE-6 | Event      | When a thread or a session starts, the system shall record the name of the profile, or no profile. A thread without a record shall count as a thread without a profile. | PROFILE-4 |
| PROFILE-7 | Event      | When the user runs `zendo doctor` or `zendo setup` with `--profile <name>`, the command shall check the agents with that profile. | ENV-1 |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Shall zendo show the current profile on the screen, for example in the status line? | PROFILE-1, PROFILE-2 | User |
