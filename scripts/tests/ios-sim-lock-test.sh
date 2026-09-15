#!/usr/bin/env bash
# Self-test for scripts/ios-sim-lock.sh and scripts/ios-sim-idle-shutdown.sh: a
# live, a crashed and a mid-acquire lock holder, and the idle shutdown holding
# off while a run holds the lock or has run since (#1885). A stub `xcrun`
# records each shutdown with its time, so no real simulator is touched.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
timer="$root/scripts/ios-sim-idle-shutdown.sh"
udid=ios-sim-lock-test-not-a-device

tmp="$(mktemp -d)"
live_pids=""
trap '{ [ -z "$live_pids" ] || kill $live_pids; wait; } 2>/dev/null; rm -rf "$tmp"' EXIT

mkdir "$tmp/bin"
cat >"$tmp/bin/xcrun" <<'EOF'
#!/usr/bin/env bash
echo "$(date +%s) $*" >>"$XCRUN_LOG"
EOF
chmod +x "$tmp/bin/xcrun"
export PATH="$tmp/bin:$PATH"

export IOS_SIM_LOCK_DIR="$tmp/lock"
export IOS_SIM_LAST_RUN_MARKER="$tmp/marker"
export IOS_SIM_LOCK_TIMEOUT=0
export IOS_SIM_LOCK_POLL=1
# shellcheck source=../ios-sim-lock.sh
source "$root/scripts/ios-sim-lock.sh"

true &
dead_pid=$!
wait "$dead_pid"
sleep 60 &
live_holder=$!
live_pids="$live_holder"

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
  rm -rf "$1"
  mkdir "$1"
  [ -z "$2" ] || echo "$2" >"$1/pid"
}

shutdown_time() {
  awk '/simctl shutdown/ { print $1; exit }' "$1" 2>/dev/null
}

wait_for_exit() {
  local waited=0
  while kill -0 "$1" 2>/dev/null && [ "$waited" -lt 20 ]; do
    sleep 1
    waited=$((waited + 1))
  done
}

# ── The lock ──

lock_with_pid "$IOS_SIM_LOCK_DIR" "$live_holder"
expect "a live holder is not stale" false ios_sim_lock_stale
expect "a live holder makes the next run wait, not take it" false ios_sim_lock_acquire
expect "a live holder keeps its lock after a refused acquire" true \
  test "$(cat "$IOS_SIM_LOCK_DIR/pid")" = "$live_holder"

lock_with_pid "$IOS_SIM_LOCK_DIR" "$dead_pid"
expect "a crashed holder is stale" true ios_sim_lock_stale
expect "the next run clears a crashed holder's lock and takes it" true ios_sim_lock_acquire
expect "the run that cleared a crashed holder now holds the lock" true \
  test "$(cat "$IOS_SIM_LOCK_DIR/pid")" = "$$"

lock_with_pid "$IOS_SIM_LOCK_DIR" ""
expect "a holder that has not written its pid yet is not stale" false ios_sim_lock_stale
rm -rf "$IOS_SIM_LOCK_DIR"

expect "a run marks its worktree's sim used while it queues" true \
  bash -c 'rm -f "$1" && . "$2" && ios_sim_lock_acquire_for_run && test -s "$1"' _ \
  "$IOS_SIM_LAST_RUN_MARKER" "$root/scripts/ios-sim-lock.sh"
rm -rf "$IOS_SIM_LOCK_DIR"

# ── The idle shutdown, four timers side by side ──

old=$(($(date +%s) - 100))
for s in idle held reran released; do mkdir "$tmp/$s"; done

echo "$old" >"$tmp/idle/marker"
XCRUN_LOG="$tmp/idle/xcrun" IOS_SIM_LOCK_DIR="$tmp/idle/lock" IOS_SIM_LOCK_TIMEOUT=30 \
  IOS_SIM_IDLE_SHUTDOWN_SECONDS=3 bash "$timer" "$udid" "$tmp/idle/marker" &
idle_timer=$!

echo "$old" >"$tmp/held/marker"
lock_with_pid "$tmp/held/lock" "$live_holder"
XCRUN_LOG="$tmp/held/xcrun" IOS_SIM_LOCK_DIR="$tmp/held/lock" IOS_SIM_LOCK_TIMEOUT=30 \
  IOS_SIM_IDLE_SHUTDOWN_SECONDS=3 bash "$timer" "$udid" "$tmp/held/marker" &
held_timer=$!

date +%s >"$tmp/reran/marker"
XCRUN_LOG="$tmp/reran/xcrun" IOS_SIM_LOCK_DIR="$tmp/reran/lock" IOS_SIM_LOCK_TIMEOUT=30 \
  IOS_SIM_IDLE_SHUTDOWN_SECONDS=3 bash "$timer" "$udid" "$tmp/reran/marker" &
reran_timer=$!

export XCRUN_LOG="$tmp/released/xcrun" IOS_SIM_LOCK_DIR="$tmp/released/lock"
export IOS_SIM_LAST_RUN_MARKER="$tmp/released/marker" IOS_SIM_IDLE_SHUTDOWN_SECONDS=1
ios_sim_lock_acquire
echo "$old" >"$IOS_SIM_LAST_RUN_MARKER"
ios_sim_lock_release_with_idle_shutdown "$udid"
released_timer=$!
released_marker="$(cat "$IOS_SIM_LAST_RUN_MARKER")"
expect "a released run frees the lock at once" false test -d "$IOS_SIM_LOCK_DIR"
expect "a released run marks its sim used" true test "$released_marker" -gt "$old"

live_pids="$live_pids $idle_timer $held_timer $reran_timer $released_timer"

sleep 1
rerun_at=$(date +%s)
echo "$rerun_at" >"$tmp/reran/marker"
expect "a sleeping idle shutdown does not hold the lock" true \
  env IOS_SIM_LOCK_DIR="$tmp/reran/lock" IOS_SIM_LOCK_TIMEOUT=2 \
  bash -c '. "$1" && ios_sim_lock_acquire && ios_sim_lock_release' _ "$root/scripts/ios-sim-lock.sh"

sleep 1
released_at=$(date +%s)
rm -rf "$tmp/held/lock"

for t in $idle_timer $held_timer $reran_timer $released_timer; do wait_for_exit "$t"; done

expect "an idle sim with the lock free shuts down" true test -n "$(shutdown_time "$tmp/idle/xcrun")"
expect "the idle shutdown releases the lock it took" false test -d "$tmp/idle/lock"
expect "an idle sim waits while another run holds the lock" true \
  test "$(shutdown_time "$tmp/held/xcrun")" -ge "$released_at"
expect "a run since the timer started restarts the idle clock" true \
  test "$(shutdown_time "$tmp/reran/xcrun")" -ge $((rerun_at + 3))
expect "a released run's sim shuts down once idle" true \
  test "$(shutdown_time "$tmp/released/xcrun")" -ge $((released_marker + 1))

echo "ios-sim-lock-test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
