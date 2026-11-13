#!/usr/bin/env bash
# Self-test for the READY block of scripts/generate-status.sh (#2800). A fake
# `gh` on PATH serves real-shaped payloads and honours --jq, so the script's
# own jq programs are what the assertions exercise: only labelled issues, the
# order by horizon then Priority then age, claimed ones dropped, epics left out, and the read
# failing.
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
    if [[ "$*" == *'labels: ["ready"]'* ]]; then
      if [ -n "${FAIL_READY:-}" ]; then
        printf '%s' '{"errors":[{"type":"INSUFFICIENT_SCOPES","message":"Your token has not been granted the required scopes."}]}'
        echo "gh: Your token has not been granted the required scopes." >&2
        exit 1
      fi
      printf '%s\n' "$*" >"$FIXTURES/query.txt"
      body="$(cat "$FIXTURES/ready.json")"
    fi
    ;;
  "issue list")
    [ -n "${FAIL_CLAIMED:-}" ] && { echo "gh: claimed read refused" >&2; exit 1; }
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

ready() {
  printf '{"data":{"repository":{"issues":{"nodes":%s}}}}' "$1" >"$FIXTURES/ready.json"
}

issue() {
  local n="$1" title="$2" horizon="$3" priority="$4"
  jq -nc --argjson n "$n" --arg t "$title" --arg h "$horizon" --arg p "$priority" '{
    number: $n, title: $t,
    labels: {nodes: ([{name: "ready"}] + (if $h == "" then [] else [{name: $h}] end))},
    projectItems: {nodes: (if $p == "" then [] else
      [{project: {number: 2}, priority: {name: $p}}] end)}
  }'
}

section_of() { printf '%s\n' "$1" | sed -n '/^READY/,/^IN FLIGHT/p'; }

echo '[]' >"$FIXTURES/claimed.json"

# ── The order: horizon, then Priority, then oldest first ────────────────────
# The fixture arrives shuffled, so a script that kept GitHub's order, or
# sorted on one key alone, fails the line-by-line comparison below.

ready "[$(issue 60 "Later work" horizon:later P0),
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
check "$(cat "$FIXTURES/query.txt")" "asks only for ready issues" 'labels: ["ready"]'
check "$(cat "$FIXTURES/query.txt")" "asks only for open issues" "states: OPEN"
json_order="$("$script" --ready-json | jq -r '[.[].number] | join(" ")')"
check "$json_order " "--ready-json, the crew loop's read, keeps the same order" "20 51 52 41 33 60 70 "

# ── A Priority on another project does not count ────────────────────────────

ready "[{\"number\":80,\"title\":\"Other board\",
  \"labels\":{\"nodes\":[{\"name\":\"ready\"},{\"name\":\"horizon:now\"}]},
  \"projectItems\":{\"nodes\":[{\"project\":{\"number\":9},\"priority\":{\"name\":\"P0\"}}]}}]"
out="$("$script")"
check "$(section_of "$out")" "reads Priority from intrada's board only" "now, no priority"

# ── An epic marked ready stays out; its issues stand on their own label ─────

ready "[{\"number\":90,\"title\":\"A whole epic\",
  \"labels\":{\"nodes\":[{\"name\":\"ready\"},{\"name\":\"epic\"},{\"name\":\"horizon:now\"}]},
  \"projectItems\":{\"nodes\":[]}}, $(issue 91 "One of its issues" horizon:now P1)]"
out="$("$script")"
refute "$(section_of "$out")" "never offers an epic" "- #90"
check "$(section_of "$out")" "keeps an issue carrying the label itself" "- #91 One of its issues"

# ── A claimed ready issue leaves the queue ──────────────────────────────────

cat >"$FIXTURES/claimed.json" <<'JSON'
[{"number":41,"title":"Being built","updatedAt":"2026-11-11T10:00:00Z"}]
JSON
ready "[$(issue 41 "Being built" horizon:now P0), $(issue 42 "Waiting" horizon:now P1)]"
out="$("$script")"
check "$(section_of "$out")" "keeps the unclaimed one" "- #42 Waiting"
refute "$(section_of "$out")" "drops the claimed one" "- #41"
check "$("$script" --ready-json | jq -r '[.[].number] | join(" ")')" "--ready-json drops the claimed one too" "42"
refute "$("$script" --ready-json | jq -r '[.[].number] | join(" ")')" "--ready-json never offers the claimed one" "41"
echo '[]' >"$FIXTURES/claimed.json"

# ── Nothing ready ───────────────────────────────────────────────────────────

ready '[]'
out="$("$script")"
check "$(section_of "$out")" "an empty queue says nothing, not blank" "    nothing"

# ── More ready issues than the read fetches ────────────────────────────────

ready "$(jq -nc '[range(50) | {number: ., title: "Ready", labels: {nodes: []}, projectItems: {nodes: []}}]')"
out="$("$script")"
check "$(section_of "$out")" "says when the cap bit" "showing the first 50; there are more"

# ── GitHub refusing the read ────────────────────────────────────────────────

ready "[$(issue 42 "Waiting" horizon:now P1)]"
out="$(FAIL_READY=1 "$script")"
check "$(section_of "$out")" "names the outage rather than showing an empty queue" "GitHub did not answer"
check "$(section_of "$out")" "carries GitHub's own words" "required scopes"
check "$(section_of "$out")" "prints the reason on its own line" "    Your token has not been granted the required scopes."
refute "$(section_of "$out")" "never prints the raw JSON body" "{"
check "$out" "keeps the rest of the report" "RECENTLY LANDED"
rc=0
FAIL_READY=1 "$script" --ready-json >/dev/null 2>&1 || rc=$?
check "$rc" "--ready-json exits 2 on an outage, never an empty queue" "2"
rc=0
FAIL_CLAIMED=1 "$script" --ready-json >/dev/null 2>&1 || rc=$?
check "$rc" "--ready-json exits 2 when the claims cannot be read, never offering a claimed issue" "2"

printf '\n%s passed, %s failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
