---
status: "approved"
approved: "2026-11-17"
date: "2026-11-17"
sources:
  - "The user, 2026-11-17: the search shall not block the UI thread; files change often, so no cache between searches; measure first; use ripgrep as a crate."
  - "../../zendo/find-background.md (FIND-36 to FIND-43)"
  - "ADR 0009 (git and agent sessions without UI stalls)"
---

# 0017 The find search uses the ripgrep crates off the UI thread

## Context

`App::run_find` searches on the UI thread on each key. The first query walks the repository and
loads every doc; each query then runs `docs::search::search` over the text of every doc. The cache
of files is filled once and never loaded again, so a new file is not found.

Measurements on 2026-11-17: release build, 2 cores, files in the page cache, best of 3. Corpora:
code10k (10,000 files of vscode, 160 MB, 160 markdown), docs10k (10,000 files of mdn, 55 MB, 9,300
markdown), vscode (21,600 files, 360 MB).

| Measure | code10k | docs10k | vscode |
| ------- | ------- | ------- | ------ |
| One key now, first key (walk and load) | 206 ms | 1,090 ms | 550 ms |
| One key now, later keys | 45–100 ms | 720–870 ms | 115–250 ms |
| `docs::search::search` alone | 40–140 ms | 750–890 ms | — |
| Walk of the repository (`ignore`) | 40 ms | 95 ms | 65 ms |
| Read every file and `memmem` | 58 ms | 40 ms | 130 ms |
| `rg -i -l` (the binary) | 81–95 ms | 98–122 ms | 138–174 ms |
| `grep-searcher` in the process, 2 threads | 64–80 ms | 62–68 ms | 125–145 ms |
| The same, query of one letter | 183 ms | 97 ms | 360 ms |

A walk on a cold disk of this VM took 1.7–2.7 s; a real SSD is faster, and the repository that the
user works in is usually in the page cache. `mmap` was slower than `read` for many small files.
`docs::search::search` is slow because `find_all` builds a lowercase copy and a map of offsets of
each line for each query.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| A copy of the text in memory for the time the panel is open | Keys in a few ms. | 55–360 MB of memory; own search code; the text is old while the panel is open. |
| The `rg` binary for each query | The fastest search there is. | The user must install `rg`; a process and JSON output for each query. |
| The ripgrep crates (`grep-searcher`, `grep-regex`) on a thread, for each query | Speed of `rg`; no copy of the text; always the files on the disk; `ignore` is already a dependency. | 60–145 ms for each query; a thread and a stop of the old search. |
| A trigram index | Fastest in a very large repository. | Must follow each change of each file; not needed at 10,000 files. |

## Decision

Accepted by the user on 2026-11-17:

- zendo shall use the ripgrep crates `grep-searcher` and `grep-regex` for the search in the text,
  case-insensitive, Unicode on. `ignore` walks the repository, as before.
- The find panel shall have one worker thread (`src/docs/grep.rs`). On the open of the panel it
  walks the repository and sends the list of files; the UI thread keeps the list until the panel
  closes and searches by path in it on each key.
- On each change of the query, the UI thread sends the query with a number. The worker waits 100 ms
  for a newer query, then searches the files of the list on the disk: the markdown files, and the
  code too for a requirement ID. The threads of the search take the next file of the list, so that
  a folder of large files does not load one thread. The worker checks the number between files and
  stops an old search; the UI thread drops results with an old number.
- The UI thread sorts each file into Notes, Docs or Code by its path, and makes the list of All
  from the same rows, in one sort order.
- The worker wakes the event loop when it sends results; the tick of zendo is 250 ms.
- `docs::search::search`, its cache of docs, and the search mode of the notes list go away.

## Consequences

- Two new dependencies, from the ripgrep project. No copy of the text stays in memory; only the
  list of paths, while the panel is open.
- A query of one letter matches nearly every line, and takes up to 360 ms in vscode. The debounce
  hides it while the user types; the list of matches by path shows at once.
- On a cold disk the first walk takes seconds. The list fills when the walk ends; the UI does not
  wait.
- A later index can stand behind the same worker without a change of the UI side.

## Verification

- The tests of `src/docs/grep_tests.rs`, `tests/find_search.rs` and `tests/find_modes.rs`:
  `typing_does_not_wait_for_the_search`, `a_late_result_of_an_old_query_does_not_show`,
  `one_search_fills_every_mode`, `a_changed_file_is_found_without_a_new_open`.
- A benchmark of the implementation on 2026-11-17, release build, 2 cores, files in the page cache:

| Measure | code10k | docs10k | vscode |
| ------- | ------- | ------- | ------ |
| Walk when the panel opens, off the UI thread | 59 ms | 113–152 ms | 65 ms |
| A key and its draw in the panel | 1–7 ms | 1–18 ms | 1–16 ms |
| Results of `config` after the last key, with the 100 ms wait | 111 ms | 133–138 ms | 129 ms |
| Results of `#FIND-1` (the text of code) after the last key | 152 ms | 141–195 ms | 212 ms |

The first draw of the preview of a markdown file in a run takes up to 70 ms, once; that is the
preview, not the search.
