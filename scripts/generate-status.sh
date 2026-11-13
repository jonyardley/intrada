#!/usr/bin/env bash
# Print what is in flight, read from GitHub. The issues are the source of truth
# (CLAUDE.md), so this was always derivable; it used to be copied by hand into
# docs/status.md, which every concurrent branch then collided on. Nothing here
# writes a tracked file, which is the whole point.

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

repo="jonyardley/intrada"
open_limit=50
claimed_limit=50
landed_limit=15

for tool in gh jq; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "❌ $tool is required" >&2
    exit 1
  }
done

open_prs=$(gh pr list --repo "$repo" --state open --limit "$open_limit" \
  --json number,title,isDraft,updatedAt,headRefName,closingIssuesReferences)
# --state merged sorts by creation, so a long-lived branch merged today can
# fall outside a window that newer-but-earlier-merged PRs stay in. Over-fetch
# and sort on mergedAt instead.
merged_prs=$(gh pr list --repo "$repo" --state merged --limit $((landed_limit * 6)) \
  --json number,title,mergedAt)
claimed=$(gh issue list --repo "$repo" --label in-flight --state open \
  --limit "$claimed_limit" --json number,title,updatedAt)

# The cut is gated on the milestone's headline, not on a date (docs/how-the-work-runs.md),
# so "is a release due" is only answerable next to the burn.
#
# Each read reports its own failure. An outage that printed an empty section
# would read as "no release in flight", which is the silent-wrong this script
# exists to remove, and swallowing the tags call took the whole script down
# with it under `set -e`.
milestones_error=""
if ! milestones=$(gh api "repos/$repo/milestones?state=open" 2>&1); then
  milestones_error=$(printf '%s' "$milestones" | head -1)
  milestones='[]'
fi

# A failed read joins gh's message onto the JSON body with no newline (#2806),
# so the reason is what follows the last "gh: "; with none, the first line stands.
gh_reason() {
  local reason
  reason=$(printf '%s\n' "$1" | sed -n 's/.*gh: //p' | tail -1)
  [ -n "$reason" ] || reason=$(printf '%s' "$1" | head -1)
  printf '%s' "$reason"
}

# The epics are the backlog's containers (#1968): the next slice is picked from
# one of them, not from the flat list of open issues. GitHub caps an epic at
# 100 sub-issues, so one page of children is all of them.
epics_limit=30
epics_error=""
epics_query='query($owner: String!, $name: String!, $limit: Int!) {
  repository(owner: $owner, name: $name) {
    issues(labels: ["epic"], states: OPEN, first: $limit,
           orderBy: {field: CREATED_AT, direction: ASC}) {
      nodes { number title subIssues(first: 100) { nodes { number title state } } }
    }
  }
}'
if ! epics=$(gh api graphql -f query="$epics_query" -f owner="${repo%%/*}" \
  -f name="${repo#*/}" -F limit="$epics_limit" --jq '.data.repository.issues.nodes' 2>&1); then
  epics_error=$(gh_reason "$epics")
  epics='[]'
fi

# The ready queue (#2800). A claimed issue is left out: CLAIMED or IN FLIGHT already shows it.
# An epic is left out too: it is approved through its issues, not built (#2808).
ready_limit=50
ready_error=""
ready_query='query($owner: String!, $name: String!, $limit: Int!) {
  repository(owner: $owner, name: $name) {
    issues(labels: ["ready"], states: OPEN, first: $limit,
           orderBy: {field: CREATED_AT, direction: ASC}) {
      nodes {
        number title
        labels(first: 30) { nodes { name } }
        projectItems(first: 10) { nodes {
          project { number }
          priority: fieldValueByName(name: "Priority") {
            ... on ProjectV2ItemFieldSingleSelectValue { name }
          }
        } }
      }
    }
  }
}'
if ! ready=$(gh api graphql -f query="$ready_query" -f owner="${repo%%/*}" \
  -f name="${repo#*/}" -F limit="$ready_limit" --jq '.data.repository.issues.nodes' 2>&1); then
  ready_error=$(gh_reason "$ready")
  ready='[]'
fi

last_tag=""
tags_error=""
if tags=$(gh api "repos/$repo/tags?per_page=100" --jq '.[].name' 2>&1); then
  last_tag=$(printf '%s\n' "$tags" |
    grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -1 || true)
