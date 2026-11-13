# Ergonomics

Ergonomics is not one feature of zendo. It is a requirement for every part of it: each key, hint,
pane, message and default. zendo exists so that you stay in the document while the agents work. A
detail that makes you stop, think about the tool, or leave the terminal works against that goal.

If you feel that something can be better, you are welcome to
[open an issue](https://github.com/mtuchkov/zendo/issues). Small things count: one extra key press,
a hint that names the wrong key, a pane that opens in the wrong place. Tell us what you did, what you
expected, and what got in the way.

## What zendo does today

The features are in groups. In each group, the features with the most effect come first. The keys
are the default keys; the [keymap](#keymap) can change all of them.

### Stay in the flow

- **No save key.** Write mode saves by itself after a pause and when you leave it.
- **No question for each edit.** The agent edits the files at once. You check the result in the
  delta (`d`), word by word.
- **Ask at any time.** A question that you send while the agent answers goes to a queue and goes
  when the turn ends. `esc` stops the answer and puts the queue back into the input.
- **Work goes on in the background.** Claude Code sessions and terminal commands keep running
  when you go back to the document. `ctrl-o` works also while the chat answers: Claude Code starts
  when the turn ends.
- **zendo calls you back.** When a turn or a command ends while you look elsewhere, the terminal
  bell rings once and the session button blinks until you look. `[sound]` turns the bell off.
- **Where you left off.** A recap above the note says what the file is about and what changed.
  zendo writes recaps while you are away, not while you work.
- **The agent does the chores.** It adds the title of a new note and the front matter of a new
  document when you leave write mode.
- **The same place in both modes.** Read mode and write mode keep the cursor on the same char, at
  the same height on the screen.
- **A fast start.** The start, `zendo setup` and `zendo doctor` do not use the network when
  everything is installed. The check for a new version runs in the background.

### Read

- The text reflows to the width of the pane, also when the file wraps lines by hand. The column is
  76 chars by default; `alt-=` and `alt--` change it for the repository.
- Tables get solid borders, and their rows have alternate backgrounds.
- Code blocks, code files and the front matter show in colors. A language is a plugin file.
- Files that change on disk, from an agent or from git, update on the screen without a restart.
- Past the word limit, the text turns red. `S` asks the agent to split the document.
- `j` `k` `g` `G`, `space` and `b` move as in a pager; `[` and `]` go to the previous and the next
  note. `t` changes the theme; there are four.

### Write

- Write mode looks like read mode. Only the row with the cursor shows the markdown source, and it
  keeps the wrap of its paragraph.
- A long line breaks at a word, under its list marker or quote mark.
- `ctrl-b`, `ctrl-i` and `ctrl-u` mark the selection, or the word at the cursor, and take the mark
  away when it is there.
- `alt-s` asks the agent to write here, or to write the selection again, in the style of the
  document.
- In a code file, `enter` keeps the indent and long lines do not break.

### Select and copy

- A selection in a table keeps to its columns. A drag down a column copies only that column, with
  a tab between cells. The selection keys stop at the edge of a cell; two quick presses go on.
- A mouse release copies, also over SSH (OSC 52). A drag with `alt` puts the selection into the
  chat as a chip; a drag with `ctrl` selects whole lines.
- A drag selects only in the pane where it started.
- `ctrl-c` copies and never quits. While text is selected where you write, `c` copies and `x`
  cuts.

### Talk to the agent

- The open file goes with the question, with the rules of the project for that file. You do not
  attach it.
- `a` asks about the selection or the paragraph. `alt-a` adds the selection to the input as a
  chip.
- `@` completes a file name, also with a typo. Pasted paths and dropped files become chips, and so
  does a pasted image. Arrows and `backspace` move over a whole chip.
- `tab` goes between the input and the transcript. The transcript has the keys of read mode.
- Five sessions on `F6` to `F10`, each one thread for the whole repository. `alt-n` goes to the
  next one.
- `alt-enter`, `shift-enter`, `ctrl-j`, or `\` and `enter` add a line. `/` shows the commands of
  the agent; `ctrl-t` starts a document from a template.
- When the agent cannot do a feature, only that key is off.

### Run commands

- A green `▶` marks a shell command in an answer. `F5` puts it into a terminal and you press
  `enter`; `alt-F5` runs it at once. `$ ` prompts are removed. The agent never runs a command.
- `alt-o` puts the result of the last command into the chat input: the command, the exit code and
  the output. `alt-o` again takes the next terminal.
- A terminal shows in the chat pane, and the document stays on the screen. `ctrl-o` makes it full
  screen; `alt-q` goes back.
- The mouse and the wheel work in the programs of a terminal, also in full-screen programs.
- No tmux and no server process. zendo does not change your shell files.

### Find files

- The file tree shows the main materials of `project-structure.toml` first, in reading order, with
  the `about` of each item.
- `/` filters by the words of the path. `Δ` marks the files that changed since your last visit.
- `F2` renames, `m` moves, `F3` and `F4` make a file or a folder, with `tab` completion. A new
  document starts from the template of its folder.
- In the notes list, `/` searches all notes, `#` searches tags, and `?` asks the agent about all
  notes.
- The view, the open folders, the widths and the layout stay for each repository.

### Layout

- Two layouts, wide screen and standard, and no other settings. `w` switches.
- When the panes do not fit, zendo hides them from the edges by turns. The pane in focus stays.
- `ctrl-←` and `ctrl-→`, or a drag of the separator, change the width of a pane.
- The wheel scrolls the pane under the pointer. Fast trackpad scrolls stay smooth.

### Install and upgrade

- One install line that never waits for input. It also installs Rust and sets the `PATH`.
- Each failed check of `zendo setup` and `zendo doctor` says what failed and how to fix it.
  `zendo doctor --fix` repairs `project-structure.toml` with the agent.
- On a company network, setup tries the certificate store of the system by itself.
- The top bar says when a new version is out; `U` upgrades and starts zendo again.
- The private notes repo commits and pushes in the background, and never delays the screen.

## Keymap

`alt-?` or `f1` shows the keys of the pane you are in, in groups. The status bar always shows the
help key first.

A keymap is personal: hands, keyboards, terminals and habits are different. So zendo makes every
key configurable out of the box:

- `zendo keymap > ~/.zendo/keymap.toml` gives you the default keymap to start from. Name it in
  `~/.zendo/config.toml` with `keymap = "~/.zendo/keymap.toml"`. An action that your file leaves
  out keeps its default keys.
- `[keys.<pane>]` in the config changes one key without a file.
- Every hint, help text and message names the key of your keymap. An action without a key shows
  no hint.
- `zendo doctor` and the status bar report unknown keys and a key with two actions in one pane.
- Without the kitty keyboard protocol, `ctrl-shift` keys fall back to `ctrl`, and most actions
  have a second key that works in every terminal. [INSTALLATION.md](INSTALLATION.md#keys) has the
  details.

The defaults follow a few rules: letters as in a pager in the panes where you read, editor keys
with `ctrl` where you type, `alt` for the actions that work in every pane, and the F keys for the
sessions and the terminals.

We accept a suggestion for the default keymap when you are confident that it makes zendo better
for most of its users, not only for you. Try it in your own keymap first. In the issue, name the
action, its key now, the key you suggest and why most users gain from it. Name the terminals where
you tried it, with and without the kitty protocol, and the keys of the system, the terminal or tmux
that it can conflict with. When a change fits only your habits, keep it in your own keymap.
