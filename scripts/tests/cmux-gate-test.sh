#!/usr/bin/env bash
# Self-test for scripts/lib/cmux-gate.sh. A fake cmux logs its arguments, so
# the test asserts what a gate would tell the sidebar without a running cmux.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
lib="$root/scripts/lib/cmux-gate.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

log="$tmp/cmux.log"
fake="$tmp/cmux"
printf '#!/usr/bin/env bash\necho "$*" >>"%s"\nexit "${FAKE_CMUX_STATUS:-0}"\n' "$log" >"$fake"
chmod +x "$fake"

pass=0
fail=0

expect() {
  local desc="$1" needle="$2"
  if grep -qF -- "$needle" "$log" 2>/dev/null; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "FAIL: $desc (expected to find: $needle)" >&2
  fi
}

expect_silent() {
  local desc="$1"
  if [ -s "$log" ]; then
    fail=$((fail + 1))
    echo "FAIL: $desc (cmux was called: $(cat "$log"))" >&2
  else
    pass=$((pass + 1))
  fi
}

run_gate() {
  local gate_status="$1"
  rm -f "$log"
  (
    source "$lib"
    cmux_gate_start "just check"
    trap 'cmux_gate_finish $?' EXIT
    cmux_gate_step 0.3 "lint"
    exit "$gate_status"
  ) || true
}

# ── A passing gate in cmux ──
export CMUX_GATE_BIN="$fake" CMUX_WORKSPACE_ID=workspace:1
run_gate 0
expect "step sets progress with the stage" "set-progress 0.3 --label just check: lint"
expect "finish clears the bar" "clear-progress"
expect "a pass is reported" "notify --title ✓ just check passed"
expect "the elapsed time is reported" "--body 0m 0s in "

# ── A failing gate keeps its own exit status ──
rm -f "$log"
status=0
(
  source "$lib"
  cmux_gate_start "just check"
  trap 'cmux_gate_finish $?' EXIT
  exit 3
) || status=$?
expect "a failure is reported" "notify --title ✗ just check failed"
if [ "$status" -eq 3 ]; then pass=$((pass + 1)); else fail=$((fail + 1)); echo "FAIL: gate exit status became $status, not 3" >&2; fi

# ── A cmux that errors never fails the gate ──
rm -f "$log"
status=0
FAKE_CMUX_STATUS=1 bash -c "set -euo pipefail; source '$lib'; cmux_gate_start g; cmux_gate_step 0.5 s; cmux_gate_finish 0" || status=$?
if [ "$status" -eq 0 ]; then pass=$((pass + 1)); else fail=$((fail + 1)); echo "FAIL: a failing cmux failed the gate ($status)" >&2; fi

# ── Outside a cmux workspace nothing is called ──
unset CMUX_WORKSPACE_ID
run_gate 0
expect_silent "no workspace means no cmux calls"

echo "cmux-gate-test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
