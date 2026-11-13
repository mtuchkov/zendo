#!/bin/sh
# The version in Cargo.toml increases with every commit. A patch bump is enough for a small change.
#   .githooks/version-bump.sh staged          the staged version is above the version of the parent
#   .githooks/version-bump.sh <base> <head>   for CI: the same for every commit in a range
# Merge commits are skipped. Exit 1 when a version does not increase.
set -eu

# Print the package version of the Cargo.toml in a revision ("" is the index). Print nothing if none.
version_of() {
	git show "$1:Cargo.toml" 2>/dev/null |
		awk '/^\[/ { pkg = ($0 == "[package]") }
		     pkg && /^version[ \t]*=/ { gsub(/^[^"]*"|".*$/, ""); print; exit }'
}

# Print the version of the package zendo in the Cargo.lock of a revision.
lock_version_of() {
	git show "$1:Cargo.lock" 2>/dev/null |
		awk '$0 == "name = \"zendo\"" { hit = 1; next }
		     hit && /^version[ \t]*=/ { gsub(/^[^"]*"|".*$/, ""); print; exit }'
}

# Exit 0 when version $1 is above version $2. A version is MAJOR.MINOR.PATCH, numbers only.
is_above() {
	printf '%s %s\n' "$1" "$2" | awk '{
		if ($1 !~ /^[0-9]+\.[0-9]+\.[0-9]+$/ || $2 !~ /^[0-9]+\.[0-9]+\.[0-9]+$/) exit 1
		split($1, a, "."); split($2, b, ".")
		for (i = 1; i <= 3; i++) if (a[i] + 0 != b[i] + 0) exit !(a[i] + 0 > b[i] + 0)
		exit 1
	}'
}

# Print the reason why revision $1 does not increase the version of revision $2. Print nothing if it does.
problem() {
	new=$(version_of "$1")
	old=$(version_of "$2")
	lock=$(lock_version_of "$1")
	if [ -z "$old" ]; then
		return 0
	elif [ -z "$new" ]; then
		echo "Cargo.toml has no package version."
	elif ! is_above "$new" "$old"; then
		echo "version $new is not above $old of the parent. Increase the version in Cargo.toml."
	elif [ "$lock" != "$new" ]; then
		echo "Cargo.lock has zendo ${lock:-?}, Cargo.toml has $new. Run: cargo update --package zendo"
	fi
}

# Exit 0 when a `git commit --amend` started this hook: find the git process above this one and
# look at its command line.
amends() {
	pid=$PPID
	for _ in 1 2 3; do
		case "$(ps -o comm= -p "$pid" 2>/dev/null)" in
		git | */git)
			case " $(ps -o args= -p "$pid" 2>/dev/null) " in
			*" --amend "*) return 0 ;;
			esac
			return 1
			;;
		esac
		pid=$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' ')
		[ -n "$pid" ] && [ "$pid" -gt 1 ] || return 1
	done
	return 1
}

check_staged() {
	gd=$(git rev-parse --absolute-git-dir)
	# A rebase or a cherry-pick replays commits that have a version already. CI checks the result.
	if [ -d "$gd/rebase-merge" ] || [ -d "$gd/rebase-apply" ] || [ -f "$gd/CHERRY_PICK_HEAD" ]; then
		return 0
	fi
	git rev-parse --verify --quiet HEAD >/dev/null || return 0
	parent=HEAD
	# `git commit --amend` replaces HEAD: the parent of the new commit is the parent of HEAD.
	if amends; then
		parent=$(git rev-parse --verify --quiet HEAD^) || return 0
	fi
	why=$(problem "" "$parent")
	[ -z "$why" ] && return 0
	echo "pre-commit: $why" >&2
	return 1
}

check_range() {
	base="$1"
	head="$2"
	if [ -z "$base" ] || [ "$base" = "0000000000000000000000000000000000000000" ] || ! git cat-file -e "$base^{commit}" 2>/dev/null; then
		# A new branch: the commits that main does not have.
		base=origin/main
	fi
	bad=0
	for c in $(git rev-list --no-merges "$head" --not "$base"); do
		why=""
		if git rev-parse --verify --quiet "$c^" >/dev/null; then
			why=$(problem "$c" "$c^")
		fi
		if [ -n "$why" ]; then
			echo "✗ $(git log -1 --format='%h %s' "$c")"
			echo "    $why"
			bad=1
		else
			echo "✓ $(version_of "$c") $(git log -1 --format='%h %s' "$c")"
		fi
	done
	# A pull request: the base branch can have new commits, and the head must be above them too.
	if [ "$(git rev-parse "$head")" != "$(git rev-parse "$base")" ]; then
		why=$(problem "$head" "$base")
		if [ -n "$why" ]; then
			echo "✗ head against base $(git rev-parse --short "$base"): $why"
			bad=1
		fi
	fi
	if [ "$bad" = 1 ]; then
		echo
		echo "The version in Cargo.toml increases with every commit (AGENTS.md, Commits)."
		exit 1
	fi
}

case "${1:-}" in
staged) check_staged ;;
*) check_range "${1:-}" "${2:?usage: version-bump.sh staged | <base> <head>}" ;;
esac
