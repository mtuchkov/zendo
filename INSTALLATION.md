# Installation

This page tells you how to install zendo, connect it to Claude, upgrade it, and remove it.

## One line

Install and upgrade use the same line:

```sh
curl -fsSL https://raw.githubusercontent.com/mtuchkov/zendo/main/install.sh | sh
```

The script:

1. Installs Rust with rustup (minimal profile) if `cargo` is missing, and updates Rust if it is older than 1.89.
2. Builds zendo from `main` over HTTPS. If HTTPS does not work, it tries SSH (your key).
3. Adds `~/.cargo/bin` to `PATH` in your shell rc file, if it is not there.
4. Runs `zendo setup`: the Claude adapter, the login check, the handshake.

Run from a clone of zendo (`./install.sh`), the script builds **that clone** and does not go to GitHub. It checks that its folder is the zendo repository: a git work tree with the `zendo` package and `src/bin/zendo-mock-agent/main.rs`. Piped (`| sh`), it builds `main` from GitHub.

You can run it again at any time. Settings: `ZENDO_REPO` (another git URL), `ZENDO_NO_SETUP=1`, `ZENDO_NO_RC=1`.

Once zendo is installed, upgrade from inside:

```sh
zendo upgrade            # builds only when main has a newer commit, then runs setup
zendo upgrade --force    # build again even when you have the newest commit
```

The rest of this page has the manual steps.

## Requirements