else
  tags_error=$(printf '%s' "$tags" | head -1)
fi

# Interpolated into a line the reader trusts, so anything but a count is
# dropped rather than printed.
ahead=""
if [ -n "$last_tag" ]; then
  ahead=$(gh api "repos/$repo/compare/$last_tag...main" --jq '.ahead_by' 2>/dev/null || echo "")
  case "$ahead" in
    "" | *[!0-9]*) ahead="" ;;
  esac
fi

# A cut tag whose release is never published leaves the history quiet
# without failing (#1961): v0.12.0 and v0.13.0 sat as drafts unnoticed.
drafts=""
drafts_error=""
if drafts=$(gh api "repos/$repo/releases?per_page=100" \
  --jq '[.[] | select(.draft)] | length' 2>&1); then
  case "$drafts" in
    "" | *[!0-9]*)
      drafts_error="the releases read answered something other than a count"
      drafts=""
      ;;
  esac
else
  drafts_error=$(printf '%s' "$drafts" | head -1)
  drafts=""
fi

# `capture` emits nothing rather than null when it does not match, which would
# drop the whole row, so it is defaulted before the field is read.
issue_of='(.closingIssuesReferences[0].number
  // ((.title | capture("#(?<n>[0-9]+)\\)\\s*$") // null)
      | if . then (.n | tonumber) else null end))'

