---
review:
  reviewer: "agent, new context"
  date: "2026-11-20"
  passes: 1
  result: "pass"
  hash: "sha256:4b5febe29d186654bf9c3a0d85027430812ccc0d2684fabf1bcaaad2fc2c163f"
---
# Chapter 15: The document: text, geometry and decoration

A document has much state: the text, the blocks, the lines after the wrap, the highlight. This
chapter tells which part owns each layer. The examples use a markdown file, but the same rules apply
to a code file.

**The layers of the document**

1. **The text**: the buffer of the editor and its version. This is the truth. It is in the model,
   and `update` changes it.
2. **The structure**: the markdown blocks that are parsed from the text.
3. **The lines**: the blocks after the wrap to the width of the column. The inline markup is also
   here: the view does not show `**` and `` ` ``, thus the markup changes the width of a line.
4. **The decoration**: the highlight of code, the colors of inline styles, the theme.

**Why the view cache cannot keep all layers**

`update` needs the lines. Without them, it cannot work:

- `j` and `k` in the reader move the cursor by rows on the screen, not by lines of the text;
- to keep the cursor visible, `update` must know how many rows a block has after the wrap;
- a click (row, column) must become a position in the text;
- a selection goes by rows on the screen.

If the lines are in an asynchronous cache, then `update` can find no lines. Then it does not know
where to move the cursor.

**Geometry and decoration**

- **The geometry (layers 2 and 3) is in the model, in the document component.** This is derived
  data, but `update` calculates it again itself, synchronously: after each edit and after a resize.
  To stay in the budget, the calculation is incremental: `update` parses and wraps again only the
  changed blocks.
- **The decoration (layer 4) is in the cache of the view, and it comes asynchronously.** While there
  is no highlight, the view shows plain text.

The main rule is the same as for an animation: **the decoration does not change the geometry.** The
highlight only gives colors to symbols that are already in their positions. Thus, when the
highlight comes, nothing jumps: not the rows, not the cursor, not the hit map.

**The cursor is a position in the text, not a row.** The rows come from the text. Thus, when the
width changes, the wrap changes, but the cursor stays on the same symbol.

**The open of a document is always asynchronous**

To open a document is IO: it is necessary to read the file. Thus the read, the parse and the wrap
are one command. There is no limit of size and no second path in the code.

While the document loads, the model keeps the previous document, and the view shows the previous
screen. The name of the new file shimmers: this is an animation, thus it changes only the style. To
prevent a flicker, the shimmer starts only after 100 ms.

If the width changes while the document loads, then `update` asks for a new wrap. Until it comes,
the cursor moves on the rows that the user sees.

**An edit is synchronous, up to the visible part of the screen**

An edit parses the changed block again. The parse continues until the structure is the same as
before. Usually this is one block.

The worst case: the user opens ` ``` `. Then all of the remaining document becomes code. The rule
for this case:

- the parse is synchronous from the edit to the end of the visible part of the screen, plus a
  margin;
- the tail after this point is marked as old, and a command parses it;
- until the answer comes, the tail below the screen keeps the old geometry. The user does not see
  it, and a scroll waits for the answer.

Thus the code has one path for all sizes. The limit is the visible part of the screen, not the size
of the file.

**The identity of a block**

The parse stops when a new block is the same as the old block. But "the same" cannot mean only "the
same text". The same text inside a code fence and outside it gives different blocks. Thus a block
is the same only if its text and the parse state at its start are the same.

**The answer of a parse**

An answer of a parse has a ticket and a version. The ticket tells that this is the answer that the
document waits for. The version tells that the answer is calculated from the current text.
`update` applies the answer only if both are current.

**A code file**

A code file has the same layers. The structure is the tokens. A block is a line, and the parse
state is the state of the highlight at the start of the line. Usually code does not wrap: one line
of the text is one row on the screen.

The case of the fence also occurs in code: when the user opens `/*`, all of the tail becomes a
comment. The rules are the same.

**An example: the user opens a fence and then closes it**

1. The user opens ` ``` `. The core parses the visible part synchronously and sends a command for
   the tail.
2. The user closes ` ``` `. The synchronous parse starts at the edit again.
3. After the closing ` ``` `, the tail is the same as before the opening: the same text and the same
   parse state. The parse stops there, and the tail is not old any more.
4. The document does not wait for the parse of the tail any more. The runtime cancels the command.
5. If the command finished before the cancellation, then its answer has an old ticket, and `update`
   discards it.

The cancellation saves CPU, and the ticket and the version guarantee correctness.

**Annotations**

Some data looks like decoration but changes the geometry: the author of a method, a code lens, an
inlay hint. It comes asynchronously, from git or from a language server, and it adds rows or width.
Thus an annotation is a part of the geometry, and it is in the model.

- The cursor goes over an annotation, and a click on it gives its own `Msg`.
- An annotation is not text, thus the cursor cannot go into it.
- An annotation comes with a ticket and a version, as an answer of a parse does.

To prevent a jump of the text:

1. **The structure reserves the space.** The core knows from the parse where a method starts. Thus
   the row for the author is there immediately, empty. Only the text in the row comes later, as
   decoration. Nothing jumps.
2. **If the space is not known before**, then the geometry changes asynchronously. Then the scroll
   holds to the position in the text of the top visible row, not to a row number. An insertion above
   the screen moves nothing. An insertion on the screen moves the rows by one row.

The general rule: **the structure gives the size, synchronously, and the decoration gives the
content, asynchronously.** Asynchronous geometry is only an exception, with the scroll that holds to
the text.
