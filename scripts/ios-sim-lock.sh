#!/usr/bin/env bash
# Machine-wide advisory lock serialising iOS Simulator test runs (#1622).
# CoreSimulatorService and SpringBoard are machine-global, so two
# xcodebuild/XCTest runs against DISTINCT simulator devices can still crash
# each other's XCUITests (#1192, measured in #1621), so this lock is the sole
# busy signal now. It replaces check-sim-free.sh's "any booted simulator"
# heuristic, which false-positived on a leftover-but-idle sim nobody shut
# down after an earlier run: a booted device no longer means anything by
# itself, only holding this lock does.
#
# A second caller WAITS rather than refusing outright, since the wait is
# short relative to the cost of an aborted gate and a manual restart (#1622).
# A full tier holds the lock for about five minutes since #1480, so the
# default timeout leaves room for two queued runs. `mkdir` is the lock
# primitive (atomic on every filesystem this
# runs on) rather than `flock`, which macOS does not ship.
#
# Usage, from a test run that keeps its worktree's sim booted for the next:
#   source scripts/ios-sim-lock.sh
#   ios_sim_lock_acquire_for_run
#   trap 'ios_sim_lock_release_with_idle_shutdown "$udid"' EXIT
#
# IOS_SIM_LOCK_DIR, IOS_SIM_LOCK_TIMEOUT, IOS_SIM_LOCK_POLL and
# IOS_SIM_LAST_RUN_MARKER override the path, the wait and poll in seconds, and
# the marker, all for tests, so this can be exercised without a 1800s wait.

IOS_SIM_LOCK_DIR="${IOS_SIM_LOCK_DIR:-/tmp/intrada-ios-test.lock}"
IOS_SIM_LOCK_TIMEOUT="${IOS_SIM_LOCK_TIMEOUT:-1800}"
IOS_SIM_LOCK_POLL="${IOS_SIM_LOCK_POLL:-5}"
IOS_SIM_LAST_RUN_MARKER="${IOS_SIM_LAST_RUN_MARKER:-ios/build/.sim-last-run}"
IOS_SIM_SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# A crashed holder (kill -9, a yanked machine) never runs its EXIT trap, so a
# dead PID means the lock is stale, not held.
ios_sim_lock_stale() {
    [ -f "$IOS_SIM_LOCK_DIR/pid" ] \
        && ! kill -0 "$(cat "$IOS_SIM_LOCK_DIR/pid" 2>/dev/null || echo 0)" 2>/dev/null
}

ios_sim_lock_acquire() {
    local waited=0 announced=0 holder
    while ! mkdir "$IOS_SIM_LOCK_DIR" 2>/dev/null; do
        holder="$(cat "$IOS_SIM_LOCK_DIR/holder" 2>/dev/null || echo "another session")"
        if ios_sim_lock_stale; then
            echo "⚠ stale iOS simulator lock ($holder, holder process gone): clearing it" >&2
            rm -rf "$IOS_SIM_LOCK_DIR"
            continue
        fi
        if [ "$waited" -ge "$IOS_SIM_LOCK_TIMEOUT" ]; then
            echo "✗ timed out after ${IOS_SIM_LOCK_TIMEOUT}s waiting for the iOS simulator lock, held by $holder" >&2
            return 1
        fi
        if [ "$announced" = 0 ]; then
            echo "… waiting for the iOS simulator lock, held by $holder" >&2
            announced=1
        fi
        sleep "$IOS_SIM_LOCK_POLL"
        waited=$((waited + IOS_SIM_LOCK_POLL))
    done
    pwd > "$IOS_SIM_LOCK_DIR/holder"
    echo "$$" > "$IOS_SIM_LOCK_DIR/pid"
}

ios_sim_lock_release() {
    rm -rf "$IOS_SIM_LOCK_DIR"
}

ios_sim_mark_used() {
    mkdir -p "$(dirname "$IOS_SIM_LAST_RUN_MARKER")" && date +%s >"$IOS_SIM_LAST_RUN_MARKER"
}

# Marked on queueing too, so an idle shutdown that wins the lock while this run
# waits behind another worktree's does not take this worktree's sim down.
ios_sim_lock_acquire_for_run() {
    ios_sim_mark_used
    ios_sim_lock_acquire
}

# Keeps this worktree's sim booted for its next run (#1885); `worktree-rm` and
# `ios-test-sim-clean` stay the hard stops. Marked before the release, so the
# idle shutdown never reads the marker from before this run.
ios_sim_lock_release_with_idle_shutdown() {
    ios_sim_mark_used
    ios_sim_lock_release
    [ -n "$1" ] || return 0
    bash "$IOS_SIM_SCRIPTS_DIR/ios-sim-idle-shutdown.sh" "$1" "$IOS_SIM_LAST_RUN_MARKER" \
        </dev/null >/dev/null 2>&1 &
}
