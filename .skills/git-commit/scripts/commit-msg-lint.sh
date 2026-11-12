#!/usr/bin/env bash
set -euo pipefail

msg_file="${1:-}"
if [ -z "$msg_file" ] || [ ! -f "$msg_file" ]; then
  echo "commit-msg-lint: expected commit message file path" >&2
  exit 1
fi

header="$(sed '/^[[:space:]]*#/d;/^[[:space:]]*$/d;q' "$msg_file" || true)"
if [ -z "$header" ]; then
  echo "commit-msg-lint: empty commit message header" >&2
  exit 1
fi

if [ "${#header}" -gt 72 ]; then
  echo "commit-msg-lint: header must be <= 72 chars" >&2
  exit 1
fi

if [[ "$header" =~ \.$ ]]; then
  echo "commit-msg-lint: header must not end with a period" >&2
  exit 1
fi

header_re='^(feat|fix|refactor|perf|test|docs|build|ci|chore|revert)(\([a-z0-9][a-z0-9._/-]*\))?(!)?: .+$'
if ! [[ "$header" =~ $header_re ]]; then
  echo "commit-msg-lint: header must match Conventional Commits:" >&2
  echo "  type(scope)!: short imperative summary" >&2
  exit 1
fi

has_motivation=0
has_modifications=0
has_result=0

if grep -q '^Motivation:$' "$msg_file"; then
  has_motivation=1
fi
if grep -q '^Modifications:$' "$msg_file"; then
  has_modifications=1
fi
if grep -q '^Result:$' "$msg_file"; then
  has_result=1
fi

has_any_section=$((has_motivation + has_modifications + has_result))
if [ "$has_any_section" -gt 0 ] && [ "$has_any_section" -lt 3 ]; then
  echo "commit-msg-lint: Netty-style body must include all sections:" >&2
  echo "  Motivation:, Modifications:, Result:" >&2
  exit 1
fi

exit 0
