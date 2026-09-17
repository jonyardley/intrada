#!/usr/bin/env bash
# Print the opener for the next session, so clearing costs a paste rather
# than a retelling (#1986). 30 of 93 main sessions in the fortnight to
# 2026-09-17 passed 200k, and 123 turns resumed a parked context at write
# price for $199: the rule to finish a unit and clear has been in CLAUDE.md
# throughout, and friction is why it has not held.
#
# Everything printed is read from git, GitHub and .claude/settings.json.
# Nothing is invented: a fact that cannot be read is named as unreadable
# rather than guessed at, since an opener that lies costs more than one that
# is short.
#
# Called via `just handover [N]`.

set -euo pipefail

usage() {
  echo "Usage: $0 [issue-number]" >&2
  exit 1
}

lib_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/claim-branch.sh
source "$lib_dir/lib/claim-branch.sh"

[ $# -le 1 ] || usage
number="${1:-}"
case "$number" in
  '') ;;
  *[!0-9]*) usage ;;
esac

root="$(git rev-parse --show-toplevel)"
branch="$(git rev-parse --abbrev-ref HEAD)"

if ! git rev-parse --verify --quiet origin/main >/dev/null; then
  echo "✗ no origin/main ref, so nothing can be said about this branch." >&2
  echo "  Try: git fetch origin" >&2
  exit 1
fi

repo="$(gh repo view --json nameWithOwner -q .nameWithOwner)"

if [ -z "$number" ]; then
  number="$(gh pr list --repo "$repo" --head "$branch" --state open --json title -q '.[0].title // empty' |
    grep -oE '#[0-9]+' | head -1 | tr -d '#' || true)"
fi
if [ -z "$number" ]; then
  echo "✗ handover needs an issue number: no open PR on '$branch' has one in its title." >&2
  echo "  Try: just handover 1986" >&2
  exit 1
fi

issue="$(gh issue view "$number" --repo "$repo" --json title,state,url,labels,parent 2>/dev/null || true)"
if [ -z "$issue" ]; then
  echo "✗ cannot read issue #$number from $repo." >&2
  exit 1
fi
title="$(printf '%s' "$issue" | jq -r '.title')"
state="$(printf '%s' "$issue" | jq -r '.state')"
labels="$(printf '%s' "$issue" | jq -r '[.labels[].name] | join(", ")')"
epic_number="$(printf '%s' "$issue" | jq -r '.parent.number // empty')"
epic_title="$(printf '%s' "$issue" | jq -r '.parent.title // empty')"

claim_body="$(gh issue view "$number" --repo "$repo" --json comments -q \
  '[.comments[] | select(.body | test("^Claimed"; "i"))] | last | .body // empty')"
claimed_branch="$(claim_branch_from_body "$claim_body")"

settings="$root/.claude/settings.json"
model=""
effort=""
if jq -e . "$settings" >/dev/null 2>&1; then
  model="$(jq -r '.model // empty' "$settings")"
  effort="$(jq -r '.effortLevel // empty' "$settings")"
  model="${model%%\[*}"
fi

ahead="$(git rev-list --count "origin/main..$branch" 2>/dev/null || true)"
if [ -n "$ahead" ]; then
  position="$ahead commit(s) ahead of origin/main"
else
  position="commits ahead of origin/main unreadable"
fi

unpushed="$(git rev-list --count "origin/$branch..HEAD" 2>/dev/null || true)"
if [ -z "$unpushed" ]; then
  pushed="not pushed"
elif [ "$unpushed" = "0" ]; then
  pushed="pushed"
else
  pushed="$unpushed commit(s) not pushed"
fi

files="$(git diff --name-only "origin/main...$branch" | sed -n '1,12p')"

pr="$(gh pr list --repo "$repo" --head "$branch" --state all --json number,state,isDraft,url \
  -q '([.[] | select(.state == "OPEN")] + .) | .[0] // empty')"

# ── the opener ───────────────────────────────────────────────────────────────

echo "Paste this into a new session in the main checkout:"
echo
echo '```'
if [ -n "$model" ] && [ -n "$effort" ]; then
  echo "/model $model"
  echo "/effort $effort"
else
  echo "Set /model and /effort by hand: neither could be read from .claude/settings.json."
fi
echo
echo "Pick up #$number in intrada: $title"
echo
echo "Read first, the plan comment is the brief:"
echo "  gh issue view $number --comments"
[ -n "$epic_number" ] && echo "  epic #$epic_number: $epic_title"
[ -n "$labels" ] && echo "  labels: $labels"
echo "  issue is $state"
echo
echo "Where it stands"
if [ -n "$claimed_branch" ]; then
  echo "  claimed on branch $claimed_branch"
else
  echo "  not claimed yet, so run just claim $number first"
fi
echo "  branch $branch, $position, $pushed"
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
if [ "$claimed_branch" != "$branch" ] && [ "$ahead" = "0" ]; then
  echo "Make your own worktree before editing anything, and prefix every command"
  echo "with cd <worktree> && ."
else
  echo "The work is in this worktree, so carry on there rather than making a new"
  echo "one, which would branch fresh from origin/main and leave it behind:"
  echo "  cd $root"
  echo "Its lease releases when this session ends, so start once it has, and"
  echo "prefix every command with cd $root && ."
fi
echo '```'
