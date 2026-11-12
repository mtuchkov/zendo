## Project structure

`project-structure.toml` at the root of the repository says what lives where: for each folder or
key file, what it is (`about`) and the rules for agents (`claude`). The format is in the zendo
documentation of `project-structure.toml`.

- Before you create, move or rename a file, read `project-structure.toml` and put the file where it
  says.
- Follow the `claude` rules of the project and of each entry that covers the files that you change.
- Do not change `project-structure.toml` unless the user asks for it.

`zendo doctor` writes this section and replaces it when zendo changes. Write your own text outside it.
