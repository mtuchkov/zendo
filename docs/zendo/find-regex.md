---
status: "draft"
revision: "1"
date: "2026-11-17"
parent: "zendo.md"
prev: "find-background.md"
next: "notes-repo.md"
level: "the regex button of the find panel and of the filter of the file tree"
note: "Agent draft of the design that the user decided on 2026-11-17. Not reviewed by the user."
---

# Find: regex

The find panel is in [Find](find.md), its modes in [Find: modes](find-modes.md).

The button `[.*]` stands at the end of the query line. A click on it or `alt-r` turns the regular
expression on or off. While it is on, the query is a regular expression (the syntax of ripgrep),
for the path and for the text, without regard to letter case; the button has the accent color.
While it is off, the query is plain words, as before. A query that starts with `#` or `?` keeps its
meaning. If the expression does not compile, then the button has the color of an error, and the
list keeps the results of the last valid query. The panel opens with the state of the last use.
The filter of the file tree has the same button (FILES-59).

```
 tab  Notes  Docs  Code  All
 > config.*(json|toml)                       [.*]
```

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FIND-42 | Event      | When the user clicks `[.*]` at the end of the query line or presses `alt-r` in the find panel, the system shall turn the regular expression on or off, and run the search again. | User, 2026-11-17 |
| FIND-43 | State      | While the regular expression is on, the system shall match the query as a regular expression in the path and in the text, without regard to letter case, except a query that starts with `#` or `?`. | User, 2026-11-17 |

### Acceptance tests

**`the_regex_button_turns_on_and_off`** (unit test; verifies FIND-42)

- Given a repository with `src/config.json` and `docs/config-toml.md`.
- When the user types `config.*json` in Code mode, presses `alt-r`, then clicks `[.*]`.
- Then the list is empty with the button off, shows `src/config.json` after `alt-r`, and is empty
  again after the click.

**`a_regex_matches_path_and_text`** (unit test; verifies FIND-43)

- Given `docs/a.md` with the line `Retry after 30 s`, and the button on.
- When the user types `retry.*[0-9]+ s` in Docs mode.
- Then the list shows `docs/a.md` with the line, and the match is marked.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FIND-44 | Unwanted   | If the regular expression does not compile, then the system shall show `[.*]` in the error color and keep the results of the last valid query. | Design |
| FIND-45 | Ubiquitous | The system shall keep the state of `[.*]` from one open of the panel to the next. | Design |

Tests that verify detailed requirements: `a_bad_regex_keeps_the_last_results` (FIND-44),
`the_regex_state_stays` (FIND-45).
