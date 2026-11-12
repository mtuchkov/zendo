---
status: "[draft, approved, implemented, completed, rejected, or superseded]"
approved: "[YYYY-MM-DD of the user approval; omit while draft]"
revision: "[N]"
date: "[YYYY-MM-DD]"
parent: "[relative path of the parent document]"
prev: "[relative path of the previous sibling in reading order]"
next: "[relative path of the next sibling in reading order]"
children:
  - "[relative path of a child document]"
level: "[the one level of detail that the whole document keeps]"
---

# [Specification title]

<!--
Rules for the author:

- The author shall keep the document within about 1,000 words. If it does not fit, then the author
  shall keep one level of detail here and move deeper content into child documents. A document with
  children is `<name>/<name>.md`; each document links to its parent and its children.
- The author shall state what the user sees and does, user decisions, the responsibilities of the
  parts of zendo (panes, the file tree, the note, the delta, recaps, the Claude connection, the
  commands `zendo setup`, `zendo doctor` and `zendo upgrade`, the installer), and the main design
  choices. Mechanisms, such as numeric limits, the fields of `.zendo/db.json`, ACP messages, crate
  calls, module layout, and unit test lists, belong in the code.
- The author shall record an architecture decision in a decision record under
  `docs/architecture/decisions/`, not in a specification.
- The author shall write each user decision as a requirement row with the Source "User, <date>".
- An architecturally significant requirement has a measurable effect on the architecture or on a
  quality of zendo, such as start time, start without the network, "if zendo is up, then Claude
  works", or the safety of the files of the user (https://adr.github.io/). A detailed requirement
  fixes a lower-level rule within that architecture.
- A requirement ID is `<TOPIC>-<n>`, with a topic of this specification, for example `TREE-3` or
  `DOCTOR-2`.
- The author shall state only requirements that stay valid in every milestone. Temporary behavior
  belongs in the milestone document under `docs/milestones/`.
- Acceptance tests name the requirements that they verify. Requirements never name tests.
- The author shall delete this comment and every section that does not apply.
-->

## [Topic section]

\[What the user sees and does, and the main design choices of the topic, in Simplified Technical
English. Mark a proposal of the author "Design".\]

## Architecturally significant requirements

| ID            | Pattern                                           | Requirement          | Source                                   |
| ------------- | ------------------------------------------------- | -------------------- | ---------------------------------------- |
| \[TOPIC-<n>\] | [Ubiquitous, Event, State, Unwanted, or Optional] | [One EARS sentence.] | \[User, <date>; a parent ID; or Design\] |

### Acceptance tests

**`[test name]`** ([unit test, mock agent, terminal check, or real Claude]; verifies [IDs])

- Given [the repository, its files and `project-structure.toml`, the state in `.zendo/db.json`, the
  terminal size].
- When [the key, the mouse action, the command, or the event from Claude].
- Then [what the user sees on the screen, in the files, or in the output of the command].

A unit test or a mock-agent test runs in `cargo test`, without the network and without a Claude
login. A terminal check runs zendo in a terminal of a given size, for example with `zendo --mock`. A
test that depends on what Claude writes uses real Claude. Each architecturally significant
requirement has at least one test that runs in `cargo test`. Each milestone document names the tests
that it must pass.

## Detailed requirements

| ID            | Pattern   | Requirement          | Source                                   |
| ------------- | --------- | -------------------- | ---------------------------------------- |
| \[TOPIC-<n>\] | [Pattern] | [One EARS sentence.] | \[User, <date>; a parent ID; or Design\] |

Retired in revision \[N\]: [IDs, with the reason]. Retired identifiers are never reused.

## Variants

**Root document.** After the status, add: the sources; a table of the child documents with their
topics; Scope (in scope, out of scope); how zendo works, in one or two paragraphs; the main design
choices; the shared inputs of the acceptance tests, such as a sample repository; the success
criterion, which is that the acceptance tests of every architecturally significant requirement pass;
and the milestones that deliver the specification.

**Pane document.** State what the pane shows; the keys and the mouse actions that work in it; how it
gets and loses focus; when it hides and opens, in each layout; and what zendo keeps about it between
starts.

**Command document.** For `zendo setup`, `zendo doctor`, `zendo upgrade`, or the installer, state: the
checks in their order; what the command changes on disk; when it needs the network; what it shows
for each failed check and the fix that it suggests; and when it asks the user, and when it must not
wait for input.

**Claude document.** State what zendo sends to Claude, including the context of the note and the
rules from `project-structure.toml`; the tool calls of Claude that zendo allows, rejects, or shows to
the user as a proposal; where each thread lives and how long it lasts; and what the user sees when
Claude is not available.

**Open points.** Only one table: open question, what it affects, and who answers it. Risks,
dependencies, and resolved questions do not belong here.
