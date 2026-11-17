Turn the note {{FILE}} into specifications of this repository.

The note holds ideas, decisions and requirements in the order that the user had them. Put each of them
where it belongs. Think first about the best way to do that, then do it:

1. Read the note.
2. Read `project-structure.toml`, if the repository has one: it says where the specifications live, which
   template they follow, and the rules for each folder. Read `AGENTS.md` or `CLAUDE.md` for the rules of the
   requirements.
3. Find the specifications that already cover the topics of the note.
4. For each topic, choose one way:
   - update the specification that covers the topic, when there is one;
   - write a new specification, only when no specification covers the topic.
   A topic that is not ripe for a specification (an open question, a task, a loose idea) stays in the note.
5. Answer first in at most 6 short lines: for each topic, the document that you update or write, and why.
6. Then make the edits.

Rules:

- Follow the rules of the repository for specifications: the template, the requirement syntax, the
  identifiers, the front matter, the links between parent and child documents, and the word limit.
- Keep the level of detail of each document that you update. Move deeper content into a child document.
- A new requirement gets a new identifier only as a proposal: say so in the note field of the front matter.
  Do not reuse retired identifiers. Keep every existing identifier.
- A new or changed document is a draft (`status: draft`) until the user reviews it.
- Do not change the note. Do not change code, tests or other files. Propose each file as an edit; do not
  run commands.
- When nothing in the note belongs in a specification, say so in one line and change nothing.
