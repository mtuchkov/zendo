# zendo

A quiet room for architecture thinking, in your terminal.

![zendo: read a note, ask the agent about a selection, keep its proposal, write](docs/media/zendo.gif)

<sub>A session with a scripted agent. More screens are in [docs/media](docs/media).</sub>

A zendo is the hall where Zen practitioners sit for zazen, seated meditation.
Deep thinking about the architecture of a project feels much the same: you sit
with a problem until it settles. zendo is a minimal, distraction-free place for
that work. It helps with three things only:

1. **Jot** down a thought the moment the insight comes.
2. **Think** it through, alone or with an AI agent as a sharp reviewer.
3. **Build**: when the thought is ready, turn it into action.

## Jot

Whenever you want to jot something down, run `zendo`, press `n` and start typing.
Skip the title: when you leave write mode, the agent adds one. A doc outside the notes
folder also gets its front matter.

There are two ways to keep notes:

- **In the project repo.** No setup: the notes go to `docs/notes`, into git and through
  review with the code they describe.
- **In your private repo.** zendo knows which project you are working in, and shows only
  the notes of that project. It commits and pushes in the background.

  ```sh
  zendo setup --notes-repo git@github.com:you/notes.git
  ```

Outside any repo, `zendo` opens a scratchpad.

## Think

This is the main part of zendo: it keeps you on what is important.

- **Nothing in the way.** A minimal terminal app: no other tool to open, no clutter.
  Nearly every feature is one key away; `?` lists the keys.
- **Where you left off.** A short recap of each note: what it is about, what it decides,
  what changed, and the open question. It tells you where you stopped yesterday. The delta
  (`d`) shows word by word what changed since you last worked on the note.
- **The agent one key away.** `ctrl-shift-c` opens the chat. Select with `ctrl-shift-h j k l` or `ctrl-shift` and the arrows, then `a` asks
  about the selection and `ctrl-shift-a` (or `alt-a`) adds it to the chat. Write mode saves by itself.
- **Docs that stay short.** Past the word limit (1,000 by default) the text turns red, and
  `S` asks the agent to split the doc and keep each part at one level of detail.

Tell the agent how to review you in `.zendo/prompt.md`: requirement style, tone, how blunt.

## Build

When the design holds, press `ctrl-o`. Claude Code starts with the whole discussion behind
it, so you don't paste or explain anything. You're the coder now. Exit Claude Code and zendo
comes back, and the delta shows what the implementation changed in your notes.

## Project structure

Put a `project-structure.toml` at the root of the repository: it says what lives where. zendo shows
its main materials as the file tree, so you find your architecture documents in a few keys. It also
tells agents where a new document goes, so they do not have to guess.

```sh
zendo doctor --fix    # the agent fixes the file, or writes it from your project-structure.md
```

## Terminal

zendo is a TUI because the terminal is where you already are: next to your editor, in tmux,
over SSH. Files, note, chat and delta sit side by side.

## AI

zendo talks to its agent over the Agent Client Protocol. Today it supports Claude, the
default, and Codex:

```sh
zendo setup                  # Claude: log in with `claude auth login`
zendo setup --agent codex    # Codex: log in with `codex login`
```

When an agent cannot do something, only that key is off. With Codex, that is `ctrl-o`
for now.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/mtuchkov/zendo/main/install.sh | sh
```

You need Node.js 22 LTS (18+ works), and Claude Code or Codex logged in.
`zendo doctor` checks each step of the setup. When a new version is out, the top bar
says so: press `U`. For proxies and certificates, see [INSTALLATION.md](INSTALLATION.md).

## More

- [docs/zendo/zendo.md](docs/zendo/zendo.md): the specification (draft).
- [INSTALLATION.md](INSTALLATION.md): install, upgrade, keys, code highlighting, the notes
  repo, certificates, removal.
