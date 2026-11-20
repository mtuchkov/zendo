---
status: "implemented"
approved: "2026-11-20"
revision: "1"
date: "2026-11-20"
parent: "zendo.md"
prev: "agent-env.md"
next: "agent-profiles.md"
level: "how zendo reads the `.envrc` file: the lines that it takes, the values, the lines that it skips"
note: "Split from agent-env.md on 2026-11-20, after the code review of the agent environment. ENV-4 and ENV-7 are as the user approved them on 2026-11-20; ENV-8 is the decision of the user of 2026-11-20 in chat; ENV-9 is a fix of the agent. The user approved the document on 2026-11-20 in chat."
---

# The `.envrc` file

The `.envrc` at the root of the main checkout is one source of the
[agent environment](agent-env.md). Many repositories have the file for `direnv`, so it is a shell
script. zendo does not run it.

## The reading

Design (User, 2026-11-19): zendo reads `.envrc` itself and uses no other program for it. So a
repository cannot run code through the text of the file.

**Variable lines.** zendo takes only the lines `export NAME=value` and `NAME=value`. A comment
can follow the value.

**Values.** A value in single quotes counts as it is written. In a value in double quotes or
without quotes, zendo puts the value of each `$NAME` and `${NAME}` itself: the variable of a line
above, else the variable of the environment of zendo, else an empty text. `$PWD` is the folder of
the file, and `~` at the start of a path is the home folder. So the common line
`export PATH="$HOME/bin:$PATH"` works. A value that needs the shell, as `$(…)`, `${NAME:-x}`,
`~name` or a text in backquotes, makes the line no variable line. So does a variable that a
skipped line sets: zendo does not know its value. A value for one command, as
`NAME=value command`, does not change the variable.

**Blocks.** What a line in a block of the shell does depends on the run of the script. zendo
takes no line of a block: a condition, a loop, a function, a here-document, a list in
parentheses or a text over several lines, and a line that continues the line before, which ends
with `\`, `&&`, `||` or `|`.

**Skipped lines.** zendo skips every other line, and shows the numbers of the skipped lines once,
at its start and in `zendo doctor`. The messages of the start show one after another.

## Architecturally significant requirements

| ID    | Pattern    | Requirement | Source |
| ----- | ---------- | ----------- | ------ |
| ENV-4 | Ubiquitous | The system shall take from `.envrc` only the lines `export NAME=value` and `NAME=value`, and shall not run the file or any other program to read it. | User, 2026-11-19 |

### Acceptance tests

**`envrc_is_read_not_run`** (unit test; verifies ENV-4, ENV-7, ENV-8)

- Given a `.envrc` with `export A=1`, `B='two words'`, `C="$HOME/c"`, `D='$HOME ~'`, a comment,
  and the line `touch marker`.
- When zendo builds the agent environment.
- Then the environment has `A=1`, `B=two words`, `C` in the home folder and `D` as written, the
  file `marker` does not exist, and zendo shows once that line 5 of `.envrc` was skipped.

## Detailed requirements

| ID    | Pattern    | Requirement | Source |
| ----- | ---------- | ----------- | ------ |
| ENV-7 | Unwanted   | If a line of `.envrc` is not empty, not a comment, and not a variable line, then the system shall skip the line and shall show the file and the line number once. | Design |
| ENV-8 | Ubiquitous | The system shall put into a value of `.envrc` that is not in single quotes the value of each `$NAME` and `${NAME}`, and the home folder for `~` at the start of a path, and shall skip a line whose value needs the shell or a variable that a skipped line sets. | User, 2026-11-20 |
| ENV-9 | Ubiquitous | The system shall skip each line of `.envrc` that stands in a block of the shell: a condition, a loop, a function, a here-document, a list in parentheses or a text over several lines, and a line that continues the line before. | Design |
