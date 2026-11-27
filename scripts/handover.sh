#!/usr/bin/env bash
# Print the opener for the next session, so clearing costs a paste rather
# than a retelling (#1986). 30 of 93 main sessions in the fortnight to
# 2026-09-17 passed 200k, and 123 turns resumed a parked context at write
# price for $199: the rule to finish a unit and clear has been in CLAUDE.md
# throughout, and friction is why it has not held.
#
# Everything printed is read from git and GitHub. The opener names no model
# or effort: the new session opens on Jon's global user settings.
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

# Only the owner's claim counts, as in claim-issue.sh: anyone can comment on a
# public repository.
claim_body="$(gh issue view "$number" --repo "$repo" --json comments |
  jq -r --arg owner "${repo%%/*}" '[.comments[] | select(.author.login == $owner and (.body | test("^Claimed"; "i")))] | last | .body // empty')"
claimed_branch="$(claim_branch_from_body "$claim_body")"

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

prs="$(gh pr list --repo "$repo" --head "$branch" --state all --json number,state,isDraft,url)"
pr="$(printf '%s' "$prs" | jq -c '([.[] | select(.state == "OPEN")] + .) | .[0] // empty')"
merged="$(printf '%s' "$prs" | jq -r 'any(.[]; .state == "MERGED")')"

# Only an issue claimed on this branch, with no merged PR, carries on here (#2969).
resume=""
elsewhere=""
if [ -n "$claimed_branch" ] && [ "$claimed_branch" = "$branch" ] && [ "$merged" != "true" ]; then
  resume=1
elif [ -n "$claimed_branch" ] && [ "$claimed_branch" != "$branch" ]; then
  elsewhere=1
fi

# ── the opener ───────────────────────────────────────────────────────────────

if [ -n "$resume" ]; then
  echo "Once this session ends, paste this into a new session in this worktree:"
elif [ -n "$elsewhere" ]; then
  echo "The issue is claimed on branch $claimed_branch, not this one. Work in that"
  echo "branch's worktree (git worktree list names it), or ask the owner before"
  echo "taking it over. Paste this into a session started there:"
else
  echo "Start from fresh main: in the main checkout run"
  echo "  just worktree-new <name>"
  echo "then start a session inside the new worktree with the command it prints,"
  echo "and paste this there:"
fi
echo
echo '```'
echo "Pick up #$number in intrada: $title"
echo
echo "Read first, the plan comment is the brief:"
echo "  just issue $number"
[ -n "$epic_number" ] && echo "  epic #$epic_number: $epic_title"
[ -n "$labels" ] && echo "  labels: $labels"
echo "  issue is $state"
echo
echo "Where it stands"
if [ -n "$claimed_branch" ]; then
  echo "  claimed on branch $claimed_branch"
else
  echo "  not claimed yet"
fi
if [ -n "$resume" ]; then
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
fi
echo
if [ -n "$resume" ]; then
  echo "The work is in this worktree, so carry on there rather than making a new"
  echo "one, which would branch fresh from origin/main and leave it behind:"
  echo "  cd $root && claude"
  echo "Its lease releases when this session ends, so start once it has."
elif [ -n "$elsewhere" ]; then
  echo "This session is in the worktree for branch $claimed_branch, where the"
  echo "work is; if that branch has no worktree, ask the owner before starting."
elif [ -z "$claimed_branch" ]; then
  echo "This session is in a fresh worktree from main. Claim the issue first:"
  echo "  just claim $number"
else
  # The claim names this branch, whose PR has merged, so `just claim` in the
  # new worktree would refuse: the fresh claim is the one claim-issue.sh asks for.
  echo "This session is in a fresh worktree from main. The claim still names the"
  echo "merged branch $claimed_branch, so first post a fresh comment on the issue:"
  echo "  Claimed: branch \`<new-branch>\`"
  echo "naming this worktree's branch, then run:"
  echo "  just claim $number"
fi
echo '```'
