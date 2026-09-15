#!/usr/bin/env bash
# Shuts a worktree's simulator down once it has sat idle (#1885): its last run
# ended at least IOS_SIM_IDLE_SHUTDOWN_SECONDS ago (default 600). It takes the
# machine-wide lock to read the marker and shut down, so no run can take the
# lock and reach its boot while the simulator is still going down.
#
# Usage: ios-sim-idle-shutdown.sh <udid> <last-run marker>
set -uo pipefail
# shellcheck source=ios-sim-lock.sh
source "$(dirname "${BASH_SOURCE[0]}")/ios-sim-lock.sh"

udid="$1"
marker="$2"
idle="${IOS_SIM_IDLE_SHUTDOWN_SECONDS:-600}"

while :; do
    ios_sim_lock_acquire 2>/dev/null || continue
    last="$(cat "$marker" 2>/dev/null || echo 0)"
    remaining=$((idle - ($(date +%s) - last)))
    if [ "$remaining" -le 0 ]; then
        xcrun simctl shutdown "$udid" >/dev/null 2>&1 || true
        ios_sim_lock_release
        exit 0
    fi
    ios_sim_lock_release
    sleep "$remaining"
done
