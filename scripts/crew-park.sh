#!/usr/bin/env bash
# The crew's park step (#2819, specs/crew-loop.md): is a plan or a diff on a
# sensitive surface? The only list is the paths: frontmatter of
# .claude/rules/sensitive-surfaces.md, read at every run so the two never drift.
#
#   crew-park.sh <file>   exit 3 and print each match; 0 when clean; 2 when the
#                         file or the list cannot be read, which the loop parks on
#
# A diff is read by its changed-file headers only: a test or document that
# quotes a sensitive path is not a change to it.

set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
rules="${CREW_PARK_RULES:-$root/.claude/rules/sensitive-surfaces.md}"

if [ $# -ne 1 ]; then
  echo "usage: $0 <plan-or-diff-file>" >&2
  exit 2
fi
file=$1
if [ ! -r "$file" ]; then
  echo "crew-park: cannot read $file" >&2
  exit 2
fi

globs=$(awk '
  NR == 1 && $0 != "---" { exit }
  NR > 1 && $0 == "---" { exit }
  /^paths:/ { inpaths = 1; next }
  inpaths && /^[[:space:]]+-[[:space:]]/ { sub(/^[[:space:]]+-[[:space:]]+/, ""); gsub(/"/, ""); print; next }
  inpaths { inpaths = 0 }
' "$rules" 2>/dev/null)
if [ -z "$globs" ]; then
  echo "crew-park: no paths: list in $rules" >&2
  exit 2
fi

if grep -q '^diff --git ' "$file"; then
  text=$(grep -E '^(diff --git|\+\+\+|---) ' "$file")
else
  text=$(cat "$file")
fi

tokens=$(printf '%s\n' "$text" | grep -oE '[A-Za-z0-9_./*@-]+' | sed -E 's#^\./##; s#^-+##; s#^[ab]/##; s#\.+$##' | sort -u)

# A plan abbreviates paths, so a token also counts when it is the tail of a
# glob's literal prefix two segments or longer (domain/session/active.rs), a
# crate's own folder (intrada-ffi), a literal glob's file name, or a wider glob
# of its own (crates/**): each errs towards parking, never past it.
names() {
  local t=$1 glob=$2 prefix tail
  # shellcheck disable=SC2053 # the glob is meant to match as a pattern
  [[ $t == $glob ]] && return 0
  prefix=${glob%%[*?[]*}
  prefix=${prefix%/}
  if [[ $t == *[*?]* ]]; then
    tail=${t%%[*?]*}
    [[ $tail == */* && "$prefix/" == "$tail"* ]]
    return
  fi
  tail=$prefix
  while [[ $tail == */* ]]; do
    if [ "$t" = "$tail" ] || [[ $t == "$tail"/* ]]; then return 0; fi
    tail=${tail#*/}
  done
  if [[ $prefix == crates/* && $prefix != crates/*/* ]] && { [ "$t" = "$tail" ] || [[ $t == "$tail"/* ]]; }; then
    return 0
  fi
  [[ $glob != *[*?[]* ]] && [ "$t" = "$tail" ]
}

matched=0
while IFS= read -r glob; do
  while IFS= read -r t; do
    if names "$t" "$glob"; then
      echo "park: $t matches $glob in ${rules#"$root"/}"
      matched=1
    fi
  done <<<"$tokens"
done <<<"$globs"

[ "$matched" -eq 0 ] || exit 3
