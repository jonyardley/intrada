#!/usr/bin/env bash
# What this branch changes, so local gates can skip work a change cannot
# affect (#2734). CI never sources this: it always runs everything.

# Every path that differs between the merge base with origin/main and the
# working tree, plus untracked files, one per line. Fails when the scope
# cannot be known (no origin/main, a detached head, or on main itself), and
# a caller treats that as "run everything". DIFF_SCOPE_BASE names another
# base ref, as LINK_CHECK_BASE does for check-links.sh.
diff_scope_paths() {
  local branch base
  branch=$(git symbolic-ref --quiet --short HEAD 2>/dev/null) || return 1
  [ "$branch" != "main" ] || return 1
  base=$(git merge-base "${DIFF_SCOPE_BASE:-origin/main}" HEAD 2>/dev/null) || return 1
  git diff --name-only --no-renames "$base" -- || return 1
  git ls-files --others --exclude-standard || return 1
}

# Hygiene check entries ("name:command") to run, one per line. Every entry
# whose command runs a scripts/tests/ self-test is dropped when the change
# touches nothing under scripts/, the justfile or .github/, so a new
# self-test is covered with no edit here; the dropped names go to stderr on
# one "skipped:" line. An unknown scope keeps every entry.
diff_scope_select_checks() {
  local paths entry skipped=""
  if ! paths=$(diff_scope_paths) || grep -qE '^(scripts/|justfile$|\.github/)' <<<"$paths"; then
    printf '%s\n' "$@"
    return 0
  fi
  for entry in "$@"; do
    case "${entry#*:}" in
      *scripts/tests/*) skipped="${skipped:+$skipped, }${entry%%:*}" ;;
      *) printf '%s\n' "$entry" ;;
    esac
  done
  if [ -n "$skipped" ]; then
    echo "skipped: no change under scripts/, the justfile or .github/, so not running $skipped" >&2
  fi
}
