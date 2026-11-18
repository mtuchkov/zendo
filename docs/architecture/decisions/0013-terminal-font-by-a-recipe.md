---
status: "draft"
date: "2026-11-18"
sources:
  - "../../zendo/fonts-recipe.md"
  - "../../zendo/fonts-tip.md"
  - "The requests of the user on 2026-11-18: a font list in the theme picker, not a button of the agent; Claude in `zendo setup fonts`; a recipe from Claude; `zendo doctor` repairs the recipe in an interactive Claude session"
---

# 0013 Set the terminal font by a recipe that Claude writes

## Context

zendo cannot set its own font: the terminal draws every character with the font of its config
(FONT-23). [ADR 0012](0012-fonts-from-their-releases.md) gave the change of the terminal to the
agent of the chat, after the user agreed in the chat. The user wants a plain font list in the theme
picker, and the agent only in `zendo setup fonts` and in `zendo doctor`.

Each terminal sets its font in another way: Ghostty and Kitty by a line of their config and a
reload, Alacritty by `alacritty msg config`, WezTerm by an escape sequence and a handler in its Lua
config, iTerm2 by a profile, xterm by an escape sequence. Some terminals have no way at all.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| The agent of the chat changes the terminal each time | No terminal knowledge in zendo. | Each change of the font is a conversation with the agent. The user does not want it in the picker. |
| zendo knows the popular terminals | No agent at all for those terminals. | Code and tests for each terminal and each change of its config format. Other terminals stay without a font list. |
| Claude writes a recipe once; zendo sets fonts by it | A new terminal needs no code. The font list needs no agent. zendo knows only three ways: a line of a file, a command, an escape sequence. | Needs Claude once for each terminal, and again when the recipe stops to work. |

How Claude takes part:

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| A task through ACP, as `zendo doctor --fix --claude` | zendo controls each edit. | zendo must ask the user about each change of the terminal config itself. |
| An interactive `claude` session in the same terminal | The user sees what Claude does and agrees to each change in Claude itself. | zendo knows the result only from the recipe file, when the session ends. |

## Decision

Claude writes the recipe in an interactive `claude` session that `zendo setup fonts` and
`zendo doctor --fix --claude` start (FONT-22, FONT-25). zendo sets fonts only by the recipe, runs
its commands without a shell, and checks it in `zendo doctor` (FONT-23, FONT-24, FONT-30). This
decision replaces the change of the terminal by the agent of the chat in ADR 0012.

## Consequences

- The font list of the theme picker works without an agent and offline.
- A recipe can stop to work after an update of the terminal; `zendo doctor` finds it, and
  `zendo doctor --fix --claude` repairs it.
- Tests use a mock `claude` and a recipe in a test home; no test starts the real Claude.
