---
status: "implemented"
approved: "2026-11-17"
revision: "1"
date: "2026-11-17"
parent: "zendo.md"
prev: "find.md"
next: "find-ids.md"
level: "the modes of the find panel and the global `/` key"
note: "Design that the user decided and approved on 2026-11-17. A design for the search in the text of code and for an index comes later, in its own document."
---

# Find: modes and the `/` key

The find panel is in [Find](find.md). This document adds the modes of the panel and makes `/` a
global key.

## Modes

The panel has four modes. Each mode sets what the panel looks for:

- **Notes**: the notes, by path and by text. A query that starts with `?` asks the agent about all
  notes ([Notes list and search](notes-list.md)). With an empty query, the list shows the notes by
  age, as the notes list does.
- **Docs**: the markdown files of the repository that are not notes, by path and by text, as
  [Find](find.md) states. With an empty query, the list shows the recent ones of them.
- **Code**: the text files that are not markdown, by path only. With an empty query, the list shows
  the recent ones of them.
- **All**: the merge of the results of Notes, Docs and Code ([Find: search in the
  background](find-background.md)). With an empty query, the list shows all recent files.

A query that starts with `#` finds a tag or a requirement ID: see [Find: tags and requirement IDs](find-ids.md).

`tab` moves to the next mode, and `shift-tab` to the previous mode. The query stays, and the list
shows the search of the new mode. The panel always opens in Docs mode.

Design: the modes stand in one row above the query field, after the key hint `tab` in dim colors.
The active mode has the colors of the active agent session button (the accent color); the other
modes have the colors of the other session buttons (the second accent color).

```
 tab  Notes  Docs  Code  All
 > query_                                    [.*]
```

The button `[.*]` turns the regular expression on: see [Find: regex](find-regex.md).

A full search in the text of code and an index of the code are not part of this document.

## The `/` key

`/` opens the find panel in every pane and mode where the focus does not take text. Where the focus
takes text, `/` types: in write mode, in the chat input (slash commands, CLAUDE-28), in the float,
in the query of the find panel, in each question field (rename, move, new file), and in a Claude
Code session or a terminal on the full screen. `alt-/` opens the panel there (FIND-1).

`/` in the file tree and in the notes list opens the panel too. The filter of the tree by file name
moves to `` ` `` ([File tree: actions](files/files-actions.md)).

## Architecturally significant requirements

| ID      | Pattern | Requirement | Source |
| ------- | ------- | ----------- | ------ |
| FIND-21 | Event   | When the user presses `tab` in the find panel, the system shall switch to the next mode in the order Notes, Docs, Code, All, keep the query, and show the search of the new mode. | User, 2026-11-17 |
| FIND-22 | Event   | When the user presses `/` in a pane or mode where the focus does not take text, the system shall open the find panel. | User, 2026-11-17 |
| FIND-23 | State   | While the focus takes text, or a Claude Code session or a terminal is on the full screen, the system shall send `/` to the field or to the program and not open the find panel. | User, 2026-11-17 |

### Acceptance tests

**`tab_switches_the_mode_and_keeps_the_query`** (unit test; verifies FIND-21)

- Given a repository with `docs/find.md`, `src/find.rs` and the note `docs/notes/find-idea.md`.
- When the user presses `alt-/`, types `find`, and presses `tab` three times.
- Then the list shows `docs/find.md` and not the note in Docs mode, `src/find.rs` in Code mode,
  all three files in All mode, and `docs/notes/find-idea.md` in Notes mode after one more `tab`,
  and the query is `find` each time.

**`slash_opens_find_where_no_text_is_typed`** (unit test; verifies FIND-22, FIND-23)

- Given a repository with notes.
- When the user presses `/` in read mode, in the file tree, in the notes list, in the delta and in
  the transcript; then in write mode, in the chat input and in the rename field.
- Then the find panel opens in each of the first five; in the last three `/` is typed and the panel
  does not open.

**`slash_goes_to_a_full_screen_session`** (integration test on a test terminal; verifies FIND-23)

- Given a Claude Code session on the full screen.
- When the user presses `/`.
- Then the session gets `/`, and the find panel does not open.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FIND-24 | Event      | When the user presses `shift-tab` in the find panel, the system shall switch to the previous mode. | User, 2026-11-17 |
| FIND-25 | Ubiquitous | The system shall show the modes Notes, Docs, Code and All in one row above the query field, after the dim key hint `tab`. | User, 2026-11-17 |
| FIND-26 | Ubiquitous | The system shall show the active mode in the colors of the active agent session button, and the other modes in the colors of the other session buttons. | User, 2026-11-17 |
| FIND-27 | Event      | When the find panel opens, the system shall show it in Docs mode. | User, 2026-11-17 |
| FIND-28 | State      | While the panel is in Docs mode, the system shall search the markdown files of the repository that are not notes, by path and by text (FIND-9). | User, 2026-11-17 |
| FIND-29 | State      | While the panel is in Code mode, the system shall search the text files that are not markdown by path only, and list the recent ones of them for an empty query. | User, 2026-11-17 |
| FIND-30 | State      | While the panel is in Notes mode, the system shall search the notes by path and by text, with the notes of the notes repo (FIND-18), ask the agent for `?` (NOTE-12, NOTE-14), and list the notes by age for an empty query. | User, 2026-11-17 |
| FIND-31 | State      | While the panel is in All mode, the system shall show the merge of the results of Notes, Docs and Code (FIND-42), and list all recent files for an empty query. | User, 2026-11-17 |

Tests that verify detailed requirements: `shift_tab_goes_to_the_previous_mode` (FIND-24),
`the_modes_stand_above_the_query` (FIND-25, FIND-26), `find_opens_in_docs_mode` (FIND-27),
`docs_mode_has_no_notes_and_no_code` (FIND-28), `code_mode_finds_by_path_only` (FIND-29),
`notes_mode_keeps_questions` (FIND-30), `all_mode_finds_every_kind` (FIND-31).
