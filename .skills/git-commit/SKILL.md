---
name: git-commit
description: Use when writing, polishing, or reviewing git commit messages, especially when enforcing Conventional Commits and Netty-style structured bodies for larger changes.
---

# Git Commit Message Authoring

## Overview

Write commit messages with a strict format:
- Always use a Conventional Commit header.
- For larger changes, add a Netty-style body with `Motivation`, `Modifications`, and `Result`.

References:
- `references/commit-message-references.md`

## When to Use

Use this skill when the user asks to:
- Draft a commit message from a diff or change summary.
- Rewrite a weak commit message.
- Enforce consistent commit standards across a repo.

Do not use when:
- The user asks for exact project-specific phrasing that conflicts with Conventional Commits.

## Required Format

Header (always required):

`type(scope)!: short imperative summary`

Allowed `type`:
- `feat`
- `fix`
- `refactor`
- `perf`
- `test`
- `docs`
- `build`
- `ci`
- `chore`
- `revert`

Header rules:
- Imperative mood (`add`, `fix`, `remove`).
- No trailing period.
- Keep concise (target <= 72 chars).
- Add `!` if breaking.

## Large Change Body (Netty-style)

For multi-file, behavioral, risky, or architecture-affecting changes, append:

```text
Motivation:
- Why this change is necessary.

Modifications:
- Key technical changes made.

Result:
- Observable outcome after the change.
```

Optional footers:
- `Refs: #123`
- `Closes: #123`
- `BREAKING CHANGE: <details>`

## Selection Rules

Choose `type` by intent:
- New user-facing behavior -> `feat`
- Bug fix/regression fix -> `fix`
- Internal restructuring without behavior change -> `refactor`
- Performance-only improvement -> `perf`
- Tests-only update -> `test`
- Docs-only update -> `docs`
- Tooling/build/deps -> `build` or `chore`
- CI pipeline changes -> `ci`

Choose `scope`:
- Prefer nearest bounded component (package/module/service).
- Omit scope if no clear bounded component exists.

## Output Contract

When asked to author a message:
1. Return exactly one final commit message block unless user requests variants.
2. Include Netty-style body only when change is large; otherwise header-only.
3. If input is ambiguous, state one assumption in a single sentence, then provide the message.

## Automation

Use the bundled linter:
- `.skills/git-commit/scripts/commit-msg-lint.sh <commit-message-file>`

Enable automatic validation in this repository:
- `git config core.hooksPath .githooks`

The repo hook `.githooks/commit-msg` calls this linter on every commit.
