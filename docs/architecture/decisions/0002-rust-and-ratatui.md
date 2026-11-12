---
status: "draft"
date: "2026-11-11"
sources:
  - "../../zendo/zendo.md"
  - "The choice of the user: \"i thought of rust\", 2026-11-11"
---

# 0002 Write zendo in Rust with ratatui

## Context

zendo is a terminal program that must start fast, draw a calm screen with several panes, and run on
macOS and Linux. The user installs it with one line and upgrades it often.

## Options

| Option                       | Benefit                                                                                           | Cost or constraint                                                    |
| ---------------------------- | ------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| Rust with ratatui            | One native program, fast start. ratatui draws panes, tables and styled text. The user chose Rust. | Builds take minutes the first time. Needs a Rust toolchain to build.  |
| Go with Bubble Tea           | One native program, fast builds.                                                                  | Not the choice of the user.                                           |
| Python with Textual          | Fast to write.                                                                                    | Needs a Python environment on each machine; slower start.             |

## Decision

zendo is a Rust program with ratatui on the crossterm backend, and tokio for the connection to
Claude.

## Consequences

- The installer builds zendo from source with `cargo`. It keeps the build folder, so an upgrade
  builds only what changed.
- `Cargo.toml` states the minimum Rust version that the dependencies need.

## Verification

`cargo test` runs on each change.

## Decision record

The user chose Rust on 2026-11-11. This record is not reviewed by the user.
