---
status: "draft"
revision: "1"
date: "2026-11-12"
parent: "zendo.md"
prev: "terminal-output.md"
next: "project-structure.md"
level: "how zendo calls the user back when a turn of an agent or a command of a terminal ends: the bell, the blinking button, the sound button"
note: "Agent draft from the requests of the user on 2026-11-12. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. It answers the open point on the bell in sessions.md."
---

# Attention

When a long task of an agent session or a terminal stops while the user is elsewhere, the terminal
bell rings once, and the button of the session blinks until the user looks at it.

**What stops.** A *turn* ends in three cases:

- The answer of the agent in the chat of an agent session ends (see [Claude](claude.md)).
- A Claude Code session goes from *works* to *idle* or *waits* (see [Sessions](sessions.md)). Only
  a state that a hook of Claude Code reports counts: the start of Claude Code and quiet output do
  not ring.
- A command in a terminal ends, and the shell shows its prompt again (see [Terminal](terminal.md)).

An answer that the user stopped, an answer with a question in the queue after it, and the end of
a process do not count.

**Seen.** The user *sees* a session while its view, or the chat on that agent session, is on the
screen and the terminal has the focus (from the focus events of the terminal; without them, always).
A turn that the user sees rings nothing and blinks nothing.

**The blink.** The button fades to the accent color and back each 2 s; a button in the accent
color fades to the second accent. The buttons on the right edge of the chat and in the bottom line of
the session view blink. While they are not on the screen, the top line shows each session that
blinks, left of the sound button. The blink stops when the user sees the session.

**The sound button.** `[sound]` is left of the name of the git branch in the top line. A click turns
the bell off (the button is struck through) or on, and saves `sound` in `~/.zendo/config.toml`. The
sound is the terminal bell: it works over SSH and in tmux, and the terminal decides what the user
hears (Design).

## Architecturally significant requirements

| ID     | Pattern  | Requirement | Source |
| ------ | -------- | ----------- | ------ |
| ATTN-1 | Event    | When a turn ends while the user does not see its session, the system shall ring the terminal bell once, where the sound is on. | User, 2026-11-12 |
| ATTN-2 | Event    | When a turn ends while the user does not see its session, the system shall blink the button of that session. | User, 2026-11-12 |
| ATTN-3 | State    | While a button blinks, the system shall change its background color smoothly from its color to the accent color and back, with a period of 2 s and at least 20 shades in each period. | User, 2026-11-12 |
| ATTN-7 | Ubiquitous | The system shall show the sound button `[sound]` left of the name of the git branch in the top line, struck through while the sound is off. | User, 2026-11-12 |
| ATTN-8 | Event    | When the user clicks the sound button, the system shall turn the sound off or on and save the setting `sound` in `~/.zendo/config.toml`. | User, 2026-11-12 |

### Acceptance tests

The tests are in `tests/attention.rs`, with the mock command line of [Sessions](sessions.md) and
`/bin/sh`.

**`a_claude_code_turn_rings_and_blinks_while_the_user_is_away`** (mock command line; verifies ATTN-1, ATTN-2, ATTN-4, ATTN-6)

- Given a Claude Code session of agent session 1, and the chat on agent session 2.
- When Claude Code starts, and then its hooks report *works* and *idle*.
- Then the start rings nothing, the turn rings once and blinks `F6`, and the chat of agent session 1
  stops the blink.

**`a_turn_on_the_screen_rings_only_without_the_focus`** (mock command line; verifies ATTN-1, ATTN-2, ATTN-4, ATTN-11)

- Given the session view on the screen.
- When a turn ends with the focus, a turn ends without it, and the focus comes back.
- Then only the second turn rings and blinks, and the focus stops the blink.

**`the_end_of_a_terminal_command_rings_and_blinks`** (terminal check; verifies ATTN-1, ATTN-2)

- Given the `F5` terminal that runs `sleep 1`, and the user back in zendo.
- When the command ends.
- Then the bell rings once and `F5` blinks.

**`the_end_of_a_chat_answer_rings_and_blinks`** (unit test of the events; verifies ATTN-1, ATTN-2, ATTN-5)

- Given the chat on agent session 1.
- When answers of agent session 3 end, stopped and normal, and one of agent session 1 ends.
- Then only the normal end in session 3 rings and blinks `F8`.

**`a_blinking_button_fades_to_the_accent_and_back`** (unit test of the frame; verifies ATTN-3)

- Given the `F7` button in the second accent color.
- When it blinks, with a frame each 100 ms.
- Then each frame has a new shade: the accent after 1 s, its own color after 2 s.

**`the_sound_button_turns_the_bell_off_and_saves_it`** (unit test of the frame; verifies ATTN-7, ATTN-8, ATTN-1)

- Given the branch `main` in the top line.
- When the user clicks `[sound]`, a turn ends, and the user clicks again.
- Then `[sound]` is left of `main`; after the first click it is struck through, the config has
  `sound = false`, and the turn only blinks; after the second, `sound = true`.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| ATTN-4  | Event      | When the user sees a session whose button blinks, the system shall stop the blink within 0.5 s. | Design |
| ATTN-5  | Unwanted   | If the user stopped the answer, or a question waits in the queue after it, then the system shall not ring and shall not blink. | Design |
| ATTN-6  | Unwanted   | If no hook of Claude Code reported the change of the state, then the system shall not take it as the end of a turn. | Design |
| ATTN-9  | Optional   | Where `~/.zendo/config.toml` has `sound = false`, the system shall start with the sound off; without the setting, with the sound on. | User, 2026-11-12 |
| ATTN-10 | State      | While the session buttons are not on the screen, the system shall show each session that blinks in the top line, left of the sound button, and open it on a click. | Design |
| ATTN-11 | Ubiquitous | The system shall take the focus from the focus events of the terminal, and take a terminal without focus events as one that has the focus. | Design |

Tests: `a_blinking_session_shows_in_the_top_line_while_the_chat_is_closed` (ATTN-10),
`the_config_sets_the_sound` (ATTN-9), `sound_goes_into_the_config_and_back` (ATTN-8, ATTN-9), and the
tests of `pulse` and `mix` in `src/ui/theme_tests.rs` (ATTN-3).

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| zendo reads the state of a terminal each 0.25 s, so a command that ends sooner may not ring. Is that acceptable? | ATTN-1 | User |
| The sound button has no key yet. Should a key of the keymap turn the sound off and on? | ATTN-8 | User |
