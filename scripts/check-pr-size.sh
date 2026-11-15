#!/usr/bin/env bash
# Fail a pull request over the size limit unless its description says why
# (#2877). Reads "additions<TAB>path" lines on stdin, as the PR files API gives
# them, and the description from PR_BODY. A "## Size" section with text under
# it, HTML comments aside, lets a large pull request through. A convenience,
# not a security boundary; CI runs the base branch's copy of this script.
set -euo pipefail

limit=${PR_SIZE_LIMIT:-800}

# Lock files, fixtures, snapshots and generated files are not hand-written.
# Each is anchored to where it lives, so a hand-written file elsewhere counts.
excluded() {
  case "$1" in
    Cargo.lock | Gemfile.lock | android/*.lockfile | ios/*Package.resolved) return 0 ;;
    ios/*/__Snapshots__/* | android/app/src/test/snapshots/*) return 0 ;;
    crates/*/tests/fixtures/*) return 0 ;;
    ios/generated/* | android/generated/* | design/sketch-kit/*) return 0 ;;
  esac
  return 1
}

# A "## Size" heading, in any case, opens the section and the next level one
# or two heading closes it; a line inside a ``` or ~~~ fence is never a heading.
has_size_reason() {
  printf '%s\n' "${PR_BODY:-}" | tr -d '\r' | perl -0pe 's/<!--.*?-->//gs' | awk '
    /^[[:space:]]*(```|~~~)/ { fence = !fence; next }
    !fence && /^##? / { inside = (tolower($0) ~ /^## size[[:space:]]*$/); next }
    inside && /[^[:space:]]/ { found = 1 }
    END { exit !found }'
}

total=0
# `|| [ -n "$path" ]` keeps a last line that has no newline.
while IFS=$'\t' read -r added path || [ -n "${path:-}" ]; do
  [ -n "$path" ] || continue
  if excluded "$path"; then
    echo "  left out: $path ($added added)"
    continue
  fi
  total=$((total + added))
done

if [ "$total" -le "$limit" ]; then
  echo "✓ $total added lines, within the $limit limit"
elif has_size_reason; then
  echo "✓ $total added lines, over the $limit limit, with a Size section saying why"
else
  echo "✗ $total added lines, over the $limit limit: split the pull request, or add a \"## Size\" section to its description saying why" >&2
  exit 1
fi
