---
status: "[draft or completed]"
date: "[YYYY-MM-DD]"
scope: "[the commit range, the branch, the pull request, or the paths that the review covers]"
reviewer: "[the agent and its model, or the person]"
level: "the findings of one code review, the suggested fixes, and the critic review of each fix"
---

# Code review: [scope]

<!--
Rules for the author:

- Write one report for each review, also when the review finds no problem.
- A finding states one defect. Give the location as `path:line`.
- The requirement column names the requirement ID that the finding breaks, or `-`.
- Each finding has a suggested fix: what to change, in one or two sentences. The fix does not
  change a requirement; if the defect is in the requirement, the suggested fix says so.
- At least one critic reviews each fix before its outcome is `fixed`. A critic is a person or an
  agent with a new context that did not write the fix. Give the critic the finding and the diff,
  not your conclusion.
- If a critic rejects a fix, change the fix and get a new critic review. Record each review.
- The outcome is `fixed`, `skipped`, or `no change needed`. Leave it empty until a decision exists.
- Set the status to `completed` when every finding has an outcome.
- The author shall delete this comment and every section that does not apply.
-->

[One paragraph: what the review covers, why it was done, and the result in one sentence.]

## Scope

Reviewed: [the commits, the files, or the modules].

Not reviewed: [what the review did not cover, and why].

## Findings

| ID  | Severity               | Location      | Finding                     | Requirement | Outcome   |
| --- | ---------------------- | ------------- | --------------------------- | ----------- | --------- |
| R-1 | [high, medium, or low] | `[path:line]` | [One sentence: the defect.] | [ID or -]   | [Outcome] |

### R-1: [short name]

Suggested fix: [what to change, and why it removes the defect].

## Fix review

| Finding | Fix                  | Critic                   | Verdict                | Note                  |
| ------- | -------------------- | ------------------------ | ---------------------- | --------------------- |
| R-1     | [commit or diff ref] | [person, or agent+model] | [accepted or rejected] | [what the critic saw] |

## Checks run

[The commands and their results: `cargo test`, `cargo clippy --all-targets -- -D warnings`, and
other checks.]

## Follow-up

| Finding | Why it was skipped | Where it is tracked |
| ------- | ------------------ | ------------------- |
| [R-n]   | [Reason]           | [Milestone or note] |
