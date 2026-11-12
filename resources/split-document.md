Split the document {{FILE}}. It has {{WORDS}} words; the limit is {{LIMIT}}.

Keep the Altitude:

- Find the one level of detail that this document must keep. State it in one sentence first.
- Keep in this document only the content of that level. Move the deeper content into child documents.
- If the document covers more than one topic at the same level, split it by topic into sibling documents.
- Mirror the document tree in the folders: a document with children is `<name>/<name>.md`, in a folder of
  the same name that holds its children. A document without children is a file in the folder of its
  parent. Do not use README.md files as nodes of the tree.
- Each document links to its parent and to its children, and keeps the front matter of the original
  (status, date, and the like). Each new document is a draft.
- Keep every requirement, decision and open question. Keep each requirement ID in the one document that
  owns it; do not make new IDs.
- Each document stays within {{LIMIT}} words.
- Change only this document and the new documents. Propose each file as an edit; do not run commands.

First answer in at most 6 lines: the level of this document, and the documents you will make. Then make
the edits.
