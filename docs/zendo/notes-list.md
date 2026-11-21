---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "say.md"
next: "note-to-spec.md"
level: "notes: titles, tags and links, the notes list, search"
note: "Draft, split from notes.md. Not reviewed by the user. 2026-11-18: the notes list moves into the tree as the Notes view (files-view-column.md); NOTE-15 retired."
---

# Notes list and search

A note is a markdown file in the notes folder, `docs/notes` by default. The first `# ` line is the
title. Words like `#word` are tags. `[[name]]` is a link to another note. A new note gets a file name
from the date and a short form of the title. The note pane is in [Notes](notes.md).

The notes list is the Notes view of the file tree ([The view strip and the Notes view](files/files-view-column.md)):
`o` opens it. It shows the notes by age: today, this week, older. `enter` opens a note, `n` writes a
new one in a [float](float.md), `f2` renames it to a typed name, and `delete` deletes it after a
`y`. `/` opens the find panel ([Find: modes](find-modes.md)); its Notes mode searches all notes; a search that starts with `#`
finds a tag, and one that starts with `?` asks the agent about all notes.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| NOTE-7  | Event      | When the user saves a new note, the system shall name the file with the date and a short form of the title.                         | Design           |
| NOTE-8  | Unwanted   | If the notes folder does not exist, then the system shall make it only when the user saves the first note.                          | Design           |
| NOTE-10 | Ubiquitous | The system shall use the first `# ` line of a note as its title, and `#word` tokens as its tags.                                    | Design           |
| NOTE-11 | Ubiquitous | The system shall treat `[[name]]` as a link to the note whose file name ends with `name`.                                           | Design           |
| NOTE-12 | Event      | When the user searches, the system shall find the text in all notes, without regard to letter case.                                 | Design           |
| NOTE-13 | Event      | When the search starts with `#`, the system shall find the notes with that tag.                                                     | Design           |
| NOTE-14 | Event      | When the search starts with `?`, the system shall ask Claude the question about all notes.                                          | Design           |
| NOTE-15 | —          | Retired on 2026-11-18: the Notes view has no rename to the title (FILES-69).                                                          | User, 2026-11-18 |
| NOTE-24 | Event      | When the user presses `f2` or `delete` on a note in the Notes view of the tree, the system shall rename or delete the file as in the file tree. | User, 2026-11-12 |

Tests that verify detailed requirements: `title_and_tags` (NOTE-10), `slug_is_short_and_clean`
(NOTE-7), `a_missing_notes_folder_is_not_created_on_load` (NOTE-8),
`finds_case_insensitive_with_ranges` (NOTE-12), `tag_filter` (NOTE-13).
