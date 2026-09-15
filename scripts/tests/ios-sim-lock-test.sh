#!/usr/bin/env bash
# Self-test for scripts/ios-sim-lock.sh: a live holder, a crashed holder and a
# holder still mid-acquire, as both the next run and the idle shutdown timer
# (#1885) see them.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

export IOS_SIM_LOCK_DIR="$tmp/lock"
export IOS_SIM_LOCK_TIMEOUT=0
# shellcheck source=../ios-sim-lock.sh
source "$root/scripts/ios-sim-lock.sh"

true &
dead_pid=$!
wait "$dead_pid"

pass=0
fail=0

expect() {
  local desc="$1" want="$2"
  shift 2
  local got=false
  if "$@" 2>/dev/null; then got=true; fi
  if [ "$got" = "$want" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "FAIL: $desc (expected $want, got $got)" >&2
  fi
}

lock_with_pid() {
  rm -rf "$IOS_SIM_LOCK_DIR"
  mkdir "$IOS_SIM_LOCK_DIR"
  [ -z "$1" ] || echo "$1" >"$IOS_SIM_LOCK_DIR/pid"
}

rm -rf "$IOS_SIM_LOCK_DIR"
expect "no lock is not held" false ios_sim_lock_held

lock_with_pid "$$"
expect "a live holder holds the lock" true ios_sim_lock_held
expect "a live holder is not stale" false ios_sim_lock_stale
expect "a live holder makes the next run wait, not take it" false ios_sim_lock_acquire
expect "a live holder keeps its lock after a refused acquire" true ios_sim_lock_held

lock_with_pid "$dead_pid"
expect "a crashed holder does not hold the lock" false ios_sim_lock_held
expect "a crashed holder is stale" true ios_sim_lock_stale
expect "the next run clears a crashed holder's lock and takes it" true ios_sim_lock_acquire
expect "the run that cleared a crashed holder now holds the lock" true ios_sim_lock_held

lock_with_pid ""
expect "a holder that has not written its pid yet holds the lock" true ios_sim_lock_held

echo "ios-sim-lock-test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
