---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "../internals.md"
prev: "../file-writes/file-writes.md"
children:
  - "document-edits.md"
  - "annotations.md"
level: "the layers of a document and the part that owns each layer: the text and the geometry in the model, the decoration in the cache of the view"
note: "Agent draft from the chapter The document of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# The document

Sources: the chapter
[The document: text, geometry and decoration](../../../book/15-document.md) of the zendo book.

Parent: [Internals of the TUI](../internals.md). Children:
[Open, edit and parse](document-edits.md) and [Annotations](annotations.md). What the user sees of
a document is in [Notes](../../notes.md), [Write mode](../../write.md) and
[Code files](../../files/files-code.md).

## The layers

A document has four layers. The examples use a markdown file; the same rules apply to a code file.

1. **The text**: the buffer of the editor and its version. This is the truth.
2. **The structure**: the markdown blocks that are parsed from the text.
3. **The lines**: the blocks after the wrap to the width of the column. The inline markup is also
   here: the view does not show `**` and `` ` ``, so the markup changes the width of a line.
4. **The decoration**: the highlight of code, the colors of inline styles, the theme.

## Geometry and decoration

`update` needs the lines. `j` and `k` in the reader move the cursor by rows on the screen, not by
lines of the text. To keep the cursor visible, `update` must know how many rows a block has after
the wrap. A click at a row and a column must become a position in the text. A selection goes by
rows on the screen. If the lines are in an asynchronous cache, `update` can find no lines.

So the layers have two owners:

- **The geometry (layers 2 and 3) is in the model**, in the document
  [component](../components.md). It is derived data, but `update` calculates it itself,
  synchronously: after each edit and after a resize.
- **The decoration (layer 4) is in the cache of the view** ([VIEW-4](../view.md)), and it comes
  asynchronously. While there is no highlight, the view shows plain text.

The decoration does not change the geometry, as an animation does not ([VIEW-5](../view.md)). The
highlight only gives colors to symbols that are already in their positions. When it comes, nothing
jumps: not the rows, not the cursor, not the hit map.

The cursor is a position in the text, not a row. When the width changes, the wrap changes, and the
cursor stays on the same symbol.

**A code file** has the same layers. The structure is the tokens. A block is a line. Usually code
does not wrap: one line of the text is one row on the screen.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| DOC-1 | Ubiquitous | The text of a document and its version shall be in the model, and only `update` shall change them. | User, 2026-11-20 |
| DOC-2 | Ubiquitous | The geometry of a document (its structure and its lines after the wrap) shall be in the model of the document component, and `update` shall calculate it synchronously after each edit and after each resize. | User, 2026-11-20 |
| DOC-3 | Ubiquitous | The decoration of a document shall be in the cache of the view, shall come asynchronously, and shall not change the geometry. | User, 2026-11-20 |
| DOC-4 | Ubiquitous | The cursor of a document shall be a position in the text, not a row of the screen. | User, 2026-11-20 |

### Acceptance tests

**`an_edit_changes_the_text_and_its_version`** (unit test, not written yet; verifies DOC-1)

- Given a model with a note in write mode.
- When the message of a typed symbol comes, and then the view draws two times.
- Then the text has the symbol and a new version after the message, and the draws change neither.

**`update_moves_the_cursor_by_screen_rows`** (unit test, not written yet; verifies DOC-2)

- Given a model with a paragraph that wraps to three rows, the cursor on its first row, an empty
  cache, and no frame drawn.
- When the message of the key `j` comes.
- Then the cursor is on the second row of the same paragraph.

**`the_highlight_moves_nothing`** (unit test, not written yet; verifies DOC-3)

- Given a model with a note that has a code block, and no decoration in the cache.
- When the view draws, the decoration comes, and the view draws again.
- Then the symbols, the cursor and the hit map of the two frames are equal, and only styles differ.

**`the_cursor_stays_on_its_symbol_after_a_resize`** (unit test, not written yet; verifies DOC-4)

- Given a model with the cursor on a symbol in the third row of a wrapped paragraph.
- When the message of a narrower screen comes.
- Then the cursor is on the same symbol, in the row that the new wrap gives.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| DOC-5 | Ubiquitous | The lines of the geometry shall hold the inline markup, so that the width of a line is the width that the view shows. | User, 2026-11-20 |
| DOC-6 | State | While the decoration of a document is not in the cache, the view shall show the plain text. | User, 2026-11-20 |
| DOC-7 | Ubiquitous | `update` shall calculate the geometry incrementally: it shall parse and wrap again only the changed blocks. | User, 2026-11-20 |
| DOC-8 | Ubiquitous | The same rules shall hold for a code file, where the structure is the tokens and a block is a line. | User, 2026-11-20 |
