#!/usr/bin/env bash
# Flag em dashes and en dashes added to prose or comments.
# See CLAUDE.md Conventions: house style is plain British English with neither.
# This gates the rule on changed lines only, the same diff-scoped way
# check-comment-density.sh works, so it binds new content without rewriting a
# tree that already carries hundreds of historical dashes. Bypass a genuinely
# justified case (a vendored notice, quoted material) with SKIP_DASH_CHECK=1.

set -euo pipefail

if [ "${SKIP_DASH_CHECK:-}" = "1" ]; then
  exit 0
fi

# CI passes the PR base via DASH_CHECK_BASE (HEAD is detached on a PR checkout).
# Locally the pre-push hook leaves it unset and we derive origin/main.
base="${DASH_CHECK_BASE:-}"
if [ -z "$base" ]; then
  branch=$(git symbolic-ref --short HEAD 2>/dev/null || true)
  if [ -z "$branch" ]; then
    exit 0
  fi
  case "$branch" in
    main|master) exit 0 ;;
  esac
  base="origin/main"
fi

if ! git rev-parse --verify "$base" >/dev/null 2>&1; then
  exit 0
fi

range="$base...HEAD"

# em dash U+2014, en dash U+2013, written as raw UTF-8 bytes via ANSI-C quoting
# (bash 3.2 has no \u) so the literal glyphs never appear in this file.
em=$'\xe2\x80\x94'
en=$'\xe2\x80\x93'
dash_re="$em|$en"

# Prose and comment surfaces only. The label-separator exception (#1231) lives
# here as an exemption: the three structured docs that use it as house style are
# excluded, along with the archived specs kept only for reference.
files=$(git diff "$range" --name-only -- \
  '*.md' '*.rs' '*.swift' '*.sh' '*.py' '*.yml' '*.yaml' \
  ':(exclude)CLAUDE.md' \
  ':(exclude)docs/roadmap.md' \
  ':(exclude)design/CLAUDE.md' \
  ':(exclude)specs/_archive/**' \
  2>/dev/null || true)

if [ -z "$files" ]; then
  exit 0
fi

found=0
report=""
while IFS= read -r f; do
  [ -n "$f" ] || continue
  hits=$(git diff "$range" -- "$f" \
    | grep -E '^\+' | grep -vE '^\+\+\+' \
    | grep -E "$dash_re" || true)
  if [ -n "$hits" ]; then
    found=1
    report="$report"$'\n'"  $f:"$'\n'"$(printf '%s\n' "$hits" | sed 's/^/    /')"
  fi
done <<EOF
$files
EOF

if [ "$found" = "1" ]; then
  cat <<EOF >&2

Blocked: this branch adds em or en dashes to prose or comments.
$report

House style is plain British English with neither. Replace them with a comma,
a colon, a full stop, or a rephrase.

If a case is genuinely justified, bypass with:

  SKIP_DASH_CHECK=1 git push

EOF
  exit 1
fi

exit 0