| Tool | Version | Why |
|---|---|---|
| Rust (`cargo`) | 1.89 or newer | To build zendo. Install it with [rustup](https://rustup.rs). |
| Node.js | 22 LTS (18 or newer works) | To run the Claude adapter. Node 22.15 or newer can use the system certificate store. |
| Claude Code | current | zendo uses your Claude Code login. |
| git | any | For the delta against `HEAD` and for upgrades. |

Optional: `wl-copy`, `xclip` or `xsel` on Linux, so that a mouse selection goes to the clipboard. On macOS, zendo uses `pbcopy`.

## Install by hand

1. Get the code:

   ```sh
   git clone git@github.com:mtuchkov/zendo.git
   cd zendo
   ```

2. Build and install `zendo` and `zendo-mock-agent` into `~/.cargo/bin`:

   ```sh
   cargo install --path . --locked
   ```

3. Make sure that `~/.cargo/bin` is on your `PATH`:

   ```sh
   zendo --version
   ```

   If your shell shows `command not found`, add this line to `~/.zshrc` (or `~/.bashrc`) and open a new shell:

   ```sh
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

4. Log in to Claude Code, if you did not do it before:

   ```sh
   claude auth login
   ```

5. Install the Claude adapter and check every step:

   ```sh
   zendo setup
   ```

   `zendo setup` installs the adapter once, with a fixed version, in `~/.local/share/zendo/agent`. Then it runs all the checks of `zendo doctor`.

6. Start zendo in a repository:

   ```sh
   cd ~/your-repo
   zendo
   ```

   zendo opens only when Claude works. If a check fails, zendo prints the failed step and its fix.

## Codex in place of Claude

```sh
zendo setup --agent codex      # installs @agentclientprotocol/codex-acp once; `agent = "codex"` in the config
codex login                    # or set OPENAI_API_KEY
```

Everything works the same, except the hand-off to the command line (`ctrl-o`): it is off with Codex.
`zendo setup --agent claude` goes back to Claude. Any other ACP agent works with `--agent "<command>"`.

## Keys

Every key of every pane is in the keymap; `?` (or `f1`) shows the keys of the pane you are in. zendo
ships its default keys as a file. To make your own keymap, start from it and name it in the config:

```sh
zendo keymap > ~/.zendo/keymap.toml
```

```toml
# ~/.zendo/config.toml
keymap = "~/.zendo/keymap.toml"
```

An action that your file leaves out keeps its default keys. To change one key without a file, name
the action in `~/.zendo/config.toml`; these keys go after the file:

```toml
[keys.read]
add_to_chat = ["+", "m"]
split = "ctrl-s"

[keys.write]
italic = "alt-i"
```

Panes: `global`, `read`, `write`, `chat`, `tree`, `delta`, `notes`. A key is `j`, `J`, `+`, `ctrl-b`, `ctrl-shift-b`,
`alt-i`, `shift-down`, `ctrl-shift-left`, `enter`, `esc`, `tab`, `space`, `pageup`, `f1`… `zendo doctor`
reports unknown panes, actions and keys. In terminals with the kitty keyboard protocol (kitty, Ghostty,
WezTerm, iTerm2 with CSI u) `ctrl-i` is italic; elsewhere it is Tab, so `alt-i` is italic too. The
same holds for `shift-enter` in the chat: without the protocol, use `alt-enter`, `ctrl-j`, or `\` and
`enter` for a new line. `ctrl-shift-b` (the file tree) and `ctrl-shift-h j k l` (select) also need
the protocol; elsewhere use `ctrl-b` or `e` in read mode, `H J K L`, or Shift and the arrows.

## Text width and font size

Read mode wraps the text at 76 chars, and write mode breaks a line that gets longer. Set another
width in `~/.zendo/config.toml`:

```toml
width = 90
```

`alt-=` and `alt--` change the width for one repository. The font size belongs to the terminal:
most terminals change it with `ctrl-+` and `ctrl--`.

## Code highlighting

Code blocks and the front matter show in colors. `sh`, `rust`, `java`, `python`, `go`, `yaml` and `json`
are on. Every language is a plugin, a Sublime syntax definition:

```toml
[highlight]
languages = ["sh", "rust", "java", "python", "go", "yaml", "json", "ruby", "sql"]
```

- zendo ships many more definitions; add their names to `languages` to turn them on.
- For any other language, put its `.sublime-syntax` file into `~/.zendo/syntaxes/`. It is on at once.
- `zendo doctor` lists the languages that are on, and the plugin files that do not load.

## A private notes repo

By default, zendo keeps the notes of a repository in its `docs/notes`. To keep all your notes in one
private git repository instead:

```sh
zendo setup --notes-repo git@github.com:you/notes.git   # zendo clones it into ~/.local/share/zendo/notes
zendo setup --notes-repo ~/notes                         # or use a clone you already have
```

- zendo keeps the notes of each repository in `<machine>/<path from your home>/` in the notes repo,
  and the scratchpad in `<machine>/~/`. It shows only the folder of this machine and repository.
- It commits and pushes every 15 seconds, in the background. To change the interval, set
  `sync_seconds` in the `[notes]` table of `~/.zendo/config.toml`.
- When you turn it on in a repository with notes in `docs/notes`, Claude moves them into the notes
  repo. Run `zendo notes move` to do that in another repository.
- `zendo notes consolidate` merges the folders of one project from all machines and clones, by its
  git origin. Claude does the merge.
- `zendo doctor` checks the notes repo. `zendo setup --notes-repo off` turns it off.

## Network: when zendo needs npm

zendo needs npm only once, to install the Claude adapter. After that, `zendo setup`, `zendo doctor` and the start of zendo do not use npm, also on a VPN. A new zendo version that pins a new adapter version needs npm once more.

If npm cannot reach its registry on your VPN, run `zendo setup` once off the VPN.

## If npm fails with a certificate error

On a company network, npm can fail with `UNABLE_TO_VERIFY_LEAF_SIGNATURE` or `SELF_SIGNED_CERT_IN_CHAIN`. A proxy replaces the certificates, and Node does not trust the proxy's CA.

- `zendo setup` installs the adapter again with `NODE_OPTIONS=--use-system-ca`. If that works, it saves the setting in `~/.zendo/config.toml`. You do not have to do more.
- If the system store does not have the CA either, get the CA file from your IT team and point Node at it:

  ```sh
  export NODE_EXTRA_CA_CERTS=/path/to/company-ca.pem   # add this to ~/.zshrc
  zendo setup
  ```

## Upgrade

### See which version you have

```sh
zendo --version
```

### Upgrade with one command

```sh
zendo upgrade
```

`zendo upgrade` compares the commit you installed with `main` on GitHub. It builds only when `main` is newer. Then it runs `zendo setup`, which also installs a new adapter version when the new zendo needs one.

### Upgrade a build from your clone

When zendo was built from your clone (`./install.sh` or `cargo install --path .`), `zendo upgrade` and `U` run `git pull --ff-only` in that clone and build it again. They do not switch you to a build from GitHub. If the pull cannot fast-forward (local commits or changes), zendo says so and builds nothing.

The update suggestion for such a build appears only when `main` on GitHub has a commit that your clone does not have. When your clone is ahead of `main`, there is no suggestion.

### Upgrade from the git repository by hand

1. Get the new code:

   ```sh
   cd path/to/zendo
   git pull
   ```

2. Build and install it over the old version:

   ```sh
   cargo install --path . --locked --force
   ```

   `--force` replaces the installed binaries, also when the version number did not change.

3. Check the result:

   ```sh
   zendo --version
   zendo doctor
   ```

4. If `zendo doctor` shows `adapter … zendo expects …`, the new version uses a different adapter. Install it:

   ```sh
   zendo setup
   ```

### Upgrade without a local clone

Cargo can build straight from GitHub:

```sh
cargo install --git https://github.com/mtuchkov/zendo.git --locked --force
zendo doctor
```

### What an upgrade keeps

An upgrade does not touch your data:

| Path | Content |
|---|---|
| `docs/notes/*.md` in each repository | Your notes |
| `.zendo/db.json` in each repository | Theme, layout, Claude thread ids, visits, delta bases, recaps |
| `.zendo/prompt.md` in each repository | Your instructions for Claude |
| `~/.zendo/config.toml` | Agent command and environment from `zendo setup` |
| `~/.local/share/zendo/agent` | The installed Claude adapter |
| `~/.cache/zendo/target` | The build folder; it makes the next upgrade faster. You can delete it. |

zendo migrates old files itself. For example, 0.3 reads `.zendo/state.json` from 0.2 and writes `.zendo/db.json` from then on.

### Version notes

| Version | What to do after the upgrade |
|---|---|
| 0.12.0 | Nothing. New: the keymap (`[keys.*]`), code highlighting (`[highlight]`), Codex (`zendo setup --agent codex`). The top-bar buttons are gone: every action has a key. |
| 0.11.1 | Nothing. The config moves from `~/.config/zendo/config.json` to `~/.zendo/config.toml` on the first run. |
| 0.10.x | zendo was called dawn before 0.10. Run the install line (or `./install.sh` in your clone) once: `dawn upgrade` cannot build the new name. The installer removes the `dawn` command. On the first run, zendo moves `~/.config/dawn`, `~/.local/share/dawn`, `~/.cache/dawn` and `.dawn/` of the repository to the new names. Use `ZENDO_*` in place of `DAWN_*` variables. |
| 0.4.x | Nothing. From now on, use `zendo upgrade`. |
| 0.3.x | Nothing. `.zendo/db.json` replaces `.zendo/state.json` on the first start. |
| 0.2.x | Run `zendo setup` once. Before 0.2, zendo used `npx` at each start. |

## Uninstall

```sh
cargo uninstall zendo
rm -rf ~/.zendo ~/.local/share/zendo ~/.cache/zendo
```

The `.zendo/` folder in each repository holds local state. Delete it if you do not want it. Your notes stay where they are.
