#!/bin/sh
# After a push to main: check the version of each open pull request against the new main, and set
# the commit status "version against main" on its head. The check of a pull request runs against
# main as it was then; when main gets the same version later, git merges the equal lines without a
# conflict. This status shows the problem before anyone merges the pull request.
#   GH_TOKEN=… GITHUB_REPOSITORY=owner/repo .githooks/pull-versions.sh <main sha>
set -eu
repo="${GITHUB_REPOSITORY:?}"
main="${1:?usage: pull-versions.sh <main sha>}"
here=$(dirname "$0")

gh api "repos/$repo/pulls?state=open&base=main&per_page=100" --jq '.[] | "\(.number) \(.head.sha)"' |
	while read -r n sha; do
		git fetch -q origin "pull/$n/head"
		if why=$(sh "$here/version-bump.sh" above "$sha" "$main"); then
			state=success
			why="above the version of main $(git rev-parse --short "$main")"
		else
			state=failure
		fi
		echo "#$n $state: $why"
		gh api -X POST "repos/$repo/statuses/$sha" -f state="$state" -f context="version against main" \
			-f description="$(printf '%s' "$why" | cut -c1-140)" >/dev/null
	done
