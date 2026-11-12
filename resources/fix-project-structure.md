You are fixing the project structure file of a repository for zendo, a notes app.

Task: make {{FILE}} a valid project-structure.toml that follows the format below.

{{MODE}}

Rules:

- Edit only {{FILE}}. Do not change, create or delete any other file. Do not run commands.
- Keep the meaning and the words of the human text (`about`, `claude`, `[meta]`, comments). Move text to the
  correct key; do not rewrite it, shorten it or invent new facts.
- Keep every folder and file that the current text describes. Do not add entries for folders that the text
  does not describe, except when a `main` entry needs a path that exists.
- Map keys with another name to the correct key: for example `description` to `about`, `instructions` to
  `claude`, `title` or `name` (in an entry) to `label`, `dir` or `folder` to `path`, `primary` or `pinned` to `main`.
- Paths are relative to the repository root, without a leading `/` or `./`. Use the folder list below to
  correct a path that does not exist, but only when the correct path is clear.
- When you do not know which value is correct, keep the text in a `#` comment next to the entry, and say so
  in your answer.
- Keep the order of the entries. The order of the `main` entries is the reading order.
- After the edit, answer in at most 5 short lines: what you changed, and what you could not fix.

What zendo found:

{{PROBLEMS}}

Folders of the repository (two levels, .gitignore respected):

{{TREE}}

The format:

{{FORMAT}}
