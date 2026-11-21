---
status: "approved"
approved: "2026-11-17"
note: "ADR 0023 (draft, 2026-11-18) changes it in part: only `claude -p`; no API key and no `ant`."
date: "2026-11-17"
sources:
  - "The user, 2026-11-17: a name shall not change while its session runs; use the AI API directly in place of agent sessions, since only one word comes back; the API depends on the configured agent, Claude or Codex; fall back to a session without a key; consider the sign-in of the `ant` CLI; Anthropic encourages local development with the Agent SDK and `claude -p`."
  - "Claude Code docs, headless mode: `claude -p` on subscription plans draws from the Agent SDK credit of the plan (since 2026-06-15)."
  - "../../zendo/session-names/session-names-api.md (NAMES-15 to NAMES-21)"
  - "../../zendo/chat.md (CLAUDE-37 to CLAUDE-39)"
  - "ADR 0001 (Claude through ACP)"
  - "Claude Platform docs, CLI authentication and scripting: scripts may call the API with the token of `ant auth print-credentials --access-token`."
  - "Claude Code docs, legal and compliance (2026-02): OAuth tokens of Claude Free, Pro and Max plans are not for third-party tools."
---

# 0010 Session names through the API of the agent

## Context

zendo names the agent sessions and the pasted images, and titles new notes (FLOAT-18), with a light
model. Up to now it opened a
throwaway ACP session of the agent for each question (ADR 0001). A throwaway session starts an agent
process context, loads the system prompt and the tools of the agent, and holds the work queue of
the recaps while it runs. The answer is one word.

The user asked to call the model API directly. The API shall follow the agent of the config:
Claude or Codex. Three credentials exist on a machine of a user:

- an API key in the environment: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY` or `CODEX_API_KEY`;
- the Claude Console sign-in of the `ant` CLI, which the docs allow for any HTTP client;
- the subscription sign-in of Claude Code or of Codex. The terms of Anthropic forbid the Claude
  subscription token in third-party tools, and the server refuses it, but they allow `claude -p`
  with a subscription for local use. The ChatGPT token of Codex works only with an endpoint that
  OpenAI does not document.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Throwaway ACP session (now) | Works with every login, also a subscription. | Slow start; the agent context; holds the recap queue. |
| Direct API with a key or the `ant` sign-in, else the throwaway session | Fast, small requests; no agent context; a subscription user still gets names. | Two ways to keep; an HTTP client dependency. |
| Direct API only | One way. | No names for a user with a subscription only. |
| Subscription tokens of the agents | No key needed. | Against the terms of Anthropic; undocumented for Codex. |
| `claude -p` with no tools, settings or MCP servers, after a key and `ant` | Uses the subscription as Anthropic allows; a short system prompt. | A process start per question (about 1 s); Claude only. |
| `codex exec` | Uses every login of Codex. | A full agent run per word. |

## Decision

Accepted: the direct API with a fallback to the throwaway session.

- zendo shall choose the API from the kind of the configured agent: the Anthropic Messages API
  (`POST {ANTHROPIC_BASE_URL or https://api.anthropic.com}/v1/messages`) for Claude, the OpenAI
  Responses API (`POST {OPENAI_BASE_URL or https://api.openai.com/v1}/responses`) for Codex. Another
  ACP agent has no API.
- The credential shall come from `[env]` of the config, then from the environment of zendo. For
  Claude: `ANTHROPIC_API_KEY` as `x-api-key`; without it, the token of
  `ant auth print-credentials --access-token` as a bearer token, read again for each question, so
  that `ant` refreshes it. For Codex: `OPENAI_API_KEY`, then `CODEX_API_KEY`, as a bearer token.
- zendo shall find the credential once at start, and the `ant` token at each question.
- The request shall run on a thread of its own with a limit of 20 s; the tick of the app takes the
  answer, so the UI never waits (as ADR 0009).
- A question through the API shall not wait for the agent or for the recap queue, and shall not
  start the agent; the titles of new notes take the same way.
- An HTTP error shall show once in the flash line; the name waits for the next threshold.
- The short model names of `[name_models]` map to API identifiers (NAMES-18); a full identifier
  passes as it is.

## Consequences

- New dependencies: `ureq` (blocking HTTP with rustls, the proxy of the environment) and `base64`
  (images). No async client: a short thread per question is enough.
- `AgentLink` holds the API and the answers; `names.rs` and `image_names.rs` send through it when it
  has an API, and through a throwaway session when it has none.
- The question for a rename is gone (NAMES-14 retired, NAMES-17): fewer questions in all.
- The mapping of short model names needs an update when the vendors release new models.
- The `ant` sign-in bills the Console workspace, not a subscription. `claude -p` draws from the
  Agent SDK credit of the subscription.
- `claude -p` without `--bare` still reads `~/.claude/CLAUDE.md`; `--bare` would skip it but reads
  no subscription login.

## Verification

- `tests/session_names.rs`: `the_anthropic_api_names_a_claude_session`,
  `the_openai_api_names_a_codex_session`, `a_pasted_image_gets_its_name_from_the_api` with a mock
  HTTP server; the other tests cover the fallback.
- `tests/session_names.rs`: `claude_p_names_a_claude_session` with a mock `claude`.
- `src/agent/api/api_tests.rs`: the choice of the API and of the credential, the `ant` token, the
  bodies of the requests, the parse of the answers, and the model names.
- `src/agent/api/cli_tests.rs`: the flags of `claude -p`, its input line, its login, its result.
