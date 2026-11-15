---
status: "draft"
revision: "2"
date: "2026-11-15"
parent: "cli.md"
prev: "doctor.md"
next: "upgrade.md"
level: "the keymap check of doctor, the wizard of --keys that moves clashing keys, and its offer at setup"
note: "Agent draft from the note of the user `2026-11-15-keymap-wizard.md` and the answers of the user on 2026-11-15. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Keymap check and wizard

A key of zendo clashes when the OS or the terminal takes it first: it never reaches zendo. Doctor
finds the clashes; `zendo doctor --keys` opens a wizard that moves each action to a free key. See
[Keys, hints and help](../panes/keymap.md), [Doctor](doctor.md), [Setup](setup.md) and
[Usage counts](../usage.md).

**The check.** Doctor compares the keymap in effect with the keys that the OS and the terminal
take, from tables that zendo ships (WIZARD-14) and from the config of the terminal where it can
read it.

**The wizard.** The clashes depend on the terminal, and the user chooses it. The user presses each
doubtful key; for each clash the user picks a suggestion, types a key, keeps the clash, or skips
it; at the end the wizard writes the changes on confirmation. `--fix` does not open the wizard: it
asks no question (CMD-15).

**At setup.** `zendo setup` offers the wizard to a person in a terminal. When an agent or the
installer runs setup, nobody can press the keys, so setup tells the agent what to ask the user.

**Screen, agent and suggestions.** A zendo screen lists the clashes with their states and the
progress, for example `3 of 7`, and shows for the current clash the action, its pane and modes, and
up to three suggestions with reasons. The steps are a state machine in zendo, not a conversation:
the agent explains and ranks suggestions with wizard tools ([ADR 0007](../../architecture/decisions/0007-keymap-wizard-as-a-state-machine.md)) (the clashes, the free keys of an
action, the usage counts, the test of a key). A good suggestion keeps the sense of the old key and
gives frequent actions short keys. The example counts that zendo ships are the counts of the
author without personal data; the counts of the user replace them when they are more.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| WIZARD-1 | Event      | When the user runs `zendo doctor`, the system shall report each key of the keymap in effect that the OS or the terminal takes, with its action and pane and what it does there, and name `zendo doctor --keys`. | User, 2026-11-15 |
| WIZARD-2 | Event      | When the user runs `zendo doctor --keys` in a terminal, the system shall open the keymap wizard. | User, 2026-11-15 |
| WIZARD-3 | Unwanted   | If the input or the output of doctor or setup is not a terminal, then the system shall not open or offer the wizard and shall wait for no input. | User, 2026-11-15 |
| WIZARD-4 | Ubiquitous | The system shall run the wizard as a fixed sequence of steps, in which the agent only explains and ranks suggestions that the system checks. | User, 2026-11-15 |
| WIZARD-5 | Event      | When the user confirms the changes, the system shall write them to the keymap file of the user, and change no other file than the config that names it. | Design |

### Acceptance tests

**`doctor_reports_a_key_that_the_os_takes`** (unit test; verifies WIZARD-1)

- Given an action on `ctrl-left` on macOS, where Mission Control takes it.
- When the user runs `zendo doctor`.
- Then the output names the key, the action, its pane, Mission Control and `zendo doctor --keys`.

**`the_wizard_opens_only_in_a_terminal`** (integration test; verifies WIZARD-2, WIZARD-3)

- Given a keymap with one clash.
- When `zendo doctor --keys` and `zendo setup` run in a pseudo terminal, then piped.
- Then only the pseudo terminal gets the wizard or its offer; the piped runs wait for no input, and
  setup ends with the instruction for the agent.

**`the_wizard_shows_only_checked_suggestions`** (mock agent; verifies WIZARD-4)

- Given an agent that suggests a key of another action of the pane.
- When the wizard shows the clash.
- Then that key is not among the suggestions.

**`the_wizard_writes_only_after_the_confirmation`** (terminal check; verifies WIZARD-5)

- Given a fixed clash.
- When the user leaves before the confirmation, then confirms in a second run.
- Then only the second run changes a file: the keymap file of the user.

## Detailed requirements

| ID        | Pattern    | Requirement | Source |
| --------- | ---------- | ----------- | ------ |
| WIZARD-6  | Event      | When the wizard starts, the system shall name the terminal it runs in and ask whether the user runs zendo there; if not, ask the user to run `zendo doctor --keys` in that terminal, and stop. | User, 2026-11-15 |
| WIZARD-7  | Event      | When the wizard tests a key, the system shall ask the user to press it, and count a clash when no key or another key reaches zendo. | User, 2026-11-15 |
| WIZARD-8  | Event      | When the user chooses a key, the system shall test it before it takes it. | Design |
| WIZARD-9  | Ubiquitous | The system shall show every clash with its state and the progress, and what the current key does in the OS or the terminal and in zendo. | User, 2026-11-15 |
| WIZARD-10 | Ubiquitous | The system shall suggest only keys that are free in the pane of the action and the global pane (every pane for a global action), with a modifier in write mode. | User, 2026-11-15 |
| WIZARD-11 | Ubiquitous | The system shall rank suggestions by the usage counts of the user when they have more actions than the example counts that zendo ships, else by the example counts. | User, 2026-11-15 |
| WIZARD-12 | Event      | When the user keeps a clash, the system shall say how to free the key in the OS or the terminal, where it knows how. | Design |
| WIZARD-13 | Unwanted   | If the agent is not available, then the system shall run the wizard with the suggestions of its rules. | Design |
| WIZARD-14 | Ubiquitous | The system shall know the keys that macOS, Terminal, iTerm2, GNOME, GNOME Terminal and the default terminal of Ubuntu take. | User, 2026-11-15 |
| WIZARD-15 | Event      | When `zendo setup` ends in a terminal, the system shall offer to start the keymap wizard. | User, 2026-11-15 |
| WIZARD-16 | Unwanted   | If `zendo setup` does not run in a terminal, then the system shall end its output with an instruction for the agent to ask the user to run `zendo doctor --keys` in the terminal where the user runs zendo. | User, 2026-11-15 |

