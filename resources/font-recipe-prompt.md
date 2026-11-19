zendo needs a recipe to set the font of my terminal. My terminal is `{terminal}`.

Write the recipe as the table `[{terminal}]` in the file `{recipes}`. Keep the other tables of the
file as they are.

A recipe sets a font in one or more of these ways, in this order:

- A file: `file` (the path; `~/` is the home folder), `prefix` and `line`. zendo replaces the first
  line of the file that starts with `prefix`, or adds `line` at the end. With `whole = true`,
  `line` is the whole content of the file.
- A command: `command`, a list of the program and its arguments. zendo runs it without a shell.
- An escape sequence: `escape`, which zendo writes to the terminal.

`reload` is an optional command that makes the terminal load its config again; zendo runs it
last. In each text, `{font}` stands for the family name of the font, and `{size}` for the current
size of the font: then the recipe needs a `size` command whose output ends with the size.

The fonts, with their family names:
{fonts}

Please:

1. Find out how `{terminal}` sets its font, and where its config is on this machine.
2. If the terminal needs a change once before a recipe can work, such as a handler in its config,
   show me the change and ask me before you make it.
3. Ask me before each change of a file other than `{recipes}`.
4. Write the recipe, and tell me when it is ready, so that I can end this session.
