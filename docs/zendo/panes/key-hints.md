---
status: "implemented"
approved: "2026-11-18"
revision: "1"
date: "2026-11-18"
parent: "panes.md"
prev: "key-change.md"
level: "how a light model suggests free keys in the key float, without a wait for the user"
note: "Agent draft, moved out of key-change.md on 2026-11-18 to keep each document within two pages; the IDs did not change. Approved by the user on 2026-11-18. Implemented in zendo 0.93.0."
---

# Key hints

The key float suggests free keys when the pressed key clashes. See [Change a key](key-change.md),
[Keymap check and wizard](../cli/keymap-wizard.md) and
[ADR 0013](../../architecture/decisions/0013-key-hints-from-a-light-model.md).

**Suggestions.** When the float opens, zendo starts the hint channel with the keymap in effect:
`haiku` with low effort, through the API of the agent. On a clash, the float shows the suggestions
of the rules at once, and zendo asks the model for free keys near the pressed key and the old one.
Its checked answer replaces them when it comes. To take a suggestion, the user presses that key.
The user never waits for the model: a set key, a new key or `esc` cancels the open request, and a
late answer is dropped.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| REKEY-4 | Ubiquitous | The system shall show the suggestions as a hint that does not block the input of the user. | User, 2026-11-17 |
| REKEY-5 | Event      | When the user sets a key, presses another key or closes the float, the system shall cancel the open request and drop its answer. | User, 2026-11-17 |
| REKEY-6 | Ubiquitous | The system shall ask the model of `[hint_models]` of the config for the suggestions, by default `haiku` with low effort, through `claude -p` (NAMES-21, ADR 0010), with the keymap in effect. | User, 2026-11-18 |
| REKEY-7 | Ubiquitous | The system shall show only the suggestions that pass the free-key rules of WIZARD-10. | WIZARD-4, Design |

### Acceptance tests

**`a_clash_asks_the_model_with_low_effort_and_the_keymap`** (mock API; verifies REKEY-6, REKEY-10, REKEY-11)

- Given the float for `toggle_tree` and a mock model API that answers `alt-y` and `alt-u`.
- When the user presses `tab`.
- Then the float shows the keys of the rules at once, then `alt-y` and `alt-u`; the request asks
  for `haiku` with low effort and has the keymap in effect.

**`the_user_presses_keys_while_the_model_thinks`** (mock API; verifies REKEY-4)

- Given a mock model API that answers after two seconds.
- When the user presses a clashing key, then a free key at once.
- Then the free key is set without a wait.

**`a_late_answer_does_not_change_the_float`** (mock API; verifies REKEY-5)

- Given a mock model API that answers after the user closed the float.
- When the answer comes.
- Then the screen does not change.

**`a_suggestion_that_clashes_is_not_shown`** (mock API; verifies REKEY-7)

- Given a mock model that suggests `j` and a key of another action, for an action of the write pane.
- When the float shows the suggestions.
- Then neither key is among them.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| REKEY-10 | Event      | When the float opens, the system shall start the hint channel of the model and give it the keymap in effect. | User, 2026-11-17 |
| REKEY-11 | Event      | When a key clashes, the system shall show the suggestions of the rules at once, and replace them with the checked suggestions of the model when they come. | Design |
| REKEY-12 | Event      | When the user presses or clicks a suggested key, the system shall take it as the new key. | Design |
| REKEY-13 | Unwanted   | If the model is not available or does not answer, then the system shall keep the suggestions of the rules and show no error. | WIZARD-13, Design |
