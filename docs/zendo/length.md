---
status: "draft"
revision: "1"
date: "2026-11-11"
parent: "zendo.md"
prev: "notes-repo.md"
next: "files.md"
level: "the word limit of a document and the help to split it"
note: "Agent draft of the design that the user decided on 2026-11-11. Not reviewed by the user."
---

# Length

A design document stays within about 1,000 words (see AGENTS.md). zendo shows where a document
passes its limit: from the row of the first word after the limit down, the whole width of the note
area has a slightly red background, also when the user scrolls. The left margin at that row says how
many words the document has, and that `S` asks Claude to split it.

To split, Claude gets the rules of "Keep the Altitude": keep one level of detail in the document,
move deeper content into child documents (`<name>/<name>.md`), split by topic into siblings, link each
document to its parent and children, keep every requirement and its ID, and stay within the limit.
The instructions are part of the zendo program. Claude proposes each file as an edit.

The limit is 1,000 words. `words = N` in an entry of `project-structure.toml` sets another limit for
the documents that the entry covers; the most specific entry wins; `words = 0` turns the limit off.
zendo counts the words of prose: not the title, the front matter, code or table borders.

## Architecturally significant requirements

| ID       | Pattern  | Requirement                                                                                                                                     | Source           |
| -------- | -------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| LENGTH-1 | State    | While a document has more words than its limit, the system shall give the whole note area a slightly red background from the row of the first word after the limit down. | User, 2026-11-11 |
| LENGTH-2 | Event    | When the user presses `S` on a document over its limit, the system shall ask Claude to split it with the rules of Keep the Altitude.          | User, 2026-11-11 |
| LENGTH-3 | Optional | Where an entry of `project-structure.toml` sets `words`, the system shall use that limit for the documents that the entry covers.             | User, 2026-11-11 |

### Acceptance tests

**`red_area_starts_at_the_word_after_the_limit`** (terminal check, not written yet; verifies LENGTH-1)

- Given a note of 1,320 words in a terminal of 150 × 40.
- When the user scrolls to the end.
- Then the rows from the row of word 1,001 down have the red background, and the rows above do not.

**`split_asks_claude_with_the_altitude_rules`** (mock agent, not written yet; verifies LENGTH-2)

- Given a note over its limit.
- When the user presses `S`.
- Then the agent gets the rules of Keep the Altitude, the file name, its words and its limit.

**`words_from_the_most_specific_entry`** (unit test, not written yet; verifies LENGTH-3)

- Given `words = 2000` for `docs` and `words = 0` for `docs/archive`.
- When zendo finds the limits of `docs/a.md` and `docs/archive/b.md`.
- Then the limits are 2,000 and none.

## Detailed requirements

| ID       | Pattern    | Requirement                                                                                                          | Source           |
| -------- | ---------- | -------------------------------------------------------------------------------------------------------------------- | ---------------- |
| LENGTH-4 | Ubiquitous | The system shall count the words of prose only: not the title, the front matter, code or table borders.            | Design           |
| LENGTH-5 | Ubiquitous | The system shall show the red area and the hint in write mode too.                                                   | Design           |
| LENGTH-6 | Unwanted   | If the left margin is too narrow for the hint, then the system shall show the hint in the status bar only.         | Design           |
