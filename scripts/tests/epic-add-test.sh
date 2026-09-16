#!/usr/bin/env bash
# Self-test for scripts/epic-add.sh (#1968). A fake `gh` on PATH serves one
# fixture per issue and logs every mutation, so each refusal is proved to
# write nothing at all, not merely to print a message.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/epic-add.sh"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/bin" "$work/fixtures" "$work/calls"

cat >"$work/bin/gh" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail

case "$1 ${2:-}" in
  "repo view")
    echo "jonyardley/intrada"
    ;;
  "issue view")
    cat "$FIXTURES/issue-$3.json"
    ;;
  "api graphql")
    printf '%s' "$*" | tr '\n' ' ' >>"$CALLS/mutations.log"
    echo >>"$CALLS/mutations.log"
    echo '{"data":{"addSubIssue":{"subIssue":{"number":0}}}}'
    ;;
  *)
    echo "fake gh: unhandled invocation: $*" >&2
    exit 1
    ;;
esac
FAKE
chmod +x "$work/bin/gh"

export FIXTURES="$work/fixtures"
export CALLS="$work/calls"
export PATH="$work/bin:$PATH"

issue() {
  local number="$1" labels="$2" parent="$3"
  local parent_json="null"
  if [ -n "$parent" ]; then
    parent_json="{\"number\":$parent,\"title\":\"Epic $parent\"}"
  fi
  printf '{"id":"ID_%s","number":%s,"title":"Issue %s","labels":[%s],"parent":%s}\n' \
    "$number" "$number" "$number" "$labels" "$parent_json" >"$FIXTURES/issue-$number.json"
}

epic='{"name":"epic"}'
bug='{"name":"bug"}'

pass=0
fail=0

reset() {
  rm -rf "$CALLS" && mkdir -p "$CALLS"
  rm -f "$FIXTURES"/issue-*.json
  issue 100 "$epic" ""
  issue 200 "$epic" ""
  issue 1 "$bug" ""
  issue 2 "" ""
}

mutations() {
  if [ -f "$CALLS/mutations.log" ]; then wc -l <"$CALLS/mutations.log" | tr -d ' '; else echo 0; fi
}

expect_refusal() {
  local desc="$1" needle="$2"
  shift 2
  local out run_status
  set +e
  out="$("$script" "$@" 2>&1)"
  run_status=$?
  set -e
  if [ "$run_status" -eq 0 ]; then
    fail=$((fail + 1))
    printf '✗ %s: expected a refusal, script exited 0\n    output: %s\n' "$desc" "$out" >&2
  elif ! printf '%s' "$out" | grep -qF -- "$needle"; then
    fail=$((fail + 1))
    printf '✗ %s: refused, but the message is missing %s\n    output: %s\n' "$desc" "$needle" "$out" >&2
  elif [ "$(mutations)" != "0" ]; then
    fail=$((fail + 1))
    printf '✗ %s: refused, but still wrote %s sub-issue(s)\n' "$desc" "$(mutations)" >&2
  else
    pass=$((pass + 1))
  fi
}

equals() {
  local actual="$1" desc="$2" expected="$3"
  if [ "$actual" = "$expected" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf '✗ %s\n    expected %s, saw %s\n' "$desc" "$expected" "$actual" >&2
  fi
}

check() {
  local haystack="$1" desc="$2" needle="$3"
  if printf '%s' "$haystack" | grep -qF -- "$needle"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    printf '✗ %s\n    expected to find: %s\n    in: %s\n' "$desc" "$needle" "$haystack" >&2
  fi
}

# ── Free children attach, in the order given, without replacing a parent ───

reset
out="$("$script" 100 1 2)"
equals "$(mutations)" "writes one sub-issue per child" "2"
check "$(sed -n 1p "$CALLS/mutations.log")" "attaches the first child first" "child=ID_1"
check "$(sed -n 2p "$CALLS/mutations.log")" "attaches the second child second" "child=ID_2"
check "$(cat "$CALLS/mutations.log")" "attaches to the epic's node" "parent=ID_100"
check "$(cat "$CALLS/mutations.log")" "never replaces a parent on a plain add" "replace=false"
check "$out" "confirms each child" "✓ #2 under #100"

# ── A child already under this epic is left alone ───────────────────────────

reset
issue 1 "$bug" 100
out="$("$script" 100 1 2)"
equals "$(mutations)" "writes only the child that is not attached yet" "1"
check "$out" "says the child was already there" "#1 is already under #100"

reset
issue 1 "$bug" 100
out="$("$script" 100 1)"
equals "$(mutations)" "writes nothing when every child is already attached" "0"
check "$out" "says there was nothing to do" "nothing new to attach"

# ── A child under another epic is refused, and nothing is written ──────────

reset
issue 2 "" 200
expect_refusal "another epic's child" "already under #200 (Epic 200)" 100 1 2

# ── epic-move takes it, and says so to GitHub ───────────────────────────────

reset
issue 2 "" 200
out="$("$script" --move 100 2)"
equals "$(mutations)" "moves the child" "1"
check "$(cat "$CALLS/mutations.log")" "asks GitHub to replace the parent" "replace=true"

# ── The one-level rule and the label rule ───────────────────────────────────

reset
expect_refusal "a parent without the epic label" "#1 has no epic label" 1 2

reset
issue 100 "$epic" 200
expect_refusal "an epic nested under another" "#100 sits under #200" 100 1

reset
expect_refusal "an epic as a child" "#200 is an epic itself" 100 1 200

reset
expect_refusal "an issue as its own child" "cannot be its own sub-issue" 100 100

# ── Arguments ───────────────────────────────────────────────────────────────

reset
expect_refusal "no children" "Usage" 100
expect_refusal "a child that is not a number" "Usage" 100 1 "#2"

printf '\n%s passed, %s failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
