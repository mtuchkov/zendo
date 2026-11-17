---
status: "draft"
revision: "1"
date: "2026-11-17"
parent: "zendo.md"
prev: "float.md"
next: "find.md"
level: "the title of a new note in the float: when the agent writes it, where it goes, what it does not change"
note: "Agent draft of the request of the user on 2026-11-17. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Title of a new note

The user starts a new note with the text, not with the title: the title line is empty, and the
cursor is on the body (see [New note in a float](float.md)).
The name model of the agent writes the title: `haiku` for Claude, or the model of `[name_models]`
in the config (see [Session names](session-names/session-names.md)). It is cheap and fast, and it
works in a background session without tools, so the chat does not see it.

**When.** The name model writes the title when the body of the note has three lines with text, or
200 chars of text. It writes it again when the float closes, if the title is still the one of the
agent and the text changed since. When the float closes and the note has no title yet, it writes
one then.

**Where.** zendo puts the title into the empty title line, `# <title>`. When the user took the
title line away, zendo puts it as the first line, with an empty line under it. The cursor stays on
the same char of the text, so the user types on. When the float is closed
already, zendo puts the title into the file. Then zendo renames the file to the date and a short
form of the title, as for a new note (NOTE-7).

**What zendo does not change.** A title that the user writes or changes is the title of the user.
zendo does not replace it, and does not ask for another.

## Architecturally significant requirements

| ID       | Pattern | Requirement                                                                                                                                                                                       | Source           |
| -------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| FLOAT-18 | Event   | When the body of the note in the float has three lines with text, or 200 chars of text, and the title of the note is empty, the system shall ask the name model for a title in a background session without tools, put it into the title line, `# <title>`, and keep the cursor on the same char. | User, 2026-11-17 |
| FLOAT-19 | Event   | When the user closes the float, and the title of the note is empty, or the note has the title of the agent unchanged and a text that changed since that title, the system shall ask the name model for the title and put it into the file. | User, 2026-11-17 |

### Acceptance tests

**`the_name_model_titles_the_note_after_three_lines`** (mock agent; verifies FLOAT-18, FLOAT-20)

- Given the float on a new note.
- When the user types three lines of text.
- Then the title line has the title of the mock agent, the cursor is on the same char of the third
  line, and the file name is the date and the short form of the title.

**`closing_the_float_titles_or_updates_the_note`** (mock agent; verifies FLOAT-19, FLOAT-21)

- Given three floats: one with one line of text, one with a title of the agent and more text after
  it, and one with a title that the user typed.
- When the user closes each float.
- Then the first note gets a title, the second gets the title of the agent again, and the third
  keeps the title of the user.

## Detailed requirements

| ID       | Pattern  | Requirement                                                                                                                                      | Source           |
| -------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------- |
| FLOAT-20 | Event    | When a title of the agent goes into the note of a float, the system shall rename the file to the date and a short form of the title (NOTE-7).  | User, 2026-11-17 |
| FLOAT-21 | Unwanted | If the user writes or changes the `# ` title of the note, then the system shall not change the title.                                           | User, 2026-11-17 |
| FLOAT-23 | Ubiquitous | The system shall ask the name model for the title of a note at most once while the float is open, and once when it closes.                   | Design           |

Tests that verify detailed requirements: `a_title_goes_in_only_where_the_user_has_none` (FLOAT-19,
FLOAT-21), `the_name_model_titles_the_note_after_three_lines` (FLOAT-20, FLOAT-23).
