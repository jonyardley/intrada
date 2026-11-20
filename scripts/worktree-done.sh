#!/usr/bin/env bash
# Remove the worktree this runs in, once its work is safe on GitHub (#2918).
# Run by the session that owns the worktree, from inside it, after its PR
# merges. Refuses unless all three hold:
#   merged:   a merged PR for this branch exists on GitHub.
#   pushed:   that PR's last commit is exactly this HEAD, so no local commit
#             is missing from it. Read from the PR because the merge deletes
#             the remote branch, so there is no upstream left to compare with.
#   clean:    git status shows nothing, untracked files included.
# The worktree guard (~/.claude/hooks/guard-worktree.sh) runs the same checks
# and also refuses another live session's worktree. The branch is kept, as
# `just worktree-rm` keeps it.

set -euo pipefail

here="$(git rev-parse --show-toplevel)"
main_root="$(git rev-parse --path-format=absolute --git-common-dir | xargs dirname)"

refuse() {
  echo "✗ not removing $here: $1" >&2
  exit 1
}

[ "$here" != "$main_root" ] || refuse "this is the main checkout; run it inside the worktree to remove"
command -v gh >/dev/null 2>&1 || refuse "gh is needed to see the merged PR"

status="$(git status --porcelain)" || refuse "git status failed"
[ -z "$status" ] || refuse "it has uncommitted or untracked files"

branch="$(git symbolic-ref --quiet --short HEAD)" || refuse "it is not on a branch"
head="$(git rev-parse HEAD)"

heads="$(gh pr list --head "$branch" --state merged --json number,headRefOid \
  --jq '.[] | "\(.number) \(.headRefOid)"')" || refuse "GitHub could not be read"
[ -n "$heads" ] || refuse "no merged PR for $branch"
pr="$(printf '%s\n' "$heads" | awk -v h="$head" '$2 == h {print $1; exit}')"
[ -n "$pr" ] || refuse "this HEAD is not the last commit of the merged PR for $branch; push anything new to a new PR first"

(just ios-test-sim-clean) || true

cd "$main_root"
git worktree remove "$here"
echo "✓ removed $here (PR #$pr merged, clean, nothing unpushed)"
echo "  This session's folder has gone: end it with /exit."
