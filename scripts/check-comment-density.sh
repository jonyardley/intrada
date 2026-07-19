#!/usr/bin/env bash
# Flag pushes that add too many comments relative to code.
# See CLAUDE.md "Comments" — code should be self-explanatory; comments are
# for non-obvious WHY, not narration. Bypass with `SKIP_COMMENT_CHECK=1`.

set -euo pipefail

if [ "${SKIP_COMMENT_CHECK:-}" = "1" ]; then
  exit 0
fi

# CI passes the PR base ref via COMMENT_CHECK_BASE — HEAD is detached there, so
# the branch-name guard below would otherwise skip the check entirely. Locally
# the pre-push hook leaves it unset and we derive the base from origin/main.
base="${COMMENT_CHECK_BASE:-}"
if [ -z "$base" ]; then
  branch=$(git symbolic-ref --short HEAD 2>/dev/null || true)
  if [ -z "$branch" ]; then
    exit 0
  fi
  case "$branch" in
    main|master) exit 0 ;;
  esac
  base="origin/main"
fi

if ! git rev-parse --verify "$base" >/dev/null 2>&1; then
  exit 0
fi

range="$base...HEAD"

# One diff call so git's rename detection stays intact: a get-names-then-
# re-diff-those-paths two-step loses the pairing with the old path, so a
# 100%-identical file relocated by `git mv` (or an agent recreating the same
# content elsewhere) reads as pure new content instead of a zero-line rename.
diff_out=$(git diff "$range" -- \
  '*.rs' '*.swift' '*.css' '*.ts' '*.tsx' '*.js' '*.jsx' 2>/dev/null || true)
if [ -z "$diff_out" ]; then
  exit 0
fi

added=$(printf '%s\n' "$diff_out" | grep -E '^\+' | grep -vE '^\+\+\+' || true)
if [ -z "$added" ]; then
  exit 0
fi

# Heuristic: a line is "comment" if its first non-whitespace is //, ///, /*,
# * (block continuation), or # (shell/python).
added_comments=$(printf '%s\n' "$added" \
  | grep -cE '^\+[[:space:]]*(//|/\*|\*[^/]|\*$|#[[:space:]])' || true)
added_blank=$(printf '%s\n' "$added" | grep -cE '^\+[[:space:]]*$' || true)
added_total=$(printf '%s\n' "$added" | grep -cE '^\+' || true)
added_code=$((added_total - added_comments - added_blank))

if [ "$added_code" -le 0 ]; then
  exit 0
fi

threshold="0.15"
ratio=$(awk -v c="$added_comments" -v k="$added_code" \
  'BEGIN { printf "%.2f", c / k }')
over=$(awk -v r="$ratio" -v t="$threshold" 'BEGIN { print (r > t) }')

if [ "$over" = "1" ]; then
  cat <<EOF >&2

❌ Pre-push blocked: comment density on this branch is ${ratio} (threshold ${threshold}).

   Added comment lines: ${added_comments}
   Added code lines:    ${added_code}

   CLAUDE.md asks for self-explanatory code with minimal comments. Add a
   comment only for non-obvious WHY (bug ref, framework quirk, hidden
   invariant) — never to restate WHAT.

   Inspect what tripped the check:

     git diff origin/main...HEAD -- '*.rs' '*.swift' '*.ts' '*.tsx' \\
       | grep -E '^\+[[:space:]]*(//|/\*|\*)'

   If the comments are genuinely justified (incident write-up, vendored
   notice, etc.), bypass with:

     SKIP_COMMENT_CHECK=1 git push

EOF
  exit 1
fi

exit 0
