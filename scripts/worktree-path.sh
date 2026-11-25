#!/usr/bin/env bash
# Prints the folder of every worktree of this repo that still exists, the main
# checkout first; given a name, prints the one other folder with that name. It asks
# git rather than a fixed parent: new worktrees live in .claude/worktrees/ while
# older ones finish in the sibling intrada-worktrees folder (#3028).
#
# Usage: bash scripts/worktree-path.sh [name]

set -euo pipefail

name="${1:-}"
paths=()
first=1
while IFS= read -r p; do
  is_main=$first
  first=0
  [ -d "$p" ] || continue
  if [ -n "$name" ]; then
    # Never the main checkout by name: worktree-rm would clean its simulator.
    [ "$is_main" -eq 0 ] && [ "$(basename "$p")" = "$name" ] || continue
  fi
  paths+=("$p")
done < <(git worktree list --porcelain | sed -n 's/^worktree //p')

if [ -n "$name" ] && [ "${#paths[@]}" -eq 0 ]; then
  echo "✗ no worktree named $name" >&2
  exit 1
fi
if [ -n "$name" ] && [ "${#paths[@]}" -gt 1 ]; then
  echo "✗ more than one worktree named $name; remove one with git worktree remove:" >&2
  printf '  %s\n' "${paths[@]}" >&2
  exit 1
fi
[ "${#paths[@]}" -eq 0 ] || printf '%s\n' "${paths[@]}"
