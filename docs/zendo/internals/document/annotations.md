---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "document.md"
prev: "document-edits.md"
level: "data that looks like decoration but changes the geometry: where it lives, and how the text does not jump when it comes"
note: "Agent draft from the part Annotations of the chapter The document of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# The document: annotations

Sources: the part "Annotations" of the chapter
[The document: text, geometry and decoration](../../../book/15-document.md) of the zendo book.

Parent: [The document](document.md).

## An annotation is geometry

Some data looks like decoration but changes the geometry: the author of a method, a code lens, an
inlay hint. It comes asynchronously, from git or from a language server, and it adds rows or
width. So an annotation is a part of the geometry, and it is in the model
([DOC-2](document.md)).

- The cursor goes over an annotation, and a click on it gives its own message.
- An annotation is not text, so the cursor cannot go into it.
- An annotation comes with a ticket and a version, as the answer of a parse does
  ([DOC-11](document-edits.md)).

## The text does not jump

1. **The structure reserves the space.** The core knows from the parse where a method starts. So
   the row for the author is there immediately, empty. Only the text in the row comes later, as
   decoration. Nothing jumps.
2. **If the space is not known before**, the geometry changes asynchronously. Then the scroll
   holds to the position in the text of the top visible row, not to a row number. An insertion
   above the screen moves nothing. An insertion on the screen moves the rows by one row.

The general rule: the structure gives the size, synchronously, and the decoration gives the
content, asynchronously. Asynchronous geometry is an exception, with the scroll that holds to the
text.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| DOC-19 | Ubiquitous | Data that adds rows or width to a document shall be an annotation in the geometry of the model, and shall not be decoration. | User, 2026-11-20 |
| DOC-20 | Optional | Where the structure tells the place of an annotation, the geometry shall reserve its space synchronously, and only its content shall come later. | User, 2026-11-20 |
| DOC-21 | Unwanted | If the space of an annotation is not known before its data comes, then the scroll shall hold to the position in the text of the top visible row while the geometry changes. | User, 2026-11-20 |

### Acceptance tests

**`an_annotation_has_rows_in_the_geometry`** (unit test, not written yet; verifies DOC-19)

- Given a model with a code file of three methods and the author row of each method.
- When the key `j` moves the cursor from the last row of a method to the next method, with no frame
  drawn.
- Then the cursor is on the first text row of the next method, and `layout` counts the author row
  between them.

**`a_reserved_annotation_moves_nothing`** (unit test, not written yet; verifies DOC-20)

- Given the same model before the answer of git: each author row is there and empty.
- When the answer with the authors comes.
- Then each text row, the cursor and the scroll are where they were, and the author rows have
  their text.

**`an_annotation_above_the_screen_moves_nothing`** (unit test, not written yet; verifies DOC-21)

- Given a model with a code file, scrolled so that row 200 of the text is the top visible row, and
  no reserved space.
- When an annotation of one row comes for a place above the screen, and then one for a place on the
  screen.
- Then the top visible row shows the same text after both, and the second annotation moves the rows
  under it by one row.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| DOC-22 | Ubiquitous | The cursor shall go over an annotation and shall not go into it. | User, 2026-11-20 |
| DOC-23 | Event | When the user clicks an annotation, the core shall get the message of that annotation. | User, 2026-11-20 |
| DOC-24 | Event | When the data of an annotation comes, `update` shall apply it only if its ticket and its version are both current. | User, 2026-11-20 |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| zendo shows no annotation today. Which one comes first, and which specification of [zendo](../../zendo.md) states what the user sees of it? | DOC-19 to DOC-24 | User |
