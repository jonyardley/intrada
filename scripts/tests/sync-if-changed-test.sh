#!/usr/bin/env bash
# Self-test for scripts/sync-if-changed.sh (#2732): an unchanged file keeps its
# date, a changed file is rewritten, an added file lands in a new folder, a
# removed file is deleted and its emptied folder goes with it, and a
# destination that would wipe the tree is refused.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/sync-if-changed.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
staging="$tmp/staging"
dest="$tmp/dest"
empty="$tmp/empty-staging"
mkdir -p "$staging/Sources" "$dest/Sources" "$dest/Old" "$empty"

fail=0
check() {
  if eval "$2"; then
    echo "ok: $1"
  else
    echo "FAIL: $1"
    fail=1
  fi
}
sync() {
  if ! bash "$script" "$@"; then
    echo "FAIL: sync $* exited non-zero"
    fail=1
  fi
}
refused() {
  local label="$1" want="$2" code=0 err
  shift 2
  err="$(bash "$script" "$@" 2>&1 >/dev/null)" || code=$?
  check "$label" '[ "$code" = "$want" ] && [ -n "$err" ]'
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

sync "$staging" "$dest"

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

sync "$staging/" "$dest/"
check "trailing slashes on both folders still keep every file" \
  '[ "$(find "$dest" -type f | wc -l | tr -d " ")" = 3 ]'
check "trailing slashes on both folders keep an unchanged date" \
  '[ "$(mtime "$dest/Sources/Same.swift")" = "$same_before" ]'

rm -rf "$dest"
sync "$staging" "$dest"
check "a missing destination is created" \
  '[ -f "$dest/Sources/Same.swift" ] && [ -f "$dest/Sources/Added/Added.swift" ]'

sync "$empty" "$tmp/never"
check "an empty staging folder onto a missing destination does nothing" \
  '[ ! -e "$tmp/never" ]'

refused "a missing staging folder is refused" 1 "$tmp/absent" "$dest"
refused "one argument is refused with usage" 2 "$staging"
refused "three arguments are refused with usage" 2 "$staging" "$dest" extra
# Each unsafe case syncs an empty staging folder, so a broken guard deletes
# only scratch files rather than writing anywhere real.
refused "an empty destination is refused" 1 "$empty" ""
mkdir -p "$tmp/cwd"
printf 'keep\n' > "$tmp/cwd/sentinel"
code=0
(cd "$tmp/cwd" && bash "$script" "$empty" . 2>/dev/null) || code=$?
check "the current folder is refused" '[ "$code" = 1 ] && [ -f "$tmp/cwd/sentinel" ]'
ln -s / "$tmp/root-link"
refused "a destination resolving to the root is refused" 1 "$empty" "$tmp/root-link"
check "a refused sync leaves the destination intact" \
  '[ -f "$dest/Sources/Same.swift" ]'

if [ "$fail" -ne 0 ]; then
  echo "✗ sync-if-changed self-test failed"
  exit 1
fi
echo "✓ sync-if-changed self-test passed"
