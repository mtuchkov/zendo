#!/bin/sh
# Check the message of every commit in a range, for CI: .githooks/lint-range.sh <base> <head>
# Merge commits are skipped. Exit 1 when a message does not follow .skills/git-commit/SKILL.md.
set -eu
base="$1"
head="$2"
root=$(git rev-parse --show-toplevel)
if [ -z "$base" ] || [ "$base" = "0000000000000000000000000000000000000000" ] || ! git cat-file -e "$base^{commit}" 2>/dev/null; then
	# A new branch: the commits that main does not have.
	range="$head --not origin/main"
else
	range="$base..$head"
fi
msg=$(mktemp)
trap 'rm -f "$msg"' EXIT
bad=0
for c in $(git rev-list --no-merges $range); do
	git log -1 --format=%B "$c" >"$msg"
	if ! out=$("$root/.githooks/commit-msg" "$msg" 2>&1); then
		echo "✗ $(git log -1 --format='%h %s' "$c")"
		echo "$out" | sed 's/^/    /'
		bad=1
	else
		echo "✓ $(git log -1 --format='%h %s' "$c")"
	fi
done
if [ "$bad" = 1 ]; then
	echo
	echo "Commit messages follow .skills/git-commit/SKILL.md. Fix them with: git rebase -i, then reword."
	exit 1
fi
