---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "text-input.md"
prev: "text-formatting.md"
level: "the date selector of the rich editors: how it starts, its keys, how it ends"
note: "Agent draft from the requests of the user on 2026-11-17; split from text-input.md with the approval of the user on 2026-11-18, no requirement changed. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Date selector

The date selector is an element of a rich editor of [Text input](text-input.md).

**Dates.** `//` at the start of a word in a rich editor puts the date of today, in the format of
the locale of the system, and starts the date selector on its first part. While the selector is on,
its keys come before the keys of the pane: up and `k` make the selected part greater, down and `j`
smaller, left and `h`, right and `l` go to the other parts. Left on the first part, right on the last
part, `enter` and `esc` leave the selector; another key leaves it and does what it does in the
editor. In code, `//` is text.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TEXT-15 | Event      | When the user types `//` at the start of a word outside code in a rich editor, the system shall put the date of today in the format of the locale of the system, and start the date selector on its first part. | User, 2026-11-17 |
| TEXT-16 | State      | While the date selector is on, the system shall make the selected part greater with up or `k`, smaller with down or `j`, and select the part before or after with left or `h`, right or `l`. | User, 2026-11-17 |
| TEXT-17 | Event      | When the user presses left on the first part, right on the last part, `enter` or `esc` in the date selector, the system shall end the selector and give the keys back to the pane. | User, 2026-11-17 |

### Acceptance tests

**`slashes_open_the_date_selector_in_write_mode`** (unit test; verifies TEXT-15, TEXT-16, TEXT-17)

- Given write mode at an empty line.
- When the user types `due //`, then `k` and `j`, then `enter` and ` x`.
- Then the line holds `due`, the date of today in the format of the locale, and `x`.

**`enter_in_the_chat_date_selector_does_not_send`** (unit test; verifies TEXT-17)

- Given the chat input with `//` typed and the year one up.
- When the user presses `enter`.
- Then the selector ends and the input still holds the date.

Tests that verify detailed requirements: the tests of `src/text/editor/date_tests.rs` (TEXT-15,
TEXT-16, TEXT-17).
