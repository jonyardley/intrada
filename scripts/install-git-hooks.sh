#!/usr/bin/env bash
# Opt in to the repo-tracked git hooks under .githooks/.
#
# What it does: sets `core.hooksPath` to `.githooks` for this clone,
# which makes git look there for pre-commit / pre-push / etc. instead
# of the default `.git/hooks/`.
#
# Run once after cloning:
#   bash scripts/install-git-hooks.sh
#
# To opt out:
#   git config --unset core.hooksPath
#
# This is a per-clone, opt-in setting — it does not affect other
# contributors automatically. Required dependencies for the hooks to
# function: `gh` (GitHub CLI) and `jq`. If either is missing, the
# affected hook will pass silently rather than block.

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

if [ ! -d .githooks ]; then
  echo "❌ .githooks/ not found at repo root." >&2
  exit 1
fi

# Make every hook executable (in case it was checked out without +x).
chmod +x .githooks/* 2>/dev/null || true

# Set in the main repo config (shared across all worktrees by default).
git config core.hooksPath .githooks

# Worktrees can carry their own `core.hooksPath` override (some tooling
# sets this when a worktree is created so hooks default to the main
# repo's .git/hooks/). That override shadows our setting. Clear it
# from any worktree-local config so the repo-shared value takes effect.
worktree_configs=$(find .git/worktrees -name config.worktree 2>/dev/null || true)
for cfg in $worktree_configs; do
  if grep -q "^[[:space:]]*hookspath" "$cfg" 2>/dev/null; then
    git --git-dir="$(dirname "$cfg")" config --worktree --unset core.hooksPath 2>/dev/null || true
  fi
done

# If the current invocation IS in a worktree, the loop above may not
# reach our own config (paths are tricky). Belt-and-braces.
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  git config --worktree --unset core.hooksPath 2>/dev/null || true
fi

echo "✅ Git hooks installed (core.hooksPath = .githooks)."
echo
echo "Hooks active:"
for h in .githooks/*; do
  [ -f "$h" ] && echo "  - $(basename "$h")"
done
echo
echo "To opt out:  git config --unset core.hooksPath"

# Light dependency hint, doesn't block install.
missing=""
command -v gh >/dev/null 2>&1 || missing="${missing} gh"
command -v jq >/dev/null 2>&1 || missing="${missing} jq"
if [ -n "$missing" ]; then
  echo
  echo "⚠️  Missing dependencies for full hook coverage:${missing}"
  echo "    Install via Homebrew: brew install${missing}"
fi
