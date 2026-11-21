# Chapter 15: The document: text, geometry and decoration

A document has much state: the text, the blocks, the lines after the wrap, the highlight. This chapter
tells which part owns each layer. The examples use a markdown file, but the same rules apply to a
code file.

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
  data, but `update` calculates it again itself, synchronously: after each edit and after
  `Msg::Resize`. To stay in the budget, the calculation is incremental: `update` parses and wraps
  again only the changed blocks.
- **The decoration (layer 4) is in the cache of the view, and it comes asynchronously.** The
  highlight is a `Cmd`, and its key is (the hash of the block, the theme). While there is no
  highlight, the view shows plain text.

The main rule is the same as for an animation: **the decoration does not change the geometry.** The
highlight only gives colors to symbols that are already in their positions. Thus, when the
highlight comes, nothing jumps: not the rows, not the cursor, not the hit map.

**The cursor is a position in the text, not a row.** The rows come from the text. Thus, when the
width changes, the wrap changes, but the cursor stays on the same symbol.

**The open of a document is always asynchronous**

To open a document is IO: it is necessary to read the file. Thus the parse and the wrap go in the
same command. There is no limit of size and no second path in the code.

```rust
enum Doc {
    Loading { path: PathBuf, req: u64, since: Instant, prev: Option<Box<DocState>> },
    Ready(DocState),
    Failed { path: PathBuf, error: String },
}

struct DocState {
    text: Buffer,
    version: u64,
    blocks: Vec<Block>,   // the structure
    lines: Lines,         // the wrap for `width`
    width: u16,
}
```

1. `update` gets "open the file". It sets `Doc::Loading` and returns
   `Cmd::OpenDoc { path, width, req }`.
2. The executor reads the file in `spawn_blocking`, parses the blocks, wraps them to `width`, and
   sends `Msg::DocOpened { req, text, blocks, lines, width }`.
3. `update` checks `req`. It discards an old answer. It changes a new answer into `Doc::Ready`.
4. If the width changed during this time, then `update` returns `Cmd::Rewrap { version, width }`.
   Until the answer comes, the model keeps the old wrap, and the cursor moves on the rows that the
   user sees.

A small file comes after one millisecond, in the next frame. While the document loads, the model
keeps the previous document in `prev`, and the view shows the previous screen. The name of the new
file shimmers: this is an animation, thus it changes only the style. To prevent a flicker, the
shimmer starts only when `now - since` is more than 100 ms. This is the same rule: the view
calculates from `now`.

**An edit is synchronous, up to the window**

`doc.edit(op)` parses the changed block again and continues until the structure is the same as the
old structure. Usually this is one block.

The worst case: the user opens ` ``` `. Then all of the remaining document becomes code, and the
parse must go to the end of the file. The rule for this case:

- the parse is synchronous from the edit to the end of the visible window, plus a margin;
- the tail after this point is marked as old, and it goes to `Cmd::Reparse { version, from }`;
- until the answer comes, the tail below the screen keeps the old geometry. The user does not see
  it. The cursor cannot go there without a scroll, and the scroll waits for the answer.

Thus the code has one path for all sizes. The limit is the window of the screen, not the size of the
file.

**The key of a block**

The parse stops when a new block is the same as the old block. But "the same" cannot mean only "the
same text". The same text inside a code fence and outside it gives different blocks. Thus the key
of a block is:

```rust
struct BlockKey {
    hash: u64,          // the text of the block
    entry: ParseState,  // the parse state at the start: Normal, InFence(lang), ...
}
```

The parse stops when a new block has the same `BlockKey` as the old block at this position.

**A code file**

A code file has the same layers. The structure is the tokens, not the markdown blocks. A "block" is
a line, and the entry state is the state of the highlight at the start of the line. Usually code
does not wrap: one line of the text is one row on the screen, and a horizontal scroll shows the long
lines.

The case of the fence also occurs in code: when the user opens `/*`, all of the tail becomes a
comment. The rules are the same: the parse is synchronous up to the end of the window, the tail goes
to a command, and the key of a line is (the hash, the entry state).

**An example: the user opens a fence and then closes it**

1. The user opens ` ``` `. This is version N. `update` parses the window synchronously, marks the
   tail as old, and sends `Cmd::Reparse { version: N, from }`.
2. The user closes ` ``` `. This is version N+1. The synchronous parse starts at the edit again.
3. After the closing ` ``` `, the text of the tail is the same as before the opening. The entry
   state is `Normal` again, thus the keys are the same as the old keys. The parse finds a block with
   the same key and stops. The tail is not old any more.
4. The model does not wait for the parse of the tail any more. The runtime sees this and cancels
   task N.
5. If task N finished before the cancellation, then its answer has version N. The model is at N+1,
   and `update` discards the answer.

This is the same as in the chapter about tickets: the cancellation saves CPU, and the version
guarantees correctness.

If task N finished before the user closed the fence, then the old structure of the tail is gone: all
of the tail is code now. After the close, the keys of the tail (`InFence`) are not the same, thus
the tail is old again, and `Reparse { version: N+1 }` goes to the executor. This is correct, but it
costs more.

To make this case cheap too, the document has a memo of blocks by `BlockKey`, with a size limit. The
old blocks with `entry = Normal` are still in the memo, and the parse of the tail takes them from the
memo. It is still necessary to go through the tail, but this is a search by key, not a parse.

**Annotations**

Some data looks like decoration but changes the geometry: the author of a method, a code lens, an
inlay hint. This data comes asynchronously, from git or from a language server, and it adds rows or
width. Thus it is a third level: an annotation.

- An annotation is a part of the geometry, thus it is in the model. `update` must know about it:
  the cursor goes over it, and a click on it gives its own `Msg`, for example "show the commit".
- An annotation is not text. The cursor is a position in the text, thus the cursor cannot go into an
  annotation.
- An annotation comes as `Msg::Annotations { version, items }`, and the version works as usual.

To prevent a jump of the text:

1. **The structure reserves the space.** The core knows synchronously, from the parse, where a
   method starts. Thus the row for the author can be there immediately, empty. Only the text in the
   row comes asynchronously, as decoration. The geometry does not change, and nothing jumps.
2. **If the space is not known before** (an inlay hint is only where the language server found
   something), then the geometry changes asynchronously. Then the scroll does not hold to a row
   number on the screen. It holds to the position in the text of the top visible row. An insertion
   above the screen moves nothing. Only an insertion on the screen moves the rows, by one row for
   each annotation.

The general rule: **the structure gives the size (synchronously), and the decoration gives the
content (asynchronously).** Asynchronous geometry is only an exception, and only with the scroll
that holds to the text.
