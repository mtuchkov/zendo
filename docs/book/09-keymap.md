---
review:
  reviewer: "agent, new context"
  date: "2026-11-20"
  passes: 1
  result: "pass"
  hash: "sha256:049a1f4b15a9773d072538efa8bb43cd16630b007503d04f396658dd2b776f5f"
---
# Chapter 9: The keymap

One key can have different meanings. In the tree, `y` means "yes, delete". In the editor, it is
only a letter. The question is: which part gets the key push?

The keymap tells which key gives which action.

**The key separately from the action**

The path of a key push is:

1. Key → **action** from the keymap. For example, `ctrl-s` → `"save"`. The keymap is data from the
   config.
2. Action → **message** of the active panel. `"save"` in the editor → `Msg::Editor(Save)`.

Why two steps: "which key" and "what to do" are different questions. When the user changes a key,
only the keymap changes. The code of the components does not know about keys. It works with
actions. And a test can send `Msg::Editor(Save)` directly, with no attention to the key layout.

**Where a key goes**

A key goes to the keymap of the active panel or to the global keymap. If neither has an action for
the key, then nothing occurs. The keymap of a panel depends on its mode.

**Global keys**

A global key works in all panels, in all modes and in all overlays, with no exception. If a key
does not work in one place, then it is not global. It is a key of each panel, and the panels only
use the same key for the same action.

Thus a global key cannot be a letter or a different printable symbol, because a text input needs
these keys. Only combinations such as `ctrl-…` and `alt-…` can be global.

**Sequences with time**

A sequence of keys with time is a part of the state of the panel. For example, a run of `tab` is
`{ count, last_at }` in the model of the panel. The time comes in `Msg::Key`, thus "is the pause
long enough" is a simple comparison in `update`. A timer is necessary only if something must occur
at the end of the pause, with no new key. Then a timer `Msg` does it, as in the chapter about the view.