pr_lines=$(printf '%s' "$open_prs" | jq -r "
  sort_by(.updatedAt) | reverse | .[] |
  $issue_of as \$issue |
  \"- #\(.number) — \(.title)\"
  + (if .isDraft then \" (draft)\" else \"\" end)
  + (if \$issue then \" — issue #\(\$issue)\" else \"\" end)
  + \"\n    \(.headRefName), last touched \(.updatedAt | split(\"T\")[0])\"
")

claimed_lines=$(printf '%s\n%s' "$claimed" "$open_prs" | jq -rs '
  .[0] as $issues | .[1] as $prs |
  ($prs | map(
     (.closingIssuesReferences[].number),
     ((.title | capture("#(?<n>[0-9]+)\\)\\s*$") // null)
      | if . then (.n | tonumber) else empty end)
   )) as $covered |
  $issues | sort_by(.updatedAt) | reverse | .[] |
  select([.number] | inside($covered) | not) |
  "- #\(.number) — \(.title)
    last touched \(.updatedAt | split("T")[0])"
')

# The headline is the description's first line (docs/how-the-work-runs.md): a sentence
# split runs past a newline and breaks the layout, and truncates "i.e." besides.
release_lines=$(printf '%s' "$milestones" | jq -r '
  sort_by(.number) | .[] |
  "- \(.title): \((.description // "") | split("\n") | (.[0] // "") | if . == "" then "no headline written" else . end)
    \(.closed_issues // 0) of \((.closed_issues // 0) + (.open_issues // 0)) closed"
')

# Next is the first open child in the epic's own order that nobody has claimed:
# the epic body names the working order and the sub-issue list mirrors it.
epic_lines=$(printf '%s\n%s' "$epics" "$claimed" | jq -rs '
  .[0] as $epics | [.[1][].number] as $flying |
  $epics[] |
  .subIssues.nodes as $kids |
  ($kids | length) as $total |
  ($kids | map(select(.state == "CLOSED")) | length) as $done |
  ($kids | map(select(.state == "OPEN")
    | select([.number] | inside($flying)))) as $live |
  ($kids | map(select(.state == "OPEN")
    | select([.number] | inside($flying) | not)) | first) as $next |
  "- #\(.number) \(.title)\n    "
  + (if $total == 0 then "no sub-issues yet"
     elif $done == $total then "all \($total) done: close it"
     else "\($done) of \($total) done"
       + (if ($live | length) > 0
          then ", in flight: " + ($live | map("#\(.number)") | join(" "))
          else "" end)
       + (if $next then "\n    next: #\($next.number) \($next.title)"
          else "; every open one is in flight" end)
     end)
')

ready_lines=$(printf '%s\n%s' "$ready" "$claimed" | jq -rs '
  .[0] as $ready | [.[1][].number] as $flying |
  {"horizon:now": 0, "horizon:next": 1, "horizon:later": 2} as $horizons |
  {"P0": 0, "P1": 1, "P2": 2} as $priorities |
  $ready | map(select([.number] | inside($flying) | not)
    | select([(.labels.nodes // [])[].name] | index("epic") | not)
    | ([(.labels.nodes // [])[].name | select(startswith("horizon:"))] | first) as $h
    | ([(.projectItems.nodes // [])[] | select(.project.number == 2) | .priority.name
        | select(. != null)] | first) as $p
    | . + {h: $h, p: $p, hr: ($horizons[$h // ""] // 3), pr: ($priorities[$p // ""] // 3)})
  | sort_by(.hr, .pr, .number) | .[] |
  "- #\(.number) \(.title)\n    "
  + (if .h then (.h | ltrimstr("horizon:")) else "no horizon" end)
  + ", " + (.p // "no priority")
')

landed_lines=$(printf '%s' "$merged_prs" | jq -r "
  sort_by(.mergedAt) | reverse | .[:$landed_limit] | .[] |
  \"- #\(.number) — \(.title) (merged \(.mergedAt | split(\"T\")[0]))\"
")

# A cap that bit silently would be the same silent-wrong this replaced.
capped() {
  if [ "$(printf '%s' "$1" | jq 'length')" -ge "$2" ]; then
    echo "    (showing the first $2; there are more)"
  fi
}

section() {
  echo
  echo "$1"
  echo
  if [ -n "$2" ]; then echo "$2"; else echo "    nothing"; fi
}

echo "What's in flight in $repo, read from GitHub just now."
# The phase is the roadmap's Now heading line; a heading that moved says so
# rather than printing nothing.
now_heading=$(grep -m1 '^### Now' docs/roadmap.md || true)
echo
if [ -n "$now_heading" ]; then
  printf '%s\n' "$now_heading"
else
  echo "    No '### Now' heading in docs/roadmap.md, so the phase is not shown."
fi
echo "Direction: docs/roadmap.md."

section "RELEASE (open milestones)" "$release_lines"
echo
if [ -n "$milestones_error" ]; then
  echo "    GitHub did not answer, so this section is not an answer either:"
  echo "    $milestones_error"
fi
# Once, not per milestone: the distance is repo wide and reads as that
# milestone's own burn when it sits on the row.
if [ -n "$ahead" ]; then
  echo "    $ahead commits on main since $last_tag."
elif [ -n "$tags_error" ]; then
  echo "    Could not read the tags, so there is no distance from the last"
  echo "    release: $tags_error"
fi
if [ -n "$drafts" ] && [ "$drafts" -gt 0 ]; then
  noun="draft releases"
  [ "$drafts" -eq 1 ] && noun="draft release"
  echo "    $drafts $noun not yet published: gh release list."
elif [ -n "$drafts_error" ]; then
  echo "    Could not read the releases, so drafts are not counted: $drafts_error"
fi
echo "    Cut when the headline works on the phone, then roll whatever is"
echo "    still open into the next milestone. See docs/how-the-work-runs.md."

section "EPICS (open, oldest first)" "$epic_lines"
echo
if [ -n "$epics_error" ]; then
  echo "    GitHub did not answer, so this section is not an answer either:"
  echo "    $epics_error"
fi
capped "$epics" "$epics_limit"
echo "    Pick the next slice from an epic. An open issue under none is a"
echo "    one-off, or a sign a new epic is due (how-the-work-runs.md, Epics)."

section "READY (the owner's sign-off, top first)" "$ready_lines"
echo
if [ -n "$ready_error" ]; then
  echo "    GitHub did not answer, so this section is not an answer either:"
  echo "    $ready_error"
fi
capped "$ready" "$ready_limit"
echo "    Only the owner adds the ready label: the plan, and any mockup, are approved."
echo "    Order: horizon, then the board's Priority, then oldest first. Build from"
echo "    here first; an epic's next child waits until the owner marks it ready."

section "IN FLIGHT (open PRs)" "$pr_lines"
capped "$open_prs" "$open_limit"

section "CLAIMED, NO PR YET (in-flight label)" "$claimed_lines"
echo
echo "    An issue here is being worked on without a PR yet, or its label"
echo "    outlived the PR that closed it, or its PR names it neither in the"
echo "    title nor with a closing keyword."
capped "$claimed" "$claimed_limit"

section "RECENTLY LANDED" "$landed_lines"
echo
