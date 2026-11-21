---
review:
  date: "2026-11-21"
  passes: 1
  result: "pass"
  hash: "sha256:5820906e5dd704e705961358559dd539354554525f006d2bff46b87c8a6a7088"
---
# Chapter 14: File writes and external consistency

The write of notes and of the config is the most dangerous IO: an error here
can cause a loss of the data of the user. Thus it has its own rules on top of the usual commands.

**One queue for one path**

The user edits a note fast, and the core asks to write it three times: A, B, C. If three writes
start in parallel, they can finish in any order, and the disk can have B instead of C.

Thus each path has **its own queue**:

- at one time, only one write runs for one file;
- while it runs, new requests wait;
- of the waiting requests, **only the last one** stays. A was written, and B and C waited: the
  runtime writes C immediately and skips B.

The result: no races and no unnecessary writes. The disk always has the newest text.

**The user sees that a write runs**

Until the write is complete, the status bar shows it. Thus there is no situation "I think that I
saved it, but it is not saved".

**How a file is written**

1. The executor copies the file to a backup: `name~`.
2. It truncates the file and writes the new content in place. The write is synchronous: when it
   ends, the data is on the disk.
3. If the write succeeds, then the executor deletes the backup. If it fails, then the executor
   restores the file from the backup.

The file keeps its identity: its links, its owner and its permissions stay, and a watcher of the
file sees the change. If zendo stops during the write, then the backup stays on the disk. A reader
can see a partial file during the write: this is the cost of a write in place.

**An error does not lose data**

If the write fails:

- the runtime sends `Msg::WriteFailed`;
- the model **keeps the data marked as not saved**. The text stays in the editor;
- the status bar shows the error.

The user lost nothing and can try again.

**The state saves itself**

In this architecture, the saved part of the model has a **version**. When the version changes,
the runtime itself writes the file, after a small delay (debounce), so that ten changes in sequence
give one write. The code of the components does not do the save: it only changes the model.

**Other writers**

A different program can change the file: a different editor, git, the agent. Thus, when the core
reads a file, the model keeps the hash of the file on the disk: the base. A write command has the
base. Before the write, the executor reads the hash again. If the hash is not the base, then the
executor does not write, and it sends `Msg::WriteConflict`.

A subscription to the file sends `Msg::FileChanged`. If the document has no unsaved changes, then
the model reads the file again. If it has unsaved changes, then the model keeps the conflict.
The user decides: keep the text of the editor, take the text of the disk, or merge them. After its
own write, the model sets the base to the new hash, thus the event about its own write changes
nothing.

A short time remains between the check and the write. The check makes a race rare and visible,
but not impossible.

The subscription watches the folder of the file, not the file. A different program can replace the
file with a rename, and only the folder sees this. An advisory lock prevents a race between two
zendo processes. Other programs do not use the lock, thus for them only the check of the hash
remains.

**Exit**

When the user exits, the runtime does not stop the queue. It waits until all the waiting writes are
complete, with a time limit. All this time, the screen continues to draw and shows "saving…".

**All of the architecture in one paragraph**

An event becomes a `Msg`. `update` changes the model and returns `Cmd`. The runtime does the
commands asynchronously and returns the results as new `Msg`. `subscriptions` tells which streams
of events are necessary now. `layout` calculates the sizes. `view` only reads the model and builds
the frame. Clicks become `Msg` through the hit map. The components are isolated and send messages
through the parent. The UI loop waits only for messages, thus no IO can stop it.
