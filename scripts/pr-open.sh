#!/usr/bin/env bash
# Wrap `gh pr create`, refusing when an issue number in the title has no
# claim naming this branch (#1702 step 3): a session that inherited an issue
# from its brief without running `just claim` is the second way the
# 2026-09-11 duplicate (#1694) happened, and the claim step alone cannot
# catch that.
#
# Called via `just pr-open <title> <body> [extra gh pr create flags]`.

set -euo pipefail

lib_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/claim-branch.sh
source "$lib_dir/lib/claim-branch.sh"

repo="$(gh repo view --json nameWithOwner -q .nameWithOwner)"
branch="$(git rev-parse --abbrev-ref HEAD)"

title=""
args=("$@")
for i in "${!args[@]}"; do
  case "${args[$i]}" in
    --title | -t)
      title="${args[$((i + 1))]:-}"
      ;;
    --title=*)
      title="${args[$i]#--title=}"
      ;;
  esac
done

if [ -z "$title" ]; then
  echo "✗ pr-open needs an explicit --title (the issue-number check has nothing to read otherwise)." >&2
  exit 1
fi

# ── Body length (#2593) ──
# 43 of 58 bodies in the week to 2026-10-13 broke the old unchecked 400-word limit.
body=""
for i in "${!args[@]}"; do
  case "${args[$i]}" in
    --body) body="${args[$((i + 1))]:-}" ;;
    --body-file)
      body_file="${args[$((i + 1))]:-}"
      if [ -n "$body_file" ]; then body="$(cat "$body_file")"; fi
      ;;
  esac
done

prose="$(printf '%s\n' "$body" | perl -0pe 's/<!--.*?-->//gs; s/^## Checklist.*//ms')"
body_words="$(printf '%s' "$prose" | wc -w | tr -d ' ')"
if [ "$body_words" -gt 350 ] && [ -z "${SKIP_BODY_CHECK:-}" ] &&
  ! printf '%s' "$prose" | perl -0ne '
    while (/^## (?:Where this could bite|What changed where)[^\n]*\n?(.*?)(?=^##|\z)/msg) {
      $found = 1 if $1 =~ /\S/;
    }
    exit($found ? 0 : 1)'; then
  echo "✗ the body runs $body_words words; a Tier 1 or 2 body is at most 350 before the checklist." >&2
  echo "  Cut it, or add the Tier 3 sections if the work is Tier 3 or sensitive; SKIP_BODY_CHECK=1 for a justified case." >&2
  exit 1
fi

printf '%s' "$prose" | perl -0ne '
  s/```.*?```//gs;
  s/^(#{1,6}\s.*)$/$1\n/mg;
  s/\n(?=\s*(?:#|[-*|]|\d+\.)\s)/\n\n/g;
  for my $s (split /(?<=[.!?])\s+|\n\s*\n/) {
    my @w = split " ", $s;
    print STDERR "! sentence over 25 words: \"@w[0..5] ...\"\n" if @w > 25;
  }'

numbers="$(grep -oE '#[0-9]+' <<<"$title" | tr -d '#' | sort -u || true)"
for n in $numbers; do
  claim_body="$(gh issue view "$n" --repo "$repo" --json comments -q \
    '[.comments[] | select(.body | test("^Claimed"; "i"))] | last | .body // empty')"
  claim_branch="$(claim_branch_from_body "$claim_body")"
  if [ -z "$claim_branch" ]; then
    echo "✗ #$n has no claim comment naming a branch: run \`just claim $n\` first." >&2
    exit 1
  fi
  if [ "$claim_branch" != "$branch" ]; then
    echo "✗ #$n is claimed on \`$claim_branch\`, not \`$branch\`: check before opening this PR." >&2
    exit 1
  fi
done

gh pr create "${args[@]}"
