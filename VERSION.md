# Version

The version of zendo is the package version in `Cargo.toml`, in the format `MAJOR.MINOR.PATCH`.
`zendo --version` shows it.

## How to choose the new version

- Each commit shall increase the version.
- WHEN a commit does not change what a user can see or do (documentation, tests, CI, refactoring,
  a small fix), the author shall increase PATCH by 1.
- WHEN a commit adds, changes or removes a behavior that a user can see, the author shall increase
  MINOR by 1 and set PATCH to 0.
- The author shall increase MAJOR only when the user asks for it.
- IF two branches use the same version, THEN the branch that merges second shall increase its
  version again, above the version of `main`.

## How to set it

1. Change `version` in the `[package]` section of `Cargo.toml`.
2. Run `cargo update --package zendo`. `Cargo.lock` gets the same version.
3. Stage `Cargo.toml` and `Cargo.lock` with the commit.

## Checks

- `.githooks/pre-commit` runs `.githooks/version-bump.sh staged`: the staged version is above the
  version of the parent commit, and `Cargo.lock` holds the same version. `git commit --amend`
  compares with the parent of the amended commit. A rebase and a cherry-pick are not checked.
- The workflow `.github/workflows/version.yml` runs `.githooks/version-bump.sh <base> <head>`: it
  checks every commit of a push and of a pull request, and the head of a pull request against its
  base. Merge commits are not checked.
