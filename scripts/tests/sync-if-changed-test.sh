#!/usr/bin/env bash
# Self-test for scripts/sync-if-changed.sh (#2732): an unchanged file keeps its
# date, a changed file is rewritten, an added file lands in a new folder, a
# removed file is deleted and its emptied folder goes with it.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/sync-if-changed.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
staging="$tmp/staging"
dest="$tmp/dest"
mkdir -p "$staging/Sources" "$dest/Sources" "$dest/Old"

fail=0
check() {
  if eval "$2"; then
    echo "ok: $1"
  else
    echo "FAIL: $1"
    fail=1
  fi
}
mtime() { stat -c %Y "$1" 2>/dev/null || stat -f %m "$1"; }

printf 'same\n' > "$staging/Sources/Same.swift"
printf 'same\n' > "$dest/Sources/Same.swift"
printf 'new body\n' > "$staging/Sources/Changed.swift"
printf 'old body\n' > "$dest/Sources/Changed.swift"
mkdir -p "$staging/Sources/Added"
printf 'added\n' > "$staging/Sources/Added/Added.swift"
printf 'gone\n' > "$dest/Sources/Removed.swift"
printf 'gone\n' > "$dest/Old/Orphan.swift"
# Backdated so a rewrite within the same second still shows as a new date.
touch -t 202001010000 "$dest/Sources/Same.swift" "$dest/Sources/Changed.swift"
same_before="$(mtime "$dest/Sources/Same.swift")"
changed_before="$(mtime "$dest/Sources/Changed.swift")"

bash "$script" "$staging" "$dest"

check "an unchanged file keeps its date" \
  '[ "$(mtime "$dest/Sources/Same.swift")" = "$same_before" ]'
check "a changed file takes the new content" \
  '[ "$(cat "$dest/Sources/Changed.swift")" = "new body" ]'
check "a changed file takes a new date" \
  '[ "$(mtime "$dest/Sources/Changed.swift")" != "$changed_before" ]'
check "an added file lands in a new folder" \
  '[ "$(cat "$dest/Sources/Added/Added.swift" 2>/dev/null)" = "added" ]'
check "a file missing from staging is deleted" \
  '[ ! -e "$dest/Sources/Removed.swift" ]'
check "an emptied folder is removed" \
  '[ ! -e "$dest/Old" ]'
check "the staging folder is left alone" \
  '[ "$(find "$staging" -type f | wc -l | tr -d " ")" = 3 ]'

rm -rf "$dest"
bash "$script" "$staging" "$dest"
check "a missing destination is created" \
  '[ -f "$dest/Sources/Same.swift" ] && [ -f "$dest/Sources/Added/Added.swift" ]'

if bash "$script" "$tmp/absent" "$dest" 2>/dev/null; then
  echo "FAIL: a missing staging folder is refused"
  fail=1
else
  echo "ok: a missing staging folder is refused"
fi
check "a refused sync leaves the destination intact" \
  '[ -f "$dest/Sources/Same.swift" ]'

if [ "$fail" -ne 0 ]; then
  echo "✗ sync-if-changed self-test failed"
  exit 1
fi
echo "✓ sync-if-changed self-test passed"
