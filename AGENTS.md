# Rules for agents

## Project structure

The project structure is the set of top-level folders, the folders of `docs/`, and the file
`project-structure.toml`. Code modules are not part of it: a change of code does not change
`project-structure.toml`.

- The agent shall not change the project structure unless the user asks for that change directly.
- WHEN the user asks for a change to the project structure, the agent shall show the change and get
  the approval of the user before it makes the change.
- IF a task needs a change to the project structure that the user did not ask for, THEN the agent
  shall stop and ask the user. The agent shall not make the change as part of the task.
- `tests/project_structure.rs` checks `project-structure.toml` with the checks of `zendo doctor`
  in `cargo test`: unknown keys, paths, templates, and unique labels. The `.githooks/pre-commit`
  hook runs this test when a commit changes `project-structure.toml` or deletes or renames a file.

## Documentation and requirement authority

- The agent shall use standard technical english for project documentation and easy approach to requirements syntax: “The system shall…”, “When…, the system shall…”,
  “While…, the system shall…”, “Where…, the system shall…”, and “If…, then the system shall…”.
- Each specification shall split its requirements into architecturally significant requirements,
  with acceptance tests in the same document, and detailed requirements. The
  [specification template](docs/guides/templates/specification.md) defines the format.
- Each specification shall state only requirements that stay valid in every milestone. Temporary
  behavior shall go to the milestone document in `docs/milestones/`, which
  also names the acceptance tests that the milestone must pass.
- The agent shall record architecture decisions as
  decision records in `docs/architecture/decisions/`, not in specifications.
- Each new independently testable requirement shall receive a unique, stable, subject-specific
  identifier in its authoritative specification. Plans, code, tests, and translations shall reuse
  that identifier rather than define a competing requirement.
- The agent shall assign new requirement identifiers only in a user-reviewed high-level
  specification. Retired identifiers shall not be reused.
- Agent drafts and templates shall remain non-authoritative until reviewed.
- The `status` field of a document's YAML front matter shall hold one value of this enum: `draft` (agent-written and not reviewed by the user, including
  transcriptions and translations), `approved`, `implemented` (the code implements the document, and
  its acceptance is incomplete), `completed` (acceptance evidence is recorded), `rejected`, or
  `superseded`. The `approved` field shall hold the date of the user approval, and the `note` field
  shall hold any explanation.
- The agent shall set `approved` or a later status only after the user approves the document.

- The agent shall limit each architecture or design document that it writes or revises to two pages,
  about 1,000 words.
- The agent may split a document by topic into sibling documents. If a document does not fit into
  two pages, then the agent shall keep in it only content of one level of detail and shall move more
  detailed content into child documents (the Keep the Altitude rule).
- Each split document shall link to its parent and to its children.
- The agent shall mirror the document tree in the directory structure. A document with children
  shall be the file `<name>/<name>.md`, in a directory of the same name that holds its children. A
  document without children shall be a file in the directory of its parent and shall get no
  directory of its own. The agent shall not use `README.md` files as tree nodes.

## Code

- A source file shall have at most 300 lines, and a function at most 70 lines.
- Unit tests shall live in a `*_tests.rs` file beside the code that they test, not inline. A test
  file can be longer than 300 lines.
- `tests/code_size.rs` checks these limits in `cargo test`. Clippy checks the length and the
  cognitive complexity of each function (`clippy.toml`); CI runs
  `cargo clippy --all-targets -- -D warnings`.
- WHEN the agent refactors code to fit the limits, the agent shall reduce the cyclomatic complexity:
  fewer branches and less nesting in each function, not only shorter functions.
- The agent shall prefer iterators, from the standard library or its own, to nested loops.
- CI checks the GitHub workflows with `actionlint`. The agent shall run `actionlint` before it
  commits a change of `.github/workflows/`.

## Tests

- The agent shall use test-driven development: first a test that fails, then the code that makes
  it pass.
- WHEN an architecturally significant requirement changes or a new one comes, the agent shall
  check again what each affected test means and whether it still makes sense, and change, add or
  remove tests until the tests and the requirements agree.
- Tests shall name the requirements that they verify.

## Code review

- WHEN the agent reviews code, the agent shall write a report in "Code reviews" of
  `project-structure.toml`, from the [code review template](docs/guides/templates/code-review.md).
- IF the review finds no problem, THEN the agent shall still write the report, with an empty
  findings table.
- Each finding shall state the location, the severity, the requirement that it breaks (if there is
  one), a suggested fix, and the outcome: `fixed`, `skipped`, or `no change needed`.
- Before the agent sets the outcome `fixed`, at least one critic shall review the fix. A critic is
  a person or an agent with a new context that did not write the fix.
- IF a critic rejects a fix, THEN the agent shall change the fix and get a new critic review.
- WHEN every finding has an outcome, the agent shall set the status of the report to `completed`.

## Commits

- Each commit message shall follow [.skills/git-commit/SKILL.md](.skills/git-commit/SKILL.md): a
  Conventional Commit header (`type(scope)!: summary`, at most 72 characters, no period at the end),
  and for a large change a body with `Motivation:`, `Modifications:` and `Result:`.
- The agent shall check each message with `.githooks/commit-msg` before it commits. In a clone, turn
  the check on with `git config core.hooksPath .githooks`; CI checks every commit of a push and of a
  pull request, and the title of a pull request.
- Each commit shall increase the package version in `Cargo.toml`. The agent shall choose the new
  version with the rules of [VERSION.md](VERSION.md).
