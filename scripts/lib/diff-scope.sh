#!/usr/bin/env bash
# What this branch changes, so local gates can skip work a change cannot
# affect (#2734). CI does not source this: it runs its own list directly.

# Paths changed since the merge base with origin/main, worktree and untracked
# included; fails on no origin/main, a detached head or main ("run everything").
diff_scope_paths() {
  local branch base tracked untracked
  branch=$(git symbolic-ref --quiet --short HEAD 2>/dev/null) || return 1
  [ "$branch" != "main" ] || return 1
  base=$(git merge-base origin/main HEAD 2>/dev/null) || return 1
  tracked=$(git diff -z --name-only --no-renames "$base" -- | tr '\0' '\n'; exit "${PIPESTATUS[0]}") || return 1
  untracked=$(git ls-files -z --others --exclude-standard | tr '\0' '\n'; exit "${PIPESTATUS[0]}") || return 1
  printf '%s\n%s\n' "$tracked" "$untracked"
}

# Hygiene "name:command" entries to run: scripts/tests/ self-tests drop out, named
# on one stderr "skipped:" line, unless scripts/, the justfile or .github/ changed.
diff_scope_select_checks() {
  local paths entry kept="" skipped=""
  if ! paths=$(diff_scope_paths) || grep -qE '^(scripts/|justfile$|\.github/)' <<<"$paths"; then
    printf '%s\n' "$@"
    return 0
  fi
  for entry in "$@"; do
    case "${entry#*:}" in
      *scripts/tests/*) skipped="${skipped:+$skipped, }${entry%%:*}" ;;
      *) kept="$kept$entry"$'\n' ;;
    esac
  done
  if [ -z "$kept" ]; then
    printf '%s\n' "$@"
    return 0
  fi
  printf '%s' "$kept"
  if [ -n "$skipped" ]; then
    echo "skipped: no change under scripts/, the justfile or .github/, so not running $skipped" >&2
  fi
}

# CI's Android settings, read from ci.yml so the local gates cannot drift from
# them (#2877). Succeeds when the `changes` job says Android is paused.
ci_android_paused() {
  grep -qE '^[[:space:]]*android_paused:[[:space:]]*"true"' .github/workflows/ci.yml
}

# Succeeds when a path on stdin matches CI's `android` path filter, or when
# that list cannot be read ("run everything").
ci_android_changed() {
  local patterns pattern changed
  patterns=$(awk '
    /^[[:space:]]*filters:[[:space:]]*\|/ { infilters = 1; next }
    infilters && /^[[:space:]]*android:[[:space:]]*$/ { inandroid = 1; next }
    inandroid && /^[[:space:]]*- / { sub(/^[[:space:]]*- /, ""); gsub(/\x27|"/, ""); print; next }
    inandroid { exit }' .github/workflows/ci.yml)
  [ -n "$patterns" ] || return 0
  while IFS= read -r changed; do
    [ -n "$changed" ] || continue
    while IFS= read -r pattern; do
      # shellcheck disable=SC2053 # the pattern is a glob on purpose
      [[ $changed == ${pattern//\*\*/*} ]] && return 0
    done <<<"$patterns"
  done
  return 1
}
