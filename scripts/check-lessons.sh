#!/usr/bin/env bash
# The lessons file holds at most 30 lessons (#2801). Every session start prints
# it, so an unpruned file costs every session; the cap makes the archiving
# happen rather than be remembered. Both lessons files merge by union (#2937),
# so it also refuses a lesson kept twice or brought back from the archive, and
# a rules line above the list kept twice (#2943).
#
# The files are overridable so the self-test can point them at fixtures
# (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

file="${LESSONS_FILE:-docs/lessons.md}"
archive="${LESSONS_ARCHIVE:-docs/lessons-archive.md}"
cap=30

for f in "$file" "$archive"; do
  if [ ! -f "$f" ]; then
    echo "✗ check-lessons: no $f to read" >&2
    exit 2
  fi
done

# Count list items from "## Lessons" to the end, the span the session start
# hook prints; no heading means it prints nothing (#2801).
count=$(awk '/^## Lessons$/ { seen = 1; next } seen && /^- / { n++ } END { print (seen ? n + 0 : -1) }' "$file")

if [ "$count" -lt 0 ]; then
  echo "✗ $file has no \"## Lessons\" heading, so no session start prints it (#2801)" >&2
  exit 1
fi

status=0
if [ "$count" -gt "$cap" ]; then
  echo "✗ $file holds $count lessons, over the cap of $cap: archive to docs/lessons-archive.md each \"(issue #N)\" lesson whose issue has closed, with \"issue:\" in front, then the oldest with no \"seen again\" and no open issue (#2801, #2936)" >&2
  status=1
fi

# Two branches bumping the same "(seen again N)" each keep their line under a
# union merge, and a lesson archived on one branch comes back from the other
# (#2937). Compare with the count and the final full stop set aside, and an
# archive line by its end, since a rule's file may stand in front of it. A
# deliberate copy back raises the count; a merge brings the line back as it was.
awk -v lessons="$file" -v archive="$archive" '
  function norm(s) {
    sub(/^- /, "", s)
    sub(/[ \t]+$/, "", s)
    sub(/\.$/, "", s)
    sub(/ \(seen again [0-9]+\)$/, "", s)
    sub(/\.$/, "", s)
    return s
  }
  function seen(s) {
    if (!match(s, /\(seen again [0-9]+\)\.?[ \t]*$/)) return 0
    s = substr(s, RSTART + 12)
    return s + 0
  }
  FNR == 1 { in_list = 0 }
  FILENAME == archive && /^- / { arch[++na] = norm($0); arch_seen[na] = seen($0); arch_line[na] = FNR; next }
  FILENAME == lessons && /^## Lessons$/ { in_list = 1; next }
  # Two branches editing one rules line both keep it (#2943). Lines with no
  # words (blank, a bare quote marker, a rule or table separator) and code
  # fences repeat by design.
  FILENAME == lessons && !in_list {
    r = $0
    sub(/^[ \t]+/, "", r)
    sub(/[ \t]+$/, "", r)
    if (r !~ /[[:alnum:]]/ || r ~ /^(```|~~~)/) next
    if (r in rule) {
      printf "✗ %s line %d repeats line %d in the rules above \"## Lessons\": \"%s\"; a merge kept both edits, so read the rules around both lines and keep one version, or, if no merge did this, reword one (#2943)\n", lessons, FNR, rule[r], r > "/dev/stderr"
      bad = 1
    } else {
      rule[r] = FNR
    }
    next
  }
  FILENAME == lessons && in_list && /^- / {
    t = norm($0)
    if (t in first) {
      printf "✗ %s line %d repeats line %d once \"(seen again N)\" is set aside: keep one line, with the higher count (#2937)\n", lessons, FNR, first[t] > "/dev/stderr"
      bad = 1
    } else {
      first[t] = FNR
    }
    for (i = 1; i <= na; i++) {
      a = arch[i]
      if (seen($0) <= arch_seen[i] && (a == t || (length(a) > length(t) && substr(a, length(a) - length(t)) == " " t))) {
        printf "✗ %s line %d is archived at %s line %d with no higher \"(seen again N)\": a merge brought it back, so delete it here, or raise its count if it was seen again (#2937)\n", lessons, FNR, archive, arch_line[i] > "/dev/stderr"
        bad = 1
        break
      }
    }
  }
  END { exit bad }
' "$archive" "$file" || status=1

exit "$status"
