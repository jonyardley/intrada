#!/usr/bin/env bash
# How many fixed lessons came back (#2968). A retro fix archives its lesson
# with "fix:" in front; when the mistake happens again the lesson is copied
# back with its "(seen again N)" raised, so a fix that did not hold shows as a
# later copy, in the lessons file or further down the archive, with a higher
# count than the fixed line. `just usage` prints this after its report.
#
# The files are overridable so the self-test can point them at fixtures
# (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

file="${LESSONS_FILE:-docs/lessons.md}"
archive="${LESSONS_ARCHIVE:-docs/lessons-archive.md}"

for f in "$file" "$archive"; do
  if [ ! -f "$f" ]; then
    echo "✗ lesson-fixes: no $f to read" >&2
    exit 2
  fi
done

# A lesson is compared from its date on, so the "fix:" or "issue:" in front of
# an archive line, its count, its "(issue #N)" and full stop are set aside.
awk -v lessons="$file" -v archive="$archive" '
  function norm(s) {
    if (match(s, /[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9], /)) s = substr(s, RSTART)
    sub(/[ \t]+$/, "", s)
    sub(/\.$/, "", s)
    sub(/ \(seen again [0-9]+\)$/, "", s)
    sub(/ \(issue #[0-9]+\)$/, "", s)
    sub(/\.$/, "", s)
    return s
  }
  function seen(s) {
    if (!match(s, /\(seen again [0-9]+\)\.?[ \t]*$/)) return 0
    return substr(s, RSTART + 12) + 0
  }
  FNR == 1 { in_list = 0 }
  FILENAME == archive && /^- / {
    n++
    text[n] = norm($0); count[n] = seen($0); fixed[n] = ($0 ~ /^- fix: /)
    next
  }
  FILENAME == lessons && /^## Lessons$/ { in_list = 1; next }
  FILENAME == lessons && in_list && /^- / { n++; text[n] = norm($0); count[n] = seen($0); fixed[n] = 0 }
  END {
    for (i = 1; i <= n; i++) {
      if (!fixed[i]) continue
      fixes++
      for (j = i + 1; j <= n; j++) {
        if (text[j] == text[i] && count[j] > count[i]) { back++; list = list "  " text[i] "\n"; break }
      }
    }
    printf "Fixes that did not hold: %d of %d fixes came back (seen again after the fix).\n", back, fixes
    printf "%s", list
  }
' "$archive" "$file"
