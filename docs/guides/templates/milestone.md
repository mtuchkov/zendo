---
status: "[draft, approved, implemented, or completed]"
approved: "[YYYY-MM-DD of the user approval; omit while draft]"
date: "[YYYY-MM-DD]"
version: "[the zendo version that ships the milestone, for example 0.10.0]"
parent: "[relative path of the milestone list or the previous milestone]"
spec: "[relative path of the specification that the milestone delivers]"
level: "the scope, the temporary behavior, and the acceptance of one milestone"
---

# Milestone: [name]

<!--
Rules for the author:

- A milestone document states only what is particular to one milestone. The specification states
  the behavior that stays valid in every milestone; the milestone document never repeats it.
- Temporary behavior is behavior that a later milestone removes or changes, such as a key that is
  not bound yet or a pane that shows a placeholder. Each temporary requirement keeps its identifier
  and moves here from the specification.
- The acceptance names the acceptance tests of the specification that the milestone must pass,
  and the additional checks of its temporary behavior.
- The author shall delete this comment and every section that does not apply.
-->

[One paragraph: what the milestone gives the user and why it comes at this point.]

## Scope

In scope: [the parts of the specification that this milestone delivers].

Out of scope: [the parts that later milestones deliver, each with its milestone].

## Temporary behavior

[How the parts that are not done behave until a later milestone, and the known gaps.]

| ID            | Pattern   | Requirement          | Source                                   |
| ------------- | --------- | -------------------- | ---------------------------------------- |
| \[TOPIC-<n>\] | [Pattern] | [One EARS sentence.] | \[User, <date>; a parent ID; or Design\] |

## Acceptance

The milestone is accepted when these acceptance tests of the specification pass: [names]. In
addition: [checks of the temporary behavior, with the requirements that they verify].

## Open items

| Open item | Affected | Needed from |
| --------- | -------- | ----------- |
| [Item]    | [What]   | [Who]       |

## Completion record

\[Date, commit, version, the result of `cargo test`, the terminal checks and the terminal sizes,
the checks with real Claude and who ran them, and the tests not run yet.\]
