#!/usr/bin/env bash
# Self-test for the EPICS block of scripts/generate-status.sh (#1968). A fake
# `gh` on PATH serves real-shaped payloads and honours --jq, so the script's
# own jq programs are what the assertions exercise: the count, the next open
# child in the epic's order, claimed children, and the read failing.
set -euo pipefail

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
    if [ -n "${FAIL_EPICS:-}" ]; then
      echo "gh: Something went wrong while executing your query." >&2
      exit 1
    fi
    printf '%s\n' "$*" >"$FIXTURES/query.txt"
    body="$(cat "$FIXTURES/epics.json")"
    ;;
  "issue list") body="$(cat "$FIXTURES/claimed.json")" ;;
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
  if printf '%s' "$haystack" | grep -qF -- "$needle"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf '✗ %s\n    expected to find: %s\n' "$desc" "$needle" >&2
  fi
}

refute() {
  local haystack="$1" desc="$2" needle="$3"
  if printf '%s' "$haystack" | grep -qF -- "$needle"; then
    fail=$((fail + 1))
    printf '✗ %s\n    did not expect: %s\n' "$desc" "$needle" >&2
  else
    pass=$((pass + 1))
  fi
}

epics() {
  printf '{"data":{"repository":{"issues":{"nodes":%s}}}}' "$1" >"$FIXTURES/epics.json"
}

echo '[]' >"$FIXTURES/claimed.json"

# ── An epic part done: the count, the claimed child, the next one ───────────
# The children are out of numeric order on purpose: next follows the epic's
# own order, so a script that sorted by number would name #11 here.

cat >"$FIXTURES/claimed.json" <<'JSON'
[{"number":40,"title":"Being built","updatedAt":"2026-09-16T10:00:00Z"}]
JSON
epics '[{"number":7,"title":"Work the audit","subIssues":{"nodes":[
  {"number":30,"title":"Done already","state":"CLOSED"},
  {"number":40,"title":"Being built","state":"OPEN"},
  {"number":50,"title":"Pick me next","state":"OPEN"},
  {"number":11,"title":"After that","state":"OPEN"}]}}]'
out="$("$script")"
check "$out" "names the epic" "- #7 Work the audit"
check "$out" "counts closed children against all of them" "1 of 4 done"
check "$out" "names the claimed child" "in flight: #40"
check "$out" "names the first unclaimed open child in the epic's order" "next: #50 Pick me next"
refute "$out" "never offers a claimed child as next" "next: #40"
refute "$out" "never offers a closed child as next" "next: #30"
check "$(cat "$FIXTURES/query.txt")" "asks only for epics" 'labels: ["epic"]'
check "$(cat "$FIXTURES/query.txt")" "asks only for open epics" "states: OPEN"

# ── Every open child claimed ────────────────────────────────────────────────

epics '[{"number":7,"title":"Work the audit","subIssues":{"nodes":[
  {"number":30,"title":"Done already","state":"CLOSED"},
  {"number":40,"title":"Being built","state":"OPEN"}]}}]'
out="$("$script")"
check "$out" "says every open child is taken" "1 of 2 done, in flight: #40; every open one is in flight"
refute "$out" "offers nothing as next" "next:"
echo '[]' >"$FIXTURES/claimed.json"

# ── An epic with no children, and a finished one ────────────────────────────

epics '[{"number":7,"title":"Empty","subIssues":{"nodes":[]}},
  {"number":8,"title":"Finished","subIssues":{"nodes":[
    {"number":30,"title":"One","state":"CLOSED"},
    {"number":31,"title":"Two","state":"CLOSED"}]}}]'
out="$("$script")"
check "$out" "says an epic has no children rather than zero of zero" "no sub-issues yet"
check "$out" "says a finished epic is due to close" "all 2 done: close it"
refute "$out" "never prints zero of zero" "0 of 0 done"

# ── No epics at all ─────────────────────────────────────────────────────────

epics '[]'
out="$("$script")"
section="$(printf '%s\n' "$out" | sed -n '/^EPICS/,/^IN FLIGHT/p')"
check "$section" "an empty section says nothing, not blank" "    nothing"

# ── More epics than the read fetches ────────────────────────────────────────

epics "$(jq -nc '[range(30) | {number: ., title: "Epic", subIssues: {nodes: []}}]')"
out="$("$script")"
section="$(printf '%s\n' "$out" | sed -n '/^EPICS/,/^IN FLIGHT/p')"
check "$section" "says when the cap bit" "showing the first 30; there are more"

# ── GitHub refusing the read ────────────────────────────────────────────────

epics '[{"number":7,"title":"Work the audit","subIssues":{"nodes":[]}}]'
out="$(FAIL_EPICS=1 "$script")"
section="$(printf '%s\n' "$out" | sed -n '/^EPICS/,/^IN FLIGHT/p')"
check "$section" "names the outage rather than showing an empty section" "GitHub did not answer"
check "$section" "carries GitHub's own words" "Something went wrong"
check "$out" "keeps the rest of the report" "RECENTLY LANDED"

printf '\n%s passed, %s failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
