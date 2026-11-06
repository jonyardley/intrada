#!/usr/bin/env bash
# Usage: sync-if-changed.sh <staging> <dest>
# Makes dest match staging, writing only the files whose bytes differ, so an
# unchanged generated file keeps its date and Xcode does not recompile the
# Swift that imports it (#2732). Files absent from staging are deleted and
# emptied folders removed. Plain cp on purpose: rsync -a would copy dates.
# Regular files only; symlinks are not handled (the generated tree has none).
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <staging> <dest>" >&2
  exit 2
fi
staging="${1%/}"
dest="${2%/}"
if [ ! -d "$staging" ]; then
  echo "✗ staging folder $staging does not exist" >&2
  exit 1
fi
if [ -z "$dest" ] || [ "$dest" = "." ] || { [ -d "$dest" ] && [ "$(cd "$dest" && pwd -P)" = / ]; }; then
  echo "✗ refusing to sync onto '$2': it would delete everything not in staging" >&2
  exit 1
fi

while IFS= read -r -d '' f; do
  rel="${f#"$staging"/}"
  if ! cmp -s "$f" "$dest/$rel"; then
    mkdir -p "$(dirname "$dest/$rel")"
    cp "$f" "$dest/$rel"
  fi
done < <(find "$staging" -type f -print0)

[ -d "$dest" ] || exit 0
while IFS= read -r -d '' f; do
  rel="${f#"$dest"/}"
  [ -f "$staging/$rel" ] || rm -f "$f"
done < <(find "$dest" -type f -print0)

find "$dest" -mindepth 1 -type d -empty -delete
