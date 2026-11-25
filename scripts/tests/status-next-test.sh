#!/usr/bin/env bash
# Self-test for the NEXT UP block of scripts/generate-status.sh (#2800). A fake
# `gh` on PATH serves real-shaped payloads and honours --jq, so the script's
# own jq programs are what the assertions exercise: every open issue with no
# label filter, every page joined, the order by horizon then Priority then age,
# claimed ones dropped, epics left out, the top few shown, and the read failing.
set -euo pipefail
source "$(dirname "$0")/../lib/contains.sh"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/generate-status.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/bin" "$tmp/fixtures"

cat >"$tmp/bin/gh" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail

jq_expr=""
args=("$@")
for i in "${!args[@]}"; do
  if [ "${args[$i]}" = "--jq" ]; then jq_expr="${args[$((i + 1))]}"; fi
done

body='[]'
case "$1 ${2:-}" in
  "api graphql")
    body='{"data":{"repository":{"issues":{"nodes":[]}}}}'
    if [[ "$*" == *'issues(states: OPEN'* ]]; then
      if [ -n "${FAIL_NEXT:-}" ]; then
        printf '%s' '{"errors":[{"type":"INSUFFICIENT_SCOPES","message":"Your token has not been granted the required scopes."}]}'
        echo "gh: Your token has not been granted the required scopes." >&2
        exit 1
      fi
      printf '%s\n' "$*" >"$FIXTURES/query.txt"
      # Two pages, as --paginate reads them; --jq applies to each.
      if [ -n "$jq_expr" ]; then
        jq -r "$jq_expr" "$FIXTURES/page1.json"
        jq -r "$jq_expr" "$FIXTURES/page2.json"
      else
        cat "$FIXTURES/page1.json" "$FIXTURES/page2.json"
      fi
      exit 0
    fi
    ;;
  "issue list")
    body="$(cat "$FIXTURES/claimed.json")"
    ;;
esac

if [ -n "$jq_expr" ]; then
  printf '%s' "$body" | jq -r "$jq_expr"
else
  printf '%s' "$body"
fi
FAKE
chmod +x "$tmp/bin/gh"

export FIXTURES="$tmp/fixtures"
export PATH="$tmp/bin:$PATH"

pass=0
fail=0

check() {
  local haystack="$1" desc="$2" needle="$3"
  if contains "$needle" "$haystack"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf '✗ %s\n    expected to find: %s\n' "$desc" "$needle" >&2
  fi
}

refute() {
  local haystack="$1" desc="$2" needle="$3"
  if contains "$needle" "$haystack"; then
    fail=$((fail + 1))
    printf '✗ %s\n    did not expect: %s\n' "$desc" "$needle" >&2
  else
    pass=$((pass + 1))
  fi
}

open_issues() {
  printf '{"data":{"repository":{"issues":{"nodes":%s}}}}' "$1" >"$FIXTURES/page1.json"
  printf '{"data":{"repository":{"issues":{"nodes":%s}}}}' "${2:-[]}" >"$FIXTURES/page2.json"
}

issue() {
  local n="$1" title="$2" horizon="$3" priority="$4"
  jq -nc --argjson n "$n" --arg t "$title" --arg h "$horizon" --arg p "$priority" '{
    number: $n, title: $t,
    labels: {nodes: (if $h == "" then [] else [{name: $h}] end)},
    projectItems: {nodes: (if $p == "" then [] else
      [{project: {number: 2}, priority: {name: $p}}] end)}
  }'
}

section_of() { printf '%s\n' "$1" | sed -n '/^NEXT UP/,/^IN FLIGHT/p'; }

echo '[]' >"$FIXTURES/claimed.json"

# ── The order: horizon, then Priority, then oldest first ────────────────────
# The fixture arrives shuffled, so a script that kept GitHub's order, or
# sorted on one key alone, fails the line-by-line comparison below.

open_issues "[$(issue 60 "Later work" horizon:later P0),
  $(issue 41 "Now, no priority" horizon:now ""),
  $(issue 52 "Now, P2" horizon:now P2),
  $(issue 33 "Next, P1" horizon:next P1),
  $(issue 51 "Now, P1, newer" horizon:now P1),
  $(issue 20 "Now, P1, older" horizon:now P1),
  $(issue 70 "No horizon" "" P0)]"
