---
status: "approved"
approved: "2026-11-18"
date: "2026-11-17"
sources:
  - "The user, 2026-11-17: a double click on a key of the bottom panel or of the help changes the key and updates the keymap file; zendo starts a Claude session with the keymap; on a clash zendo asks Claude for a free key; the suggestion is a hint that does not block the user and is cancelled when the user is done or escapes; a small and fast model, so that the hints come in time."
  - "../../zendo/panes/key-change.md (REKEY-3) and ../../zendo/panes/key-hints.md (REKEY-4 to REKEY-7, REKEY-10, REKEY-13)"
  - "ADR 0007 (keymap wizard as a state machine), ADR 0009 (no UI stalls), ADR 0010 (names through the API)"
---

# 0013 Key hints from a light model through the API of the agent

## Context

The user changes a key in a small float ([Change a key](../../zendo/panes/key-change.md)). When the
pressed key clashes, a model suggests free keys. The hint has value only while the float is open:
the user often takes a key before a slow answer comes. So the model must answer in about a second,
the UI must not wait for it (ADR 0009), and zendo must drop a late answer.

The wizard of ADR 0007 already has the free-key rules (WIZARD-10) and checks each suggestion of the
agent. ADR 0010 already asks a light model through the API of the configured agent, with a fallback
for users without a key.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| A light ACP session of the agent, started when the float opens, with the keymap as context | The words of the user; the session keeps the keymap between two clashes. | A session needs seconds to start and loads the system prompt and tools of the agent; often it is not ready at the first clash. Cancel needs an ACP cancel. |
| A direct API request to the light model of ADR 0010 on each clash, with the keymap in the request | About one second with `haiku`; no agent context; cancel is to drop the request. | The keymap goes again with each request (a few thousand tokens). |
| Rules only | Instant; offline. | No sense of the old key; weak for unusual actions. |

## Decision

Accepted: the second option, with the rules as the first and last layer.

- When the float opens, zendo makes the hint channel: the API of ADR 0010, the model of
  `[hint_models]` (by default `haiku` with low effort; the names keep `[name_models]`), and a prompt with the keymap in effect, the action, its pane
  and the free keys of the rules.
- On each clash, zendo shows the suggestions of the rules at once and sends one request in the
  background. A new clash, a set key or a closed float cancels the open request. Each request has a
  generation number; zendo drops an answer whose number is not the current one.
- zendo checks each key of the answer with the rules of WIZARD-10 before it shows it.
- Without an API (a subscription user without a key), zendo uses `claude -p` as in NAMES-21 only if
  it answers in time, else it keeps the suggestions of the rules.

## Consequences

- The free-key rules move into a module that the wizard and the key float share.
- The hint channel is a small client next to the name questions of `src/agent/api`, with cancel and
  a generation number.
- The mock agent gets a mock model API with a delay, so that tests can check the cancel and the
  drop of late answers.
- The agent cannot write the keymap: only the key float writes it, after the user presses a key.

## Verification

`a_clash_keeps_the_old_key` (REKEY-3), `a_clash_asks_the_model_with_low_effort_and_the_keymap` (REKEY-6),
`the_user_presses_keys_while_the_model_thinks` (REKEY-4),
`a_late_answer_does_not_change_the_float` (REKEY-5) and
`a_suggestion_that_clashes_is_not_shown` (REKEY-7) in
[Change a key](../../zendo/panes/key-change.md) and [Key hints](../../zendo/panes/key-hints.md).

## Decision record

Proposed by the agent on 2026-11-17 from the request of the user. The user chose the direct API on 2026-11-17 and `haiku` with low effort on 2026-11-18 in the chat; approved by the user on 2026-11-18 in the chat.
