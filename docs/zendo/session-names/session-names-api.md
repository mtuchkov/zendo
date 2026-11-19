---
status: "implemented"
approved: "2026-11-17"
revision: "2"
date: "2026-11-17"
parent: "session-names.md"
prev: "session-names-rename.md"
next: "session-names-buttons.md"
level: "who answers the questions for the names of agent sessions and pasted images: the API of the agent, `claude -p`, or a throwaway session"
note: "From the requests of the user on 2026-11-17. Approved by the user on 2026-11-17, implemented in zendo 0.82.0 to 0.84.0. ADR 0010 holds the decision. Revision 2 (user, 2026-11-18, in chat; the text waits for the review of the user): only `claude -p`, no API key and no `ant`; NAMES-20 retired, NAMES-22 added."
---

# Session names: API

The questions for the name of an agent session ([Session names](session-names.md), NAMES-1), of a
pasted image ([Chat](../chat.md), CLAUDE-37) and for the title of a new note
([Float: title](../float-title.md), FLOAT-18, FLOAT-19) need a few words back. zendo sends them past the
agent, straight to the model of the agent that the config names (ADR 0010).

**Which way.** zendo asks only through `claude -p`, with the login of the user (`claude auth
login`, also with a subscription): no tools, no setting sources, no MCP servers, no slash commands,
no saved session, and a system prompt of zendo. zendo checks the login with `claude auth status`, on
a thread at start. Anthropic allows `claude -p` with a subscription for local use; with a
subscription, it draws from the Agent SDK credit of the plan.

zendo reads no credential of another program and sends no request of its own to a model API
(NAMES-22): an endpoint security product counts a program that takes the token of another program
and sends it over the network as a theft of credentials. So zendo sends no `ANTHROPIC_API_KEY`,
`OPENAI_API_KEY` or `CODEX_API_KEY` anywhere, and does not run `ant`; `zendo doctor` only checks
whether such a key is set, for the login of the agent. Codex and other ACP agents have no
light model.

**No light model.** zendo asks the agent in a throwaway session (NAMES-16).

**The model.** `[name_models]` names the model (NAMES-12). zendo sends the short names as API
identifiers (NAMES-18), and any other name as the config writes it.

## Architecturally significant requirements

| ID       | Pattern  | Requirement | Source |
| -------- | -------- | ----------- | ------ |
| NAMES-15 | Optional | Where the configured agent is Claude and `claude auth status` reports a login, the system shall ask for the names of the agent sessions, the names of the pasted images and the titles of new notes through `claude -p`, without a start of the agent. | User, 2026-11-17; User, 2026-11-18 |
| NAMES-16 | Unwanted | If the configured agent is not Claude, or `claude` is not on the `PATH`, or it reports no login, then the system shall ask the agent for the names in a throwaway session. | User, 2026-11-17; User, 2026-11-18 |
| NAMES-21 | Ubiquitous | The system shall ask `claude -p` with no tools, no setting sources, no MCP servers, no slash commands, no saved session and a system prompt of its own. | User, 2026-11-17 |
| NAMES-22 | Ubiquitous | The system shall not read a credential of another program, shall not use an API key of the environment for a request of its own, and shall not send a request of its own to a model API. | User, 2026-11-18 |

Retired: NAMES-20 (the token of `ant auth print-credentials`), in revision 2: zendo does not run
`ant` for its token (NAMES-22, ADR 0023; User, 2026-11-18).

### Acceptance tests

The tests are in `tests/session_names.rs`, with the mock agent of [Sessions](../sessions.md) and a
mock `claude` script (`tests/common/claude.rs`).

**`claude_p_names_a_claude_session`** (mock `claude`; verifies NAMES-15, NAMES-18, NAMES-21)

- Given a Claude agent and a mock `claude` that answers two words.
- When agent session 1 gets its first agent message.
- Then the mock `claude` gets one question with `--model claude-haiku-5-5` and the question for the
  name; the button shows the first word; the mock agent gets no question for the name.

**`a_pasted_image_gets_its_name_from_the_light_model`** (mock `claude`; verifies NAMES-15,
CLAUDE-37, CLAUDE-38)

- Given a Claude agent and a mock `claude`.
- When the user pastes an image.
- Then the mock `claude` gets the image in base64, and the chip shows `Image: <name>` of its answer.

**`the_api_titles_the_note_after_three_lines`** and
**`closing_the_float_titles_the_note_through_the_api`** (mock `claude`, in `tests/float.rs`; verify
NAMES-15, FLOAT-18, FLOAT-19)

- Given a Claude agent and a mock `claude` that answers `TITLE: <title>`.
- When the float has three lines, or closes with one line.
- Then the mock `claude` gets one question for the title, the title goes into the float or the file,
  and the agent does not start.

**`zendo_reads_no_credential_of_another_program`** (unit test in `src/agent/api/api_tests.rs`;
verifies NAMES-22)

- Given `ant` and `claude` on the `PATH`.
- When zendo looks for its light model.
- Then it looks only for `claude`.

The other tests of session names and titles cover the fallback of NAMES-16.

## Detailed requirements

| ID       | Pattern  | Requirement | Source |
| -------- | -------- | ----------- | ------ |
| NAMES-18 | Optional | Where the system asks through `claude -p`, it shall send the short model names `haiku`, `sonnet` and `opus` as `claude-haiku-5-5`, `claude-sonnet-5-5` and `claude-opus-5-5`, and any other name as the config writes it. | Design |
| NAMES-19 | Unwanted | If `claude -p` answers with an error, then the system shall show the error once in the flash line and ask again by NAMES-10. | Design |

Tests: `an_error_of_the_light_model_shows_once` in `tests/session_names.rs` (NAMES-19); the choice
of the light model and NAMES-18 in `src/agent/api/api_tests.rs`; the flags, the login and the result of
`claude -p` in `src/agent/api/cli_tests.rs`.
