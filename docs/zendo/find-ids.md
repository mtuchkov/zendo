---
status: "draft"
revision: "1"
date: "2026-11-17"
parent: "zendo.md"
prev: "find-modes.md"
next: "find-background.md"
level: "a `#` query in the find panel: tags and requirement IDs"
note: "Agent draft of the design that the user decided on 2026-11-17. Not reviewed by the user."
---

# Find: tags and requirement IDs

The find panel is in [Find](find.md), its modes in [Find: modes](find-modes.md).

A query that starts with `#` finds a tag or a requirement ID, in every mode:

- When the word after `#` has the form of a requirement ID, for example `#FIND-9`, the list shows
  the files of the mode with an ID that starts with the word: `#FIND-1` finds `FIND-1` and `FIND-12`,
  not `XFIND-1`. Files with the whole ID come first. In Code mode this is the only search in the
  text of code. Each row shows up to two lines with the ID.
- Otherwise the list shows the notes and the docs of the mode with a tag that starts with the word
  (NOTE-13). Code has no tags.

Design: the form of a requirement ID is a regular expression, `[A-Z][A-Z0-9]*-[0-9]+` by default.
The key `requirement_id` in the `[project]` table of `project-structure.toml` changes it for the
repository. If the expression does not compile, then zendo uses the default and `zendo doctor`
shows the problem.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FIND-30 | Event      | When the query starts with `#` and the word after it matches the requirement ID pattern, the system shall list the files of the mode with an ID that starts with the word, the files with the whole ID first, with up to two lines that show it. | User, 2026-11-17 |
| FIND-32 | Optional   | Where `project-structure.toml` sets `requirement_id` in `[project]`, the system shall use that regular expression as the requirement ID pattern, and `[A-Z][A-Z0-9]*-[0-9]+` otherwise. | User, 2026-11-17 |

### Acceptance tests

**`hash_finds_a_requirement_id_in_code_and_docs`** (unit test; verifies FIND-30)

- Given `src/find.rs` with the comment `(FIND-9, FIND-19)`, `docs/find.md` with the row `FIND-9`,
  and `src/other.rs` with `FIND-19` only.
- When the user types `#FIND-9` in Code mode, then `#FIND-1` in Code mode, then `#FIND-9` in Docs
  mode.
- Then `#FIND-9` in Code mode lists `src/find.rs` with the line of the comment, and not
  `src/other.rs`; `#FIND-1` lists `src/find.rs` and `src/other.rs`; Docs mode lists `docs/find.md`.

**`the_requirement_id_pattern_is_configurable`** (unit test; verifies FIND-32)

- Given `project-structure.toml` with `requirement_id = "REQ_[0-9]+"` and `src/a.rs` with `REQ_7`.
- When the user types `#REQ_7` in Code mode.
- Then the list shows `src/a.rs`.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FIND-31 | Event      | When the query starts with `#` and the word after it does not match the requirement ID pattern, the system shall list the notes and the docs of the mode with a tag that starts with the word (NOTE-13). | User, 2026-11-17 |
| FIND-33 | Unwanted   | If `requirement_id` is not a valid regular expression, then the system shall use the default pattern, and `zendo doctor` shall report the problem. | Design |

Tests that verify detailed requirements: `hash_finds_a_tag_in_docs` (FIND-31),
`a_bad_requirement_id_pattern_falls_back` (FIND-33).
