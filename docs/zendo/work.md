---
status: "draft"
revision: "1"
date: "2026-11-11"
parent: "zendo.md"
prev: "files.md"
next: "claude.md"
level: "visits that count as work, the delta pane, recaps"
note: "Agent rewrite of the reference of zendo 0.9.1. Not reviewed by the user."
---

# Work, delta and recaps

zendo tells a short look at a file from real work on it. A visit counts as work only when the user
edits the file, or scrolls and clicks in it many times. When such a visit ends, zendo keeps the text
of the file as the base for the delta.

The delta pane shows what changed in the open file, in the style of git-delta: line numbers, and the
changed words marked inside the changed lines. The base is the last visit that counted, or the git
`HEAD`. The delta notices changes from outside zendo, for example from Claude Code, git or another
editor.

A recap is a short summary of a file by Claude. The user asks for one, or zendo writes recaps itself
while Claude and the user are idle. Recaps never go into the chat. A recap says what the file is
about as a whole, and what its changes against git `HEAD` do; when there are none, what the last
commit of the file did. It has no open questions.

A review is on request: `ctrl-r` twice. Claude reads the file and finds the lines that a reader
finds confusing. The summary and each line with why show at the top of the file, under the recap,
and the lines show marked in read mode and write mode until the user opens another file. zendo
does not keep a review.

Keys: `d` in read mode or `ctrl-d` in write mode opens the delta; in the delta, `b` switches the
base and `a` asks Claude about the changes. `r` asks for a recap now; `R` shows or hides the recap; `ctrl-r` twice asks for a review.

## Architecturally significant requirements

| ID     | Pattern    | Requirement                                                                                                                                                                      | Source           |
| ------ | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| WORK-1 | Ubiquitous | The system shall count a visit to a file as work only when the user edits the file, or scrolls and clicks in it more times than a set number. A short open shall not count.     | User, 2026-11-11 |
| WORK-2 | State      | While Claude is idle and the user does not use zendo for a set time, the system shall write a recap of each file that the user worked on recently and that changed since its last recap. | User, 2026-11-11 |
| WORK-3 | Ubiquitous | The system shall write each recap in a separate Claude session without tools and without edits, and shall not show it in the chat.                                                | Design           |

### Acceptance tests

**`brief_open_does_not_count_but_work_does`** (unit test; verifies WORK-1)

- Given a test repository with a note.
- When the user opens the note and leaves it at once, and later opens it, edits it and leaves it.
- Then only the second visit counts, and the delta base is the text of the second visit.

**`recaps_on_request_and_when_idle`** (mock agent; verifies WORK-2, WORK-3)

- Given two files that the user worked on and that changed.
- When the user asks for a recap of one file, and later Claude and the user are idle.
- Then each file gets a recap from a separate session, and the chat has no recap.

**`no_auto_recap_while_the_user_is_active`** (mock agent; verifies WORK-2)

- Given a file that the user worked on and that changed.
- When the user keeps pressing keys.
- Then zendo writes no recap.

## Detailed requirements

| ID      | Pattern    | Requirement                                                                                                                       | Source           |
| ------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| WORK-4  | Event      | When a visit that counts ends, the system shall keep the text of the file as the base for the delta.                              | User, 2026-11-11 |
| WORK-5  | Ubiquitous | The system shall show the delta with line numbers, and mark the changed words in the changed lines.                               | User, 2026-11-11 |
| WORK-6  | Event      | When the user presses `b` in the delta, the system shall switch the base between the last visit that counted and the git `HEAD`. | Design           |
| WORK-7  | Event      | When a file changes outside zendo, the system shall show the change in the note and the delta without a restart.                   | Design           |
| WORK-8  | Event      | When the user presses `a` in the delta and sends an empty question, the system shall ask Claude a default question about the changes. | Design      |
| WORK-9  | Event      | When the user presses `r`, the system shall ask Claude for a recap of the open file.                          | User, 2026-11-11 |
| WORK-10 | Ubiquitous | The system shall write the recaps of the most recently worked files first, and only a set number of recaps in each hour.          | Design           |
| WORK-11 | Event      | When the user presses `R`, the system shall show or hide the recap above the note.                                                | Design           |
| WORK-12 | Ubiquitous | The system shall have each recap say what the file is about, and what its changes against git `HEAD` do, or else what the last commit of the file did, without open questions. | User, 2026-11-12 |
| WORK-13 | Event      | When the user presses `ctrl-r` twice in read mode or write mode, the system shall ask Claude for the confusing lines of the file, show the summary at the top, and mark the lines until the user opens another file, without keeping the review. | User, 2026-11-12 |

Tests that verify detailed requirements: `recaps_on_request_and_when_idle` (WORK-12),
`ctrl_r_twice_marks_the_confusing_lines` and `a_review_has_a_summary_and_lines` (WORK-13),
`changed_words_are_marked` (WORK-5),
`counts_and_numbers` (WORK-5).
