#!/usr/bin/env bash
# The lessons file holds at most 30 lessons (#2801). Every session start prints
# it, so an unpruned file costs every session; the cap makes the monthly prune
# happen rather than be remembered.
#
# The file is overridable so the self-test can point it at fixtures
# (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

file="${LESSONS_FILE:-docs/lessons.md}"
cap=30

if [ ! -f "$file" ]; then
  echo "✗ check-lessons: no $file to read" >&2
  exit 2
fi

# Count list items from "## Lessons" to the end, the span the session start
# hook prints; no heading means it prints nothing (#2801).
count=$(awk '/^## Lessons$/ { seen = 1; next } seen && /^- / { n++ } END { print (seen ? n + 0 : -1) }' "$file")

if [ "$count" -lt 0 ]; then
  echo "✗ $file has no \"## Lessons\" heading, so no session start prints it (#2801)" >&2
  exit 1
fi

if [ "$count" -gt "$cap" ]; then
  echo "✗ $file holds $count lessons, over the cap of $cap: move the oldest with no \"seen again\" to docs/lessons-archive.md (#2801)" >&2
  exit 1
fi
