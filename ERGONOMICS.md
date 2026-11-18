# Ergonomics

Ergonomics is not one feature of zendo. It is a requirement for every part of it: each key, hint,
pane, message and default. zendo exists so that you stay in the document while the agents work. A
detail that makes you stop, think about the tool, or leave the terminal works against that goal.

If you feel that something can be better, you are welcome to
[open an issue](https://github.com/mtuchkov/zendo/issues). Small things count: one extra key press,
a hint that names the wrong key, a pane that opens in the wrong place. Tell us what you did, what you
expected, and what got in the way.

## What is uncommon

These features are rare in editors, terminals and agent tools, and they have the most effect on how
zendo feels. The most important come first.

1. **Write mode looks like read mode.** Only the line with the cursor shows its markdown source.
   When you change the mode, the cursor stays on the same char, at the same height on the screen.
   You edit the real text and never lose your place.
2. **The delta starts at your last real visit.** zendo tells a short look at a file from real work
   on it. The delta shows, word by word, what changed since you last worked on the file, also when
   Claude Code, git or another editor made the change. You do not read the whole file again to see
   what the agent did.
3. **Recaps while you are away.** zendo writes a short recap of a file while you and the agent are
   idle: what the file is about and what its changes do. When you come back, it is there.
4. **Say it here.** `alt-s` puts the place of the cursor into the chat as a chip. You write the
   idea; the agent writes it at that place, in the style and the level of detail of the document,
   as one edit that changes nothing else.
5. **One thread for the chat and Claude Code.** Each agent session is one Claude Code session.
   `ctrl-o` opens Claude Code with the whole discussion behind it, also in the middle of an answer:
   Claude Code starts when the turn ends, and `ctrl-o` twice stops the turn and lets Claude Code
   continue it. After `alt-q`, the chat shows the prompts, the steps and the answers of Claude Code.
6. **Ask during an answer.** A question that you send while the agent answers waits in a queue.
   `esc` stops the answer and gives you the queue back to edit.
7. **The agent never runs a command.** `F5` puts a command into your shell, and you press `enter`.
   The agent sees the output only when you put it in with `alt-o`: the command, the exit code, and
   the output without colors.
8. **A call only when you do not look.** The bell rings and the session button blinks only for a
   session that is not on the screen, or while the terminal has no focus. A turn that you watch
   makes no noise.
9. **Parallel sessions that do not collide.** Each of the five agent sessions changes code only in
   its own git worktree. The file tree shows the worktree of the session and its changed files;
   hold `m` to see the document in `main`.
10. **Tables are cells, not lines.** A drag down a column copies only that column, with a tab
    between cells, and the selection keys stop at the edge of a cell. A drag with `alt` puts the
    selection into the chat in place of the clipboard.
11. **Every hint names your key.** Each hint, help text and message takes its key from your keymap,
    and an action without a key shows no hint. `ctrl-c` copies and never quits.
12. **Documents that stay short.** Past the word limit, the text turns red. `S` asks the agent to
    split the document by the Keep the Altitude rule: one level of detail in each document, with
    every requirement and its ID kept.

## What zendo does today

The features are in groups. In each group, the features with the most effect come first. The keys
are the default keys; the [keymap](#keymap) can change all of them.

### Stay in the flow

- **No save key.** Write mode and the note float save by themselves after a pause and when you
  leave them.
- **No question for each edit.** The agent edits the files at once. You check the result in the
  delta (`d`), word by word.
- **Ask at any time.** A question that you send while the agent answers goes to a queue and goes
  when the turn ends. `esc` stops the answer and puts the queue back into the input.
- **Work goes on in the background.** Claude Code sessions and terminal commands keep running
  when you go back to the document. `ctrl-o` works also while the chat answers: Claude Code starts
  when the turn ends. `ctrl-o` twice stops the turn, and Claude Code continues it. After `alt-q`,
  the chat shows what Claude Code did: each prompt, the steps of each turn, and the answer.
- **zendo calls you back.** When a turn or a command ends while you look elsewhere, zendo rings
  once and the session button blinks until you look. On a Mac, zendo plays a system sound, also
  where the bell of the terminal is off. `[sound]` in the top line turns the sound off.
- **Where you left off.** A recap above the note says what the file is about and what changed.
  zendo writes recaps while you are away, not while you work. Each agent session opens its last
  document again, at the same place, and zendo starts there.
- **The agent does the chores.** It writes the title of a new note, the front matter of a new
  document, and a one-word name for each agent session. A light model does this in the background.
- **The same place in both modes.** Read mode and write mode keep the cursor on the same char, at
  the same height on the screen.
- **A fast start.** The start, `zendo setup` and `zendo doctor` do not use the network when
  everything is installed. The check for a new version runs in the background.
- **Quiet music.** zendo plays calm lofi music that it makes itself, with no network. The music
  pauses while the terminal has no focus, and the call of a finished turn is a chime in its key.
  `[music]` or `alt-m` stops and starts it. zendo asks one time, at the first start, whether to play
  it at each start.

### Jot and write

- `alt-n` opens a new note in a float, from every pane and mode, also over a full-screen Claude
  Code session or terminal. The screen behind goes dim, and the float shows no key hints.
- Start with the text, not the title: the agent writes the title. `esc` saves the note and brings
  back the screen as it was: the same file, pane, cursor and selection.
- Write mode looks like read mode. Only the row with the cursor shows the markdown source, and it
  keeps the wrap of its paragraph.
- A long line breaks at a word, under its list marker or quote mark.
- `ctrl-b`, `ctrl-i` and `ctrl-u` mark the selection, or the word at the cursor, and take the mark
  away when it is there.
- `alt-s` asks the agent to write here, or to write the selection again, in the style of the
  document.
- In a code file, `enter` keeps the indent and long lines do not break.

### Read

- The text reflows to the width of the pane, also when the file wraps lines by hand. The column is
  76 chars by default; `alt-=` and `alt--` change it for the repository.
- Tables get solid borders, and their rows have alternate backgrounds.
- Code blocks, code files and the front matter show in colors. A language is a plugin file.
- Files that change on disk, from an agent or from git, update on the screen without a restart.
- Past the word limit, the text turns red. `S` asks the agent to split the document.
- `ctrl-r` twice: the agent marks the lines that confuse a reader, and says why.
- Hold `m` to see the version of the document in `main`, and `u` to see the version on disk. Let
  go, and the version of the session comes back.
- `j` `k` `g` `G`, `space` and `b` move as in a pager; `[` and `]` go to the previous and the next
  note. `t` changes the theme; there are four.

### Select and copy

- A selection in a table keeps to its columns. A drag down a column copies only that column, with
  a tab between cells. The selection keys stop at the edge of a cell; two quick presses go on.
- A mouse release copies, also over SSH (OSC 52). A drag with `alt` puts the selection into the
  chat as a chip; a drag with `ctrl` selects whole lines. While you drag, the status bar names
  these keys.
- A drag selects only in the pane where it started.
- `ctrl-c` copies and never quits. While text is selected where you write, `c` copies and `x`
  cuts.

### Talk to the agent

- The open file goes with the question, with the rules of the project for that file. You do not
  attach it. When the agent session has a worktree, the question takes the version of that
  worktree.
- Each agent session changes code only in its own git worktree, so five sessions work in one
  repository and do not change the files of each other. zendo gives each agent the rules of its
  worktree at the start.
- `a` asks about the selection or the paragraph. `alt-a` adds the selection to the input as a
  chip.
- `@` completes a file name, also with a typo. Pasted paths and dropped files become chips, and so
  does a pasted image; the image chip gets a short name of what it shows. Arrows and `backspace`
  move over a whole chip.
- `tab` goes between the input and the transcript. More presses go around the panes and come back
  to the chat. The transcript has the keys of read mode.
- Five agent sessions on `F6` to `F10`, each one thread for the whole repository. Each button shows
  the name of its session. `alt-]` and `alt-[` (`alt-}` on a Mac) go to the next and the previous
  session or terminal, also on the full screen.
- `s` on a note: the agent decides which specifications to update or write, says its plan, then
  edits.
- `alt-enter`, `shift-enter`, `ctrl-j`, or `\` and `enter` add a line. `/` shows the commands of
  the agent; `ctrl-t` starts a document from a template.
- When the agent cannot do a feature, only that key is off.

### Run commands

- A green `▶` marks a shell command in an answer. `F5` puts it into a terminal and you press
  `enter`; `alt-F5` runs it at once. `$ ` prompts are removed. The agent never runs a command.
- `alt-o` puts the result of the last command into the chat input: the command, the exit code and
  the output, without colors and cut to the last 500 lines. `alt-o` again takes the next terminal.
- A terminal shows in the chat pane, and the document stays on the screen. `ctrl-o` makes it full
  screen; `alt-q` goes back. On the full screen, every key goes to the program, except `alt-q`,
  `F5` to `F10`, the session keys, `alt-n` and `alt-/`.
- Up to four `[>]` terminals beside the `F5` terminal. A command keeps running when you leave its
  terminal.
- The mouse and the wheel work in the programs of a terminal, also in full-screen programs.
- No tmux and no server process. zendo does not change your shell files.

### Find files

- `alt-/` opens the find panel in every pane and mode; `/` does the same in read mode. With no
  query, it lists your recent files, so `alt-/` and `enter` go back to the previous file. A query
  finds paths first, then text, with a preview. `#` finds a tag, and `?` asks the agent about all
  notes.
- The file tree shows the main materials of `project-structure.toml` first, in reading order, with
  the `about` of each item. `v` shows the next view: structure, docs, text, the worktree of the
  session, and its changed files.
- Folders start closed, and the tree follows the open document: it opens the folders above it and
  selects its row. `*` and `_` open and close every folder.
- In the tree, `/` filters by file name. `Δ` marks the files that changed since your last visit.
- `F2` renames, `m` moves, `F3` and `F4` make a file or a folder, with `tab` completion. A new
  document starts from the template of its folder.
- The view, the open folders, the recent files, the widths and the layout stay for each
  repository.

### Layout

- Two layouts, wide screen and standard, and no other settings. `w` switches.
- When the panes do not fit, zendo hides them from the edges by turns. The pane in focus stays.
- `ctrl-←` and `ctrl-→`, or a drag of the separator, change the width of a pane.
- The wheel scrolls the pane under the pointer. Fast trackpad scrolls stay smooth.

### Install and upgrade

- One install line that never waits for input. It also installs Rust and sets the `PATH`.
- Each failed check of `zendo setup` and `zendo doctor` says what failed and how to fix it.
  `zendo doctor` changes nothing. `zendo doctor --fix` repairs what needs no judgement and asks no
  question; `--fix --claude` lets the agent repair the rest.
- On a company network, setup tries the certificate store of the system by itself.
- When a new version is out, the top bar shows `[upgrade]`. A click upgrades zendo and starts it
  again.
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
- The default keymaps of Linux and macOS differ in one key: the previous session is `alt-}` on a
  Mac, because Apple Terminal sends `alt-[` as the start of an escape code.

The defaults follow a few rules: letters as in a pager in the panes where you read, editor keys
with `ctrl` where you type, `alt` for the actions that work in every pane, and the F keys for the
sessions and the terminals.

zendo counts how often you do each action, by pane, and by key or mouse. The counts stay on your
disk and hold no text, file names or paths. They help to decide which key hints zendo shows, and
where.
`usage = false` in the config turns the counting off. See
[INSTALLATION.md](INSTALLATION.md#usage-counts).

Next: a keymap wizard, `zendo doctor --keys`, will find the keys that the OS or the terminal takes
before zendo, and move each action to a free key
([specification](docs/zendo/cli/keymap-wizard.md)).

We accept a suggestion for the default keymap when you are confident that it makes zendo better
for most of its users, not only for you. Try it in your own keymap first. In the issue, name the
action, its key now, the key you suggest and why most users gain from it. Name the terminals where
you tried it, with and without the kitty protocol, and the keys of the system, the terminal or tmux
that it can conflict with. When a change fits only your habits, keep it in your own keymap.
