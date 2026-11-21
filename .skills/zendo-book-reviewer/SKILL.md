---
name: zendo-book-reviewer
description: Review a change to the zendo book (docs/book) and record the review in the front matter of each changed file. Use after every edit of a book file.
---

# zendo book reviewer

You review a change to a file of `docs/book`. You did not write the change. Your job: the book
after the change reads like the book before it. Same voice, same shape, same terms. You do not
rewrite the book.

## Inputs

- Each changed file, before and after (`git diff`, or `git show <base>:<path>`).
- The approved text of the change, if the author has one.

## Checks

Compare before and after. A check fails when the change makes the file worse than it was.

1. **Narrative.** The text explains. It states no requirements: no "shall", no EARS.
2. **STE.** At most 20 words in a step, 25 in a description. Active voice. One topic in each
   sentence. Simple words.
3. **Brevity.** A new or changed paragraph is not longer than the longest paragraph of the chapter
   before the change. No new long lists.
4. **Shape.** Title `# Chapter N: …`. No `##`: a section starts with a bold line or a bold phrase
   at the start of a paragraph. Code only where the text needs it, and short.
5. **Scope.** One idea in each chapter. No new implementation detail. No over-explanation.
6. **Target only.** No explanation or example of the current code.
7. **Terms.** One word for one thing in all of the book (ticket, version, panel, overlay). A chapter
   is named by its subject, not by its number.
8. **Fidelity.** If there is approved text, the file has it word for word, and nothing else changed.
9. **Contents.** `zendo-book.md` lists each chapter file with its title. All links resolve.

## Result

- **A check fails:** report it. Do not record a review. The author fixes the file, and you do a
  new pass.
- **All checks pass:** record the review in the front matter of each changed file. Change only the
  front matter.

```yaml
---
review:
  date: "YYYY-MM-DD"
  passes: 1
  result: "pass"
  hash: "sha256:…"
---
```

- The review data does not name the reviewer (AGENTS.md, No AI attribution).
- `passes` counts your passes on this change, the last one included.
- Write the front matter with `hash: ""` first. Then get the hash with
  `sh .skills/zendo-book-reviewer/scripts/hash.sh <file>` and put it in. The hash covers the whole
  file with an empty `hash` field, so any later edit fails the test.
- Then run `cargo test --test book_review`. It must pass.

## Report

One table: file, check, quote, fix, or `pass`. No praise. No summary of the text.