out="$("$script")"
order="$(section_of "$out" | grep -E '^- #' | sed -E 's/^- #([0-9]+).*/\1/' | tr '\n' ' ')"
check "$order" "orders by horizon, then Priority, then oldest" "20 51 52 41 33 60 70 "
check "$(section_of "$out")" "says the horizon and Priority under each" "now, P1"
check "$(section_of "$out")" "says when an issue has neither" "no horizon, P0"
check "$(section_of "$out")" "says when an issue is off the board" "now, no priority"
refute "$(cat "$FIXTURES/query.txt")" "filters on no label" 'labels:'
check "$(cat "$FIXTURES/query.txt")" "reads every page" "--paginate"
check "$(cat "$FIXTURES/query.txt")" "starts each page after the last" 'after: $endCursor,'
check "$(cat "$FIXTURES/query.txt")" "asks where the next page starts" 'pageInfo { hasNextPage endCursor }'
check "$(cat "$FIXTURES/query.txt")" "asks only for open issues" "states: OPEN"

# ── A Priority on another project does not count ────────────────────────────

open_issues "[{\"number\":80,\"title\":\"Other board\",
  \"labels\":{\"nodes\":[{\"name\":\"horizon:now\"}]},
  \"projectItems\":{\"nodes\":[{\"project\":{\"number\":9},\"priority\":{\"name\":\"P0\"}}]}}]"
out="$("$script")"
check "$(section_of "$out")" "reads Priority from intrada's board only" "now, no priority"

# ── An epic stays out; its issues stand on their own ────────────────────────

open_issues "[{\"number\":90,\"title\":\"A whole epic\",
  \"labels\":{\"nodes\":[{\"name\":\"epic\"},{\"name\":\"horizon:now\"}]},
  \"projectItems\":{\"nodes\":[]}}, $(issue 91 "One of its issues" horizon:now P1)]"
out="$("$script")"
refute "$(section_of "$out")" "never offers an epic" "- #90"
check "$(section_of "$out")" "keeps one of its issues" "- #91 One of its issues"

# ── A claimed issue leaves the list ───────────────────────────────────────

cat >"$FIXTURES/claimed.json" <<'JSON'
[{"number":41,"title":"Being built","updatedAt":"2026-11-11T10:00:00Z"}]
JSON
open_issues "[$(issue 41 "Being built" horizon:now P0), $(issue 42 "Waiting" horizon:now P1)]"
out="$("$script")"
check "$(section_of "$out")" "keeps the unclaimed one" "- #42 Waiting"
refute "$(section_of "$out")" "drops the claimed one" "- #41"
echo '[]' >"$FIXTURES/claimed.json"

# ── Nothing open ────────────────────────────────────────────────────────────

open_issues '[]'
out="$("$script")"
check "$(section_of "$out")" "an empty queue says nothing, not blank" "    nothing"

# ── A later page ranks with the first ───────────────────────────────────────

open_issues "[$(issue 10 "Old, later" horizon:later "")]" "[$(issue 300 "New, now" horizon:now P0)]"
out="$("$script")"
order="$(section_of "$out" | grep -E '^- #' | sed -E 's/^- #([0-9]+).*/\1/' | tr '\n' ' ')"
check "$order" "ranks an issue from the second page first" "300 10 "

# ── More open issues than are shown ────────────────────────────────────────

open_issues "$(jq -nc '[range(12) | {number: ., title: "Open", labels: {nodes: []}, projectItems: {nodes: []}}]')" \
  "$(jq -nc '[range(12; 25) | {number: ., title: "Open", labels: {nodes: []}, projectItems: {nodes: []}}]')"
out="$("$script")"
check "$(section_of "$out")" "says how many are hidden" "showing the top 10 of 25"
check "$(section_of "$out" | grep -cE '^- #')" "prints only the top ten" "10"

# ── GitHub refusing the read ────────────────────────────────────────────────

open_issues "[$(issue 42 "Waiting" horizon:now P1)]"
out="$(FAIL_NEXT=1 "$script")"
check "$(section_of "$out")" "names the outage rather than showing an empty queue" "GitHub did not answer"
check "$(section_of "$out")" "carries GitHub's own words" "required scopes"
check "$(section_of "$out")" "prints the reason on its own line" "    Your token has not been granted the required scopes."
refute "$(section_of "$out")" "never prints the raw JSON body" "{"
check "$out" "keeps the rest of the report" "RECENTLY LANDED"

printf '\n%s passed, %s failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
