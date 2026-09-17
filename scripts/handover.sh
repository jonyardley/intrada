#!/usr/bin/env bash
# Print the opener for the next session, so clearing costs a paste rather
# than a retelling (#1986). 30 of 93 main sessions in the fortnight to
# 2026-09-17 passed 200k, and 123 turns resumed a parked context at write
# price for $199: the rule to finish a unit and clear has been in CLAUDE.md
# throughout, and friction is why it has not held.
#
# Everything printed is read from git, GitHub and .claude/settings.json.
# Nothing is invented: a fact that cannot be read is left out rather than
# guessed at, since an opener that lies costs more than one that is short.
#
# Called via `just handover [N]`.

set -euo pipefail

lib_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/claim-branch.sh
source "$lib_dir/lib/claim-branch.sh"

root="$(git rev-parse --show-toplevel)"
branch="$(git rev-parse --abbrev-ref HEAD)"
repo="$(gh repo view --json nameWithOwner -q .nameWithOwner)"

number="${1:-}"
if [ -z "$number" ]; then
  number="$(gh pr list --repo "$repo" --head "$branch" --state open --json title -q '.[0].title // empty' |
    grep -oE '#[0-9]+' | head -1 | tr -d '#' || true)"
fi
if [ -z "$number" ]; then
  echo "✗ handover needs an issue number: no open PR on '$branch' has one in its title." >&2
  echo "  Try: just handover 1986" >&2
  exit 1
fi

issue="$(gh issue view "$number" --repo "$repo" --json title,state,url,labels 2>/dev/null || true)"
if [ -z "$issue" ]; then
  echo "✗ cannot read issue #$number from $repo." >&2
  exit 1
fi
title="$(printf '%s' "$issue" | jq -r '.title')"
state="$(printf '%s' "$issue" | jq -r '.state')"
labels="$(printf '%s' "$issue" | jq -r '[.labels[].name] | join(", ")')"

claim_body="$(gh issue view "$number" --repo "$repo" --json comments -q \
  '[.comments[] | select(.body | test("^Claimed"; "i"))] | last | .body // empty')"
claimed_branch="$(claim_branch_from_body "$claim_body")"

settings="$root/.claude/settings.json"
model="$(jq -r '.model // "opus"' "$settings" 2>/dev/null || echo opus)"
effort="$(jq -r '.effortLevel // "high"' "$settings" 2>/dev/null || echo high)"
model="${model%%\[*}"

ahead="$(git rev-list --count origin/main.."$branch" 2>/dev/null || echo 0)"
pushed="not pushed"
git rev-parse --verify --quiet "origin/$branch" >/dev/null 2>&1 && pushed="pushed"
files="$(git diff --name-only origin/main..."$branch" 2>/dev/null | head -12)"

pr="$(gh pr list --repo "$repo" --head "$branch" --state all --json number,state,isDraft,url -q '.[0] // empty')"

# ── the opener ───────────────────────────────────────────────────────────────

echo "Paste this into a new session in the main checkout:"
echo
echo '```'
echo "/model $model"
echo "/effort $effort"
echo
echo "Pick up #$number in intrada: $title"
echo
echo "Read first, the plan comment is the brief:"
echo "  gh issue view $number --comments"
[ -n "$labels" ] && echo "  labels: $labels"
echo "  issue is $state"
echo
echo "Where it stands"
if [ -n "$claimed_branch" ]; then
  echo "  claimed on branch $claimed_branch"
else
  echo "  not claimed yet, so run just claim $number first"
fi
echo "  branch $branch, $ahead commit(s) ahead of origin/main, $pushed"
if [ -n "$pr" ]; then
  pr_number="$(printf '%s' "$pr" | jq -r '.number')"
  pr_state="$(printf '%s' "$pr" | jq -r 'if .isDraft then "draft" else (.state | ascii_downcase) end')"
  echo "  PR #$pr_number, $pr_state: $(printf '%s' "$pr" | jq -r '.url')"
else
  echo "  no PR yet"
fi
if [ -n "$files" ]; then
  echo
  echo "Files already touched on this branch"
  printf '%s\n' "$files" | sed 's/^/  /'
fi
echo
echo "Make your own worktree before editing anything, and prefix every command"
echo "with cd <worktree> && . Do not reuse $branch's worktree."
echo '```'
