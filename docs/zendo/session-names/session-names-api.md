---
status: "implemented"
approved: "2026-11-17"
revision: "1"
date: "2026-11-17"
parent: "session-names.md"
prev: "session-names-rename.md"
next: "session-names-buttons.md"
level: "who answers the questions for the names of agent sessions and pasted images: the API of the agent, `claude -p`, or a throwaway session"
note: "From the requests of the user on 2026-11-17. Approved by the user on 2026-11-17, implemented in zendo 0.82.0 to 0.84.0. ADR 0010 holds the decision."
---

# Session names: API

The questions for the name of an agent session ([Session names](session-names.md), NAMES-1), of a
pasted image ([Chat](../chat.md), CLAUDE-37) and for the title of a new note
([Float: title](../float-title.md), FLOAT-18, FLOAT-19) need a few words back. zendo sends them past the
agent, straight to the model of the agent that the config names (ADR 0010).

**Which API.** The agent of the config decides: for Claude the Anthropic Messages API, for Codex
the OpenAI Responses API. Another ACP agent has no API. The API takes `ANTHROPIC_BASE_URL` and
`OPENAI_BASE_URL` for its address, as the agents do.

**Which credential.** zendo reads `[env]` of the config, then its own environment, in this order:

| Agent  | Credential | How it goes |
| ------ | ---------- | ----------- |
| Claude | `ANTHROPIC_API_KEY` | header `x-api-key` |
| Claude | the `ant` CLI, after `ant auth login` (Claude Console) | `Authorization: Bearer` with the token of `ant auth print-credentials --access-token`, again for each question |
| Claude | the `claude` CLI, after `claude auth login`, also with a subscription | `claude -p` with no tools, no setting sources, no MCP servers, no slash commands, no saved session and a system prompt of zendo |
| Codex  | `OPENAI_API_KEY`, then `CODEX_API_KEY` | `Authorization: Bearer` |

Anthropic allows `claude -p` with a subscription for local use; the subscription token in a
third-party program is not allowed, so zendo never reads it. With a subscription, `claude -p` draws
from the Agent SDK credit of the plan. zendo checks the sign-in of `ant` with
`ant auth print-credentials` and the one of `claude` with `claude auth status`, on a thread at
start.

**No credential.** zendo asks the agent in a throwaway session (NAMES-16).

**The model.** `[name_models]` names the model (NAMES-12). zendo sends the short names as API
identifiers (NAMES-18), and any other name as the config writes it.

## Architecturally significant requirements

| ID       | Pattern  | Requirement | Source |
| -------- | -------- | ----------- | ------ |
| NAMES-15 | Optional | Where the configured agent has an API credential, the system shall ask for the names of the agent sessions, the names of the pasted images and the titles of new notes through the API of that agent, without a start of the agent: the Anthropic Messages API for Claude, the OpenAI Responses API for Codex. | User, 2026-11-17 |
| NAMES-16 | Unwanted | If the configured agent has no API credential, then the system shall ask the agent for the names in a throwaway session. | User, 2026-11-17 |
| NAMES-20 | Optional | Where the agent is Claude and the environment and `[env]` of the config have no `ANTHROPIC_API_KEY`, the system shall use the access token that `ant auth print-credentials --access-token` prints, if the command succeeds. | User, 2026-11-17 |
| NAMES-21 | Optional | Where the agent is Claude, with no `ANTHROPIC_API_KEY` and no `ant` on the `PATH`, and `claude auth status` reports a login, the system shall ask through `claude -p` with no tools, no setting sources, no MCP servers and a system prompt of its own. | User, 2026-11-17 |

### Acceptance tests

The tests are in `tests/session_names.rs`, with the mock agent of [Sessions](../sessions.md), a mock
API server on `127.0.0.1` that answers as the Anthropic or the OpenAI API, and a mock `claude`
script.

**`the_anthropic_api_names_a_claude_session`** and **`the_openai_api_names_a_codex_session`**
(mock API server; verify NAMES-15, NAMES-18)

- Given a Claude or a Codex agent with an API key, and the mock API server.
- When agent session 1 gets its first agent message.
- Then the mock server gets one request to `/v1/messages` with `x-api-key`, or to `/v1/responses`
  with a bearer token, for the API model of the config; the button shows the first word of its
  answer; the mock agent gets no question for the name.

**`a_pasted_image_gets_its_name_from_the_api`** (mock API server; verifies NAMES-15, CLAUDE-37,
CLAUDE-38)

- Given a Claude agent with an API key, and the mock API server.
- When the user pastes an image.
- Then the mock server gets the image in base64, and the chip shows `Image: <name>` of its answer.

**`the_api_titles_the_note_after_three_lines`** and
**`closing_the_float_titles_the_note_through_the_api`** (mock API server, in `tests/float.rs`;
verify NAMES-15, FLOAT-18, FLOAT-19)

- Given a Claude agent with an API key, and the mock API server that answers `TITLE: <title>`.
- When the float has three lines, or closes with one line.
- Then the mock server gets one question for the title, the title goes into the float or the file,
  and the agent does not start.

**`claude_p_names_a_claude_session`** (mock `claude` script; verifies NAMES-21)

- Given a Claude agent with `claude -p` as its way, and a mock `claude` that answers two words.
- When agent session 1 gets its first agent message.
- Then the mock `claude` gets one user message with the question for the name, and the button shows
  the first word.

**`the_ant_sign_in_is_the_claude_credential`** (unit test in `src/agent/api/api_tests.rs`, with a
mock `ant` script; verifies NAMES-20)

- Given a Claude agent, no `ANTHROPIC_API_KEY`, and a mock `ant` that prints a token.
- When zendo asks through the API.
- Then the request has `Authorization: Bearer <token>`; with a mock `ant` that fails, the agent has
  no API credential.

The other tests of session names and titles cover the fallback of NAMES-16.

## Detailed requirements

| ID       | Pattern  | Requirement | Source |
| -------- | -------- | ----------- | ------ |
| NAMES-18 | Optional | Where the system asks through the API or `claude -p`, it shall send the short model names `haiku`, `sonnet` and `opus` as `claude-haiku-5-5`, `claude-sonnet-5-5` and `claude-opus-5-5`, `luna` and `sol` as `gpt-6-luna` and `gpt-6-sol`, and any other name as the config writes it. | Design |
| NAMES-19 | Unwanted | If the API or `claude -p` answers with an error, then the system shall show the error once in the flash line and ask again by NAMES-10. | Design |

Tests: `an_error_of_the_api_shows_once` in `tests/session_names.rs` (NAMES-19); the choice of the
credential and NAMES-18 in `src/agent/api/api_tests.rs`; the flags, the login and the result of
`claude -p` in `src/agent/api/cli_tests.rs`.
