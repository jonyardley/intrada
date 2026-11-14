#!/usr/bin/env bash
# Does a ready issue qualify for the crew (#2819, the rules in #2803 and
# specs/crew-loop.md)? One GraphQL read, so the self-test's stand-in gh can
# serve it canned JSON.
#
#   crew-pick.sh <issue>   exit 0 when it qualifies; 1 when it is skipped, with
#                          the reasons; 2 when GitHub cannot be read
#
# A skip posts one comment headed Crew skipped, and no second one until the
# owner edits the plan or adds the ready label again: the loop re-reads the
# queue every run and must not repeat itself.

set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO=${CREW_REPO:-jonyardley/intrada}
# shellcheck source=crew-trusted.sh
. "$root/scripts/crew-trusted.sh"
# Abbreviated as plans write them (Views/LibraryView.swift, ui/library/): erring
# towards a screen only asks for a mockup.
SCREEN_PATHS='(^|[^[:alnum:]_.-])(Views|ui)/'

if [ $# -ne 1 ] || ! [[ $1 =~ ^[0-9]+$ ]]; then
  echo "usage: $0 <issue-number>" >&2
  exit 2
fi
n=$1

query='query($owner: String!, $name: String!, $n: Int!) {
  repository(owner: $owner, name: $name) {
    issue(number: $n) {
      state body author { login } editor { login }
      labels(first: 50) { nodes { name } }
      comments(last: 100) { nodes { author { login } body createdAt lastEditedAt } }
      timelineItems(itemTypes: [LABELED_EVENT], last: 100) { nodes { ... on LabeledEvent { createdAt label { name } } } }
      blockedBy(first: 50) { nodes { number state } }
    }
  }
}'
tmp=$(mktemp -d "${TMPDIR:-/tmp}/crew-pick.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
raw=$(gh api graphql -f query="$query" -f owner="${REPO%%/*}" -f name="${REPO#*/}" -F n="$n" 2>"$tmp/err") || {
  echo "crew-pick: could not read #$n: $(tail -1 "$tmp/err")" >&2
  exit 2
}
issue=$(printf '%s' "$raw" | jq -ce '.data.repository.issue // empty' 2>/dev/null) || {
  echo "crew-pick: no issue #$n in the reply" >&2
  exit 2
}
get() { printf '%s' "$issue" | jq -r "$1" | tr -d '\r'; }

if [ "$(get '.state')" != OPEN ] || ! grep -qx ready <<<"$(get '.labels.nodes[].name')"; then
  echo "#$n is not open with the ready label: not in the crew's queue"
  exit 1
fi

reasons=""
fail() { reasons="$reasons- $1"$'\n'; }

# The body only when the owner or the agent account opened the issue and last
# edited it: anyone's issue can carry the ready label, and its author can edit
# the body afterwards. An untrusted body is never read, so no line of it is
# quoted in the Crew skipped comment a trusted login posts.
body=""
if ! printf '%s' "$issue" | jq -e "$(trusted_jq)"' trusted' >/dev/null 2>&1; then
  fail "The issue was opened by someone other than the owner or the agent account, so the crew does not read its body: open it again as the owner."
elif ! printf '%s' "$issue" | jq -e "$(trusted_jq)"' trusted_body' >/dev/null 2>&1; then
  fail "The issue body was last edited by someone other than the owner or the agent account: edit it once as the owner to vouch for it."
else
  body=$(get '.body')
fi
# Only a plan by a trusted login, since anyone can comment on a public issue.
plan=$(get "$(trusted_jq)"'[.comments.nodes[] | select(trusted) | select((.body | test("\\*\\*Done looks like:?\\*\\*")) and (.body | test("\\*\\*Files:?\\*\\*")))] | last // empty')
plan_body=$(printf '%s' "$plan" | jq -r '.body // empty' 2>/dev/null | tr -d '\r')
plan_edit=$(printf '%s' "$plan" | jq -r '.lastEditedAt // .createdAt // empty' 2>/dev/null)
ready_at=$(get '[.timelineItems.nodes[] | select(.label.name == "ready") | .createdAt] | max // empty')
# A heading's text: written **H**: or **H:**, on its own line or not, with any
# list items under it, since a long Files line becomes a list.
plan_line() {
  printf '%s\n' "$plan_body" | awk -v h="$1" '
    on && /^[[:space:]]*([-*]|[0-9]+\.)[[:space:]]/ { sub(/^[[:space:]]*([-*]|[0-9]+\.)[[:space:]]+/, ""); print; items = 1; next }
    on && /^[[:space:]]*$/ && !items { next }
    on { exit }
    index($0, "**" h "**") == 1 || index($0, "**" h ":**") == 1 {
      rest = substr($0, length(h) + 5); sub(/^:?\**:?[[:space:]]*/, "", rest)
      if (rest != "") print rest
      on = 1
    }'
}

# Every issue needs a plan, whatever its tier: a tier guessed from the body
# would let an unplanned issue through.
if [ -z "$plan" ]; then
  fail "No plan comment in the shipping skill's skeleton: the crew builds only a plan with a Done looks like line and a Files line."
else
  for h in Files Tests Routing; do
    [ -n "$(plan_line "$h")" ] || fail "The plan comment has no $h line."
  done
  if [[ ! $ready_at > $plan_edit ]]; then
    fail "The ready label (added ${ready_at:-never}) is older than the plan's last edit ($plan_edit): approve the plan as it now reads by adding the label again."
  fi
fi

done_lines=$(printf '%s\n' "$body" | awk '
  /^#+ *Done looks like/ { on = 1; next }
  on && /^#/ { on = 0 }
  on && /^([-*]|[0-9]+\.) / { sub(/^([-*]|[0-9]+\.) +/, ""); print }
')
plan_done=$(plan_line 'Done looks like')
if grep -qE '^#+ *Done looks like' <<<"$body"; then
  [ -n "$done_lines" ] || fail "The body's Done looks like heading has no list items under it."
elif [ -n "$plan_done" ]; then
  done_lines=$(printf '%s\n' "$plan_done" | tr ';' '\n' | sed 's/^ *//; /^$/d')
fi
if [ -z "$done_lines" ]; then
  fail "No Done looks like lines, in the issue body or the plan."
fi
while IFS= read -r line; do
  [ -n "$line" ] || continue
  grep -qiE '(^|[^[:alnum:]])(self-test|tests?|snapshots?|screenshots?)([^[:alnum:]-]|$)' <<<"$line" ||
    fail "A Done looks like line names no test, snapshot or screenshot as its evidence: \"$line\""
done <<<"$done_lines"
if grep -qi 'needs your call' <<<"$done_lines"$'\n'"$plan_done"; then
  fail "A Done looks like line still needs your call."
fi

files=$(plan_line Files)
if grep -qE "$SCREEN_PATHS" <<<"$files"; then
  mockup=$(plan_line Mockup)
  if ! grep -q 'https://claude\.ai/' <<<"$mockup" ||
    ! grep -qiE '(^|[^[:alnum:]])approved( the mockup)? on [0-9]{4}-[0-9]{2}-[0-9]{2}' <<<"$mockup" ||
    grep -qi 'not approved' <<<"$mockup"; then
    fail "A screen issue needs a Mockup line with the canvas link and the owner's word, \"approved the mockup on\" a date."
  fi
fi

blockers=$(printf '%s' "$issue" | jq -er '[(.blockedBy.nodes // error("no blockedBy")) [] | select(.state == "OPEN") | "#\(.number)"] | join(", ")' 2>/dev/null) || {
  echo "crew-pick: no blocked-by list for #$n in the reply" >&2
  exit 2
}
[ -z "$blockers" ] || fail "Blocked by $blockers, still open."

if [ -n "$plan" ]; then
  park_in=$tmp/park-input
  printf '%s\n' "$plan_body" >"$park_in"
  park_out=$("$root/scripts/crew-park.sh" "$park_in" 2>&1)
  case $? in
    0) ;;
    3) fail "The plan names a sensitive surface, which the lead builds: $(printf '%s' "$park_out" | sed 's/^park: //' | paste -sd ';' -)." ;;
    *) fail "The park step could not run: $(printf '%s' "$park_out" | head -1)" ;;
  esac
fi

if [ -z "$reasons" ]; then
  echo "#$n qualifies"
  exit 0
fi

printf '#%s skipped:\n%s' "$n" "$reasons"
since=$ready_at
[[ $plan_edit > $since ]] && since=$plan_edit
said=$(printf '%s' "$issue" | jq -r --arg s "$since" \
  "$(trusted_jq)"'[.comments.nodes[] | select(trusted and (.body | test("^(\\*\\*)?Crew skipped")) and .createdAt > $s)] | length')
if [ "$said" != 0 ]; then
  echo "Already said on #$n since the plan or the label last changed; no new comment."
  exit 1
fi
comment="**Crew skipped**: the crew passes over this issue while any reason below holds, and says so once until the plan is edited or the ready label is added again.

$reasons"
if gh api "repos/$REPO/issues/$n/comments" -f body="$comment" >/dev/null 2>&1; then
  echo "Commented on #$n."
else
  echo "crew-pick: could not post the Crew skipped comment on #$n" >&2
fi
exit 1
