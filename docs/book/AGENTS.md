# Rules for agents: the zendo book

The book explains the architecture of the zendo TUI, one chapter at a time. These rules replace the
style rules of the root [AGENTS.md](../../AGENTS.md) for this folder: EARS, the two-page limit, the
Keep the Altitude rule, and the `status` field of the front matter do not apply here.

## Changes

- Only the user starts a change of the book.
- An agent changes a file of the book only with the explicit permission of the user for that
  change. An agent does not change the book on its own, also when a change looks small or obvious.

## Style

- Write in Simplified Technical English: short sentences, active voice, one topic in each sentence.
- Write explanation, not requirements. Do not use "shall".
- Describe only the target architecture. Do not explain or give examples of the current code.
- Keep paragraphs short. A chapter has no `##` headings. A section starts with a bold line
  (`**Heading**`) or with a bold phrase at the start of a paragraph.
- Refer to a chapter by its subject ("the chapter about the view"), not by its number.
- Keep the table of contents in `zendo-book.md` in step with the chapter files and their titles.

## Review

- After each change of a file of the book, a reviewer reviews the change with the skill
  [.skills/zendo-book-reviewer/SKILL.md](../../.skills/zendo-book-reviewer/SKILL.md), at least one
  pass. The reviewer is a person or an agent with a new context that did not write the change.
- The reviewer records the review in the front matter of the file:

  ```yaml
  ---
  review:
    date: "2026-11-20"
    passes: 1
    result: "pass"
    hash: "sha256:<hash of the whole file, with hash: \"\">"
  ---
  ```

- `tests/book_review.rs` checks that the hash is the hash of the current file with an empty `hash`
  field. An edit after the review fails the test, also an edit of the review data. Thus the review
  always comes after the last edit.
- This file has no review.
