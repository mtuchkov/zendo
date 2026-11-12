#!/bin/sh
# zendo: install or upgrade in one line. Safe to run again.
#
#   curl -fsSL https://raw.githubusercontent.com/mtuchkov/zendo/main/install.sh | sh
#
# Environment:
#   ZENDO_REPO        git URL to build from (default: HTTPS, then SSH)
#   ZENDO_NO_SETUP=1  do not run `zendo setup` at the end
#   ZENDO_NO_RC=1     do not add ~/.cargo/bin to your shell rc file

set -eu

SSH_REPO="ssh://git@github.com/mtuchkov/zendo.git"
HTTPS_REPO="https://github.com/mtuchkov/zendo.git"
RUST_MIN="1.89"

if [ -t 1 ]; then B="$(printf '\033[1m')"; D="$(printf '\033[2m')"; R="$(printf '\033[0m')"; else B=""; D=""; R=""; fi
say() { printf '%szendo%s %s\n' "$B" "$R" "$*"; }
note() { printf '%s     %s%s\n' "$D" "$*" "$R"; }
die() { printf '%szendo%s %s\n' "$B" "$R" "$*" >&2; exit 1; }
has() { command -v "$1" >/dev/null 2>&1; }

# 1. git
has git || die "git is not installed. Install git, then run this line again."

# 2. Rust
CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
if ! has cargo && [ -x "$CARGO_BIN/cargo" ]; then
	PATH="$CARGO_BIN:$PATH"
fi
if ! has cargo; then
	has curl || die "Rust is not installed, and curl is missing to install it. Install Rust from https://rustup.rs"
	say "installing Rust with rustup (minimal profile)…"
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --no-modify-path >/dev/null
	PATH="$CARGO_BIN:$PATH"
fi
rust_v="$(rustc --version 2>/dev/null | awk '{print $2}')"
if [ "$(printf '%s\n%s\n' "$RUST_MIN" "$rust_v" | sort -V | head -n1)" != "$RUST_MIN" ]; then
	if has rustup; then
		say "Rust $rust_v is older than $RUST_MIN: updating the stable toolchain…"
		rustup update stable >/dev/null
	else
		die "Rust $rust_v is older than $RUST_MIN. Update Rust, then run this line again."
	fi
fi

# 3. Where to build from. Run from a clone of zendo (./install.sh), the script builds that
#    clone and does not go to GitHub. Piped (`| sh`), it has no folder and builds main from GitHub.
is_zendo_repo() {
	git -C "$1" rev-parse --is-inside-work-tree >/dev/null 2>&1 &&
		grep -qx 'name = "zendo"' "$1/Cargo.toml" 2>/dev/null &&
		[ -f "$1/src/bin/zendo-mock-agent/main.rs" ]
}
SRC=""
case "$0" in
*install.sh)
	here="$(cd "$(dirname "$0")" 2>/dev/null && pwd || true)"
	if [ -z "${ZENDO_REPO:-}" ] && [ -n "$here" ] && is_zendo_repo "$here"; then SRC="$here"; fi
	;;
esac

# Build and install. HTTPS needs no account; SSH, the fallback, uses your key.
BIN="${CARGO_INSTALL_ROOT:-${CARGO_HOME:-$HOME/.cargo}}/bin"
old="$(zendo --version 2>/dev/null || "$BIN/zendo" --version 2>/dev/null || "$BIN/dawn" --version 2>/dev/null || true)"
export CARGO_NET_GIT_FETCH_WITH_CLI=true
# Keep the build folder, so that `zendo upgrade` later builds only zendo.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/zendo/target}"
# Never wait for a prompt: no passphrase or host-key question in a piped script.
export GIT_TERMINAL_PROMPT=0
export GIT_SSH_COMMAND="${GIT_SSH_COMMAND:-ssh -o BatchMode=yes -o ConnectTimeout=10 -o StrictHostKeyChecking=accept-new}"
build() {
	say "building zendo from $1 (a few minutes the first time)…"
	cargo install --git "$1" --locked --force --quiet zendo
}
if [ -n "$SRC" ]; then
	at="$(git -C "$SRC" rev-parse --short HEAD)"
	if [ -n "$(git -C "$SRC" status --porcelain --untracked-files=no)" ]; then at="$at, with your uncommitted changes"; fi
	say "building your clone $SRC ($at): no GitHub…"
	cargo install --path "$SRC" --locked --force --quiet || die "the build failed. See the cargo output above."
	# A clone is where you commit: turn on the commit message check, once.
	if [ -z "$(git -C "$SRC" config core.hooksPath 2>/dev/null)" ]; then
		git -C "$SRC" config core.hooksPath .githooks && note "commit messages: the check in .githooks is on in this clone (see .skills/git-commit)."
	fi
elif [ -n "${ZENDO_REPO:-}" ]; then
	build "$ZENDO_REPO" || die "the build failed. See the cargo output above."
elif ! build "$HTTPS_REPO"; then
	note "HTTPS did not work; trying SSH."
	build "$SSH_REPO" || die "could not fetch the repo from GitHub. Check your network or proxy, then run this line again."
fi

# 4. PATH
rc_note=""
case ":$PATH:" in
*":$BIN:"*) ;;
*)
	PATH="$BIN:$PATH"
	if [ -z "${ZENDO_NO_RC:-}" ]; then
		case "${SHELL:-}" in
		*zsh) rc="$HOME/.zshrc" ;;
		*bash) rc="$HOME/.bashrc" ;;
		*) rc="$HOME/.profile" ;;
		esac
		line="export PATH=\"$BIN:\$PATH\""
		if ! grep -qsF "$line" "$rc"; then
			printf '\n# zendo (cargo bin)\n%s\n' "$line" >>"$rc"
			note "added $BIN to PATH in $rc."
		fi
		rc_note="$line"
	fi
	;;
esac

new="$("$BIN/zendo" --version)"
# zendo was called dawn before 0.10. Remove the old command; zendo moves its folders on the first run.
if cargo install --list 2>/dev/null | grep -q '^dawn v'; then
	cargo uninstall --quiet dawn && note "removed the old \`dawn\` command: it is \`zendo\` now."
fi
if [ -z "$old" ]; then
	say "installed $new"
elif [ "$old" = "$new" ]; then
	say "reinstalled $new"
else
	say "upgraded: $old → $new"
fi

# The shell that ran this line still has the old PATH.
finish() {
	if [ -n "$rc_note" ]; then
		say "open a new shell, or run this in the current one:"
		printf '     %s\n' "$rc_note"
	fi
}
trap finish EXIT

# 5. Claude: adapter, login, handshake.
if [ -n "${ZENDO_NO_SETUP:-}" ]; then
	exit 0
fi
if ! has node; then
	note "Node.js is not installed. zendo needs Node 22 for Claude."
	note "Install Node.js, then run: zendo setup"
	exit 0
fi
"$BIN/zendo" setup || {
	note "Fix the step above, then run: zendo setup"
	exit 0
}
