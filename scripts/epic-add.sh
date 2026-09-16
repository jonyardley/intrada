#!/usr/bin/env bash
# Attach issues to an epic as GitHub sub-issues (#1968).
#
# The convention it holds (CLAUDE.md, Workflow): the parent carries the epic
# label, an epic never sits under another epic, and an issue has one parent.
# Moving an issue that already has a different parent needs --move, so a
# grouping pass never silently steals an audit finding from its audit.
#
# Every child is checked before anything is written: a refusal half way down
# the list would leave the epic half built. An outage mid-write can still do
# that; a re-run finishes it, since children already attached are skipped.
#
# Called via `just epic-add PARENT CHILD...` or `just epic-move PARENT CHILD...`.

set -euo pipefail

usage() {
  echo "Usage: $0 [--move] <epic-number> <issue-number>..." >&2
  exit 1
}

move=false
if [ "${1:-}" = "--move" ]; then
  move=true
  shift
fi
[ $# -ge 2 ] || usage
for n in "$@"; do
  case "$n" in
    '' | *[!0-9]*) usage ;;
  esac
done

parent="$1"
shift

repo="$(gh repo view --json nameWithOwner -q .nameWithOwner)"

read_issue() {
  gh issue view "$1" --repo "$repo" --json id,number,title,labels,parent
}

is_epic='([.labels[].name] | index("epic")) != null'

parent_json="$(read_issue "$parent")"
if [ "$(printf '%s' "$parent_json" | jq -r "$is_epic")" != "true" ]; then
  echo "✗ #$parent has no epic label: label it first, or pick the right parent." >&2
  exit 1
fi
grandparent="$(printf '%s' "$parent_json" | jq -r '.parent.number // empty')"
if [ -n "$grandparent" ]; then
  echo "✗ #$parent sits under #$grandparent: epics are one level deep." >&2
  exit 1
fi
parent_id="$(printf '%s' "$parent_json" | jq -r .id)"
parent_title="$(printf '%s' "$parent_json" | jq -r .title)"

ids=()
nums=()
refused=0
for child in "$@"; do
  if [ "$child" = "$parent" ]; then
    echo "✗ #$child cannot be its own sub-issue." >&2
    refused=1
    continue
  fi
  child_json="$(read_issue "$child")"
  if [ "$(printf '%s' "$child_json" | jq -r "$is_epic")" = "true" ]; then
    echo "✗ #$child is an epic itself: epics are one level deep." >&2
    refused=1
    continue
  fi
  current="$(printf '%s' "$child_json" | jq -r '.parent.number // empty')"
  if [ "$current" = "$parent" ]; then
    echo "  #$child is already under #$parent"
    continue
  fi
  if [ -n "$current" ] && [ "$move" != "true" ]; then
    current_title="$(printf '%s' "$child_json" | jq -r '.parent.title // ""')"
    echo "✗ #$child is already under #$current ($current_title): use epic-move to take it." >&2
    refused=1
    continue
  fi
  ids+=("$(printf '%s' "$child_json" | jq -r .id)")
  nums+=("$child")
done

if [ "$refused" -ne 0 ]; then
  echo "✗ nothing attached to #$parent." >&2
  exit 1
fi

if [ "${#ids[@]}" -eq 0 ]; then
  echo "#$parent $parent_title: nothing new to attach"
  exit 0
fi

mutation='mutation($parent: ID!, $child: ID!, $replace: Boolean!) {
  addSubIssue(input: {issueId: $parent, subIssueId: $child, replaceParent: $replace}) {
    subIssue { number }
  }
}'

for i in "${!ids[@]}"; do
  gh api graphql -f query="$mutation" -f parent="$parent_id" -f child="${ids[$i]}" \
    -F replace="$move" >/dev/null
  echo "✓ #${nums[$i]} under #$parent"
done

echo "#$parent $parent_title: ${#ids[@]} attached"
