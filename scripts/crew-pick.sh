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
# queue every night and must not repeat itself.

set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO=${CREW_REPO:-jonyardley/intrada}
SCREEN_PATHS='ios/Intrada/Views/|android/app/src/main/kotlin/com/intrada/android/ui/'

if [ $# -ne 1 ] || ! [[ $1 =~ ^[0-9]+$ ]]; then
  echo "usage: $0 <issue-number>" >&2
  exit 2
fi
n=$1

query='query($owner: String!, $name: String!, $n: Int!) {
  repository(owner: $owner, name: $name) {
    issue(number: $n) {
      state body
      labels(first: 50) { nodes { name } }
      comments(last: 100) { nodes { body createdAt lastEditedAt } }
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

if [ "$(get '.state')" != OPEN ] || ! get '.labels.nodes[].name' | grep -qx ready; then
  echo "#$n is not open with the ready label: not in the crew's queue"
  exit 1
fi

reasons=""
fail() { reasons="$reasons- $1"$'\n'; }

body=$(get '.body')
plan=$(get '[.comments.nodes[] | select((.body | contains("**Done looks like**")) and (.body | contains("**Files**")))] | last // empty')
plan_body=$(printf '%s' "$plan" | jq -r '.body // empty' 2>/dev/null | tr -d '\r')
plan_edit=$(printf '%s' "$plan" | jq -r '.lastEditedAt // .createdAt // empty' 2>/dev/null)
ready_at=$(get '[.timelineItems.nodes[] | select(.label.name == "ready") | .createdAt] | max // empty')
plan_line() { printf '%s\n' "$plan_body" | grep -m1 "^\*\*$1\*\*:" | sed "s/^\*\*$1\*\*: *//"; }

# Every issue needs a plan, whatever its tier: a tier guessed from the body
# would let an unplanned issue through.
if [ -z "$plan" ]; then
  fail "No plan comment in the shipping skill's skeleton: the crew builds only a plan with a Done looks like line and a Files line."
else
  for h in Tests Routing; do
    [ -n "$(plan_line "$h")" ] || fail "The plan comment has no $h line."
  done
  if [[ ! $ready_at > $plan_edit ]]; then
    fail "The ready label (added ${ready_at:-never}) is older than the plan's last edit ($plan_edit): approve the plan as it now reads by adding the label again."
  fi
fi

done_lines=$(printf '%s\n' "$body" | awk '
  /^#+ *Done looks like/ { on = 1; next }
  on && /^#/ { on = 0 }
  on && /^[-*] / { sub(/^[-*] +/, ""); print }
')
plan_done=$(plan_line 'Done looks like')
if [ -z "$done_lines" ] && [ -n "$plan_done" ]; then
  done_lines=$(printf '%s\n' "$plan_done" | tr ';' '\n' | sed 's/^ *//; /^$/d')
fi
if [ -z "$done_lines" ]; then
  fail "No Done looks like lines, in the issue body or the plan."
fi
while IFS= read -r line; do
  [ -n "$line" ] || continue
  printf '%s' "$line" | grep -qiE 'test|snapshot|screenshot' ||
    fail "A Done looks like line names no test, snapshot or screenshot as its evidence: \"$line\""
done <<<"$done_lines"
if printf '%s\n%s\n' "$done_lines" "$plan_done" | grep -qi 'needs your call'; then
  fail "A Done looks like line still needs your call."
fi

files=$(plan_line Files)
if printf '%s' "$files" | grep -qE "$SCREEN_PATHS"; then
  mockup=$(plan_line Mockup)
  if ! printf '%s' "$mockup" | grep -q 'https://claude\.ai/' ||
    ! printf '%s' "$mockup" | grep -qiE 'approved[^0-9]*[0-9]{4}-[0-9]{2}-[0-9]{2}'; then
    fail "A screen issue needs a Mockup line with the canvas link and the owner's dated approval."
  fi
fi

blockers=$(get '[.blockedBy.nodes[] | select(.state == "OPEN") | "#\(.number)"] | join(", ")')
[ -z "$blockers" ] || fail "Blocked by $blockers, still open."

if [ -n "$plan" ]; then
  park_in=$tmp/park-input
  printf '%s\n' "$files" >"$park_in"
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
  '[.comments.nodes[] | select((.body | test("^(\\*\\*)?Crew skipped")) and .createdAt > $s)] | length')
if [ "$said" != 0 ]; then
  echo "Already said on #$n since the plan or the label last changed; no new comment."
  exit 1
fi
comment="**Crew skipped**: the crew passes over this issue until the plan is edited or the ready label is added again.

$reasons"
if gh api "repos/$REPO/issues/$n/comments" -f body="$comment" >/dev/null 2>&1; then
  echo "Commented on #$n."
else
  echo "crew-pick: could not post the Crew skipped comment on #$n" >&2
fi
exit 1
