---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "document.md"
next: "annotations.md"
level: "the open of a document, the parse after an edit up to the visible part of the screen, the identity of a block, and the answer of a parse"
note: "Draft from the parts about the open, the edit and the parse of the chapter The document of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# The document: open, edit and parse

Sources: the chapter
[The document: text, geometry and decoration](../../../book/15-document.md) of the zendo book.

Parent: [The document](document.md).

## The open is always asynchronous

To open a document is IO. The read, the parse and the wrap are one
[command](../commands.md). There is no limit of size and no second path in the code.

While the document loads, the model keeps the previous document, and the view shows the previous
screen. The name of the new file shimmers: an animation of the style only. To prevent a flicker,
the shimmer starts only after 100 ms.

## An edit is synchronous, up to the visible part

An edit parses the changed block again. The parse continues until the structure is the same as
before. Usually this is one block. The worst case: the user opens a code fence, and all of the
remaining document becomes code. For this case:

- the parse is synchronous from the edit to the end of the visible part of the screen, plus a
  margin;
- the tail after this point is marked as old, and a command parses it;
- until the answer comes, the tail below the screen keeps the old geometry. The user does not see
  it, and a scroll waits for the answer.

The limit is the visible part of the screen, not the size of the file. In a code file, the same
case occurs when the user opens `/*`.

**The identity of a block.** The parse stops when a new block is the same as the old block. The
same text inside a code fence and outside it gives different blocks. So a block is the same only if
its text and the parse state at its start are the same. In a code file, the parse state is the
state of the highlight at the start of the line.

**The answer of a parse** has a [ticket](../tickets-and-cancellations.md) and a version. The
ticket tells that the document waits for this answer. The version tells that the answer was
calculated from the current text.

**An example.** The user opens a fence and then closes it. After the closing fence, the tail is the
same as before the opening. The parse stops there, the tail is not old any more, and the runtime
cancels the command. If the command finished before, `update` discards its answer by the ticket.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| DOC-9 | Ubiquitous | The open of a document shall be one command for the read, the parse and the wrap, for a document of each size. | User, 2026-11-20 |
| DOC-10 | Event | When an edit changes the structure, `update` shall parse synchronously from the edit at most to the end of the visible part of the screen plus a margin, shall mark the tail after that point as old, and shall return a command that parses the tail. | User, 2026-11-20 |
| DOC-11 | Event | When the answer of a parse comes, `update` shall apply it only if its ticket and its version are both current. | User, 2026-11-20 |

### Acceptance tests

**`a_document_of_each_size_opens_through_one_command`** (unit test, not written yet; verifies DOC-9)

- Given a model with an open note.
- When the user opens a file of 1 KB, and in a second run a file of 20 MB.
- Then `update` returns the same open command in both runs, and the model keeps the first note
  until the answer comes.

**`an_open_fence_parses_only_the_visible_part`** (unit test, not written yet; verifies DOC-10)

- Given a model with a note of 10,000 lines and a screen of 50 rows.
- When the user types an opening code fence in the fifth visible row.
- Then the rows of the screen are code, the tail after the margin has the old geometry and is
  marked as old, and `update` returned one command that parses the tail.

**`an_old_answer_of_a_parse_is_discarded`** (unit test, not written yet; verifies DOC-11)

- Given the same model, which waits for the parse of the tail.
- When an answer with an old ticket comes, then an answer with the current ticket and an old
  version, and then the answer with the current ticket and the current version.
- Then the geometry of the tail changes only after the third answer.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| DOC-12 | State | While a document loads, the model shall keep the previous document, and the view shall show the previous screen. | User, 2026-11-20 |
| DOC-13 | State | While a document loads for more than 100 ms, the name of the new file shall shimmer, as an animation of the style only. | User, 2026-11-20 |
| DOC-14 | Event | When the width changes while a document loads, `update` shall ask for a new wrap, and the cursor shall move on the rows that the user sees until it comes. | User, 2026-11-20 |
| DOC-15 | Ubiquitous | The parse after an edit shall stop at the first block that is the same as before the edit. | User, 2026-11-20 |
| DOC-16 | Ubiquitous | A block shall be the same as an old block only if its text and the parse state at its start are the same. | User, 2026-11-20 |
| DOC-17 | State | While the tail is marked as old, the tail below the screen shall keep the old geometry, and a scroll into it shall wait for the answer. | User, 2026-11-20 |
| DOC-18 | Event | When a synchronous parse finds that the tail is the same as before, the document shall not wait for the parse of the tail any more, and the runtime shall cancel that command. | User, 2026-11-20 |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| How large is the margin after the visible part: a number of rows, or of screens? | DOC-10 | User |
| "A scroll waits for the answer": does the screen stay, or does it show the old geometry of the tail? | DOC-17 | User |
