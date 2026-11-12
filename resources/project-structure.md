# The project-structure.toml format

`project-structure.toml` sits at the repository root. It describes the folders and key files of
the project for three readers:

- people read `about`;
- zendo shows the entries with `main = true` as the default file tree, in the order of the file;
- Claude gets the `claude` rules of the project and of every entry that covers a file.

## Tables

| Table | Required | Content |
|---|---|---|
| `[meta]` | no | Free facts about the file, for example `date` or `note`. Any keys. |
| `[project]` | no | `name`, `about` (for people), `claude` (rules for the whole repository). |
| `[[entry]]` | no, any number | One folder or file each. |

No other top-level tables or keys.

## Entry keys

| Key | Type | Required | Content |
|---|---|---|---|
| `path` | string | yes | A folder or a file, relative to the repository root. No leading `/` or `./`, no `..`. `*` and `**` are allowed. |
| `label` | string | no | A short name for the file tree. Default: the last part of `path`. A rule names an entry by its label in double quotes, as "Code reviews". |
| `about` | string | no | What this is, for people. |
| `claude` | string | no | What Claude must do or not do here. Imperative sentences. |
| `main` | boolean | no | `true`: a main material, shown at the top level of the default file tree. |
| `words` | integer | no | The word limit of the documents here, in place of 1,000. `0`: no limit. The most specific entry that sets it wins. |
| `template` | string | no | The template of new documents here, from the repo root, as `docs/guides/templates/specification.md`. A new file in zendo starts from it, and the agent writes a new document from it. The most specific entry that names one wins. |

No other keys in an entry.

## Rules

- Each `path` appears once.
- Each `label` appears once. Case does not matter.
- A `main` entry names a folder or file that exists. A `main` entry has no `*`.
- The order of the `main` entries is the reading order of the project: put the first things to read first.
- An entry names a folder. It names a file only when the file has a meaning for the whole project, such
  as `README.md`, `AGENTS.md` or `docs/architecture/ARCHITECTURE.md`, or to set `words` for one file.
  Describe what lives in a folder, not each file in it.
- Use multi-line strings (`"""…"""`) for long `about` and `claude` text.
- Comments (`#`) are allowed anywhere.

## Example

```toml
[meta]
date = "2026-01-01"

[project]
name = "example"
about = "What the project is."
claude = """
- Rules that hold in the whole repository."""

[[entry]]
path = "README.md"
label = "README"
about = "What the project is and how to start."
main = true

[[entry]]
path = "docs/design"
label = "Design"
about = "Design notes: one file for each decision."
claude = "Write each decision with its reason."
main = true

[[entry]]
path = "generated/*"
about = "Generated files."
claude = "Do not edit generated files."
```
