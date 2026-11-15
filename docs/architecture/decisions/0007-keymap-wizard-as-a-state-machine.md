---
status: "draft"
date: "2026-11-15"
sources:
  - "../../zendo/cli/keymap-wizard.md (WIZARD-4, WIZARD-10, WIZARD-13)"
  - "The note of the user `2026-11-15-keymap-wizard.md`: the wizard is agent driven, but strictly follows a state machine and may use wizard tools rather than rely on the model."
---

# 0007 The keymap wizard is a state machine that uses the agent

## Context

`zendo doctor --keys` opens a wizard that moves the keys of zendo away from the keys that the OS and
the terminal take ([Keymap check and wizard](../../zendo/cli/keymap-wizard.md)). The wizard must
explain each clash in words and suggest good keys: a key that keeps the sense of the old one, short
keys for frequent actions. A model does this well. But the wizard also writes the keymap of the
user, and a wrong key there breaks zendo: a key that another action of the pane has, a letter in
write mode, a key that the terminal takes. The wizard must also work without the agent (WIZARD-13).

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| A conversation with the agent, which edits the keymap file | Least code in zendo; free-form questions of the user. | The order of the steps, the key tests and the free keys depend on the model; a mistake goes to the keymap file; no wizard without the agent. |
| Rules only, without the agent | Same result for the same input; works offline. | Terse explanations; weaker suggestions for keys of unusual actions. |
| A state machine in zendo that asks the agent for words and ranks through wizard tools | The steps, the tests and the checks of keys are code; the agent adds explanations and ranking; without the agent the rules remain. | Two parts to keep in step: the tools and the rules. |

## Decision

Proposal of the author, not yet accepted: the third option.

The keymap wizard shall keep its steps, the key tests and the writes in a state machine in zendo.
The agent shall reach the keymap only through wizard tools that zendo gives it: the clashes, the
free keys of an action, the usage counts and the test of a key. zendo shall check each suggestion of
the agent against the rules of WIZARD-10 before it shows it, and shall show the suggestions of its
rules when the agent is not available.

## Consequences

- zendo gets a set of wizard tools for the agent, next to the MCP tools of zendo sessions.
- The rules for free keys live in code and serve both the suggestions without the agent and the
  checks of the suggestions of the agent.
- The agent cannot write the keymap file: only the write step of the state machine does, after the
  confirmation of the user (WIZARD-5).
- Tests can run the whole wizard with the mock agent, because the order of the steps does not depend
  on the model.

## Verification

`the_wizard_shows_only_checked_suggestions` (WIZARD-4) and
`the_wizard_writes_only_after_the_confirmation` (WIZARD-5) in
[Keymap check and wizard](../../zendo/cli/keymap-wizard.md).

## Decision record

Proposed by the agent on 2026-11-15 from the note of the user. Not yet reviewed by the user.
