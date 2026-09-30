#!/usr/bin/env bash
# Creates a worktree's test simulator and prints its UDID. It clones a template
# that has already booted once, so the first boot skips the data migration a
# blank device runs: ready in 7 s against 19 s, measured in #1904. Any failure
# on that path falls back to a blank device, so a run never fails because of
# the template.
#
# Usage: bash scripts/ios-sim-device.sh <device-name>
#
# The template's name derives from the pin, so a pin bump prepares a new one
# and deletes the old. `simctl clone` needs its source shut down, so the
# template is only booted while it is prepared, under a lock of its own so two
# worktrees never prepare it at once. IOS_SIM_TEMPLATE_LOCK_DIR and
# IOS_SIM_TEMPLATE_LOCK_TIMEOUT override the lock path and wait, for tests.
set -euo pipefail

name="${1:?usage: ios-sim-device.sh <device-name>}"
device="iPhone 16"
runtime="iOS26.5"
template_prefix="intrada-template-"
template="${template_prefix}$(printf '%s-%s' "$device" "$runtime" | tr -c 'A-Za-z0-9_-' '-')"

IOS_SIM_LOCK_DIR="${IOS_SIM_TEMPLATE_LOCK_DIR:-/tmp/intrada-ios-sim-template.lock}"
IOS_SIM_LOCK_TIMEOUT="${IOS_SIM_TEMPLATE_LOCK_TIMEOUT:-300}"
# shellcheck source=ios-sim-lock.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/ios-sim-lock.sh"

devices_named() {
    xcrun simctl list devices --json | python3 -c '
import json, sys
mode, want = sys.argv[1], sys.argv[2]
for group in json.load(sys.stdin)["devices"].values():
    for d in group:
        if mode == "is":
            match = d["name"] == want
        else:
            match = d["name"].startswith(want) and d["name"] != sys.argv[3]
        if match:
            print(d["udid"])
' "$@"
}

delete_stale_templates() {
    local udid
    for udid in $(devices_named prefix "$template_prefix" "$template"); do
        xcrun simctl shutdown "$udid" >/dev/null 2>&1 || true
        xcrun simctl delete "$udid" >&2 || return 1
    done
}

ready_template() {
    local udid
    udid="$(devices_named is "$template" | head -1)" || return 1
    if [ -n "$udid" ]; then
        xcrun simctl shutdown "$udid" >/dev/null 2>&1 || true
        echo "$udid"
        return 0
    fi
    delete_stale_templates || return 1
    echo "… preparing the simulator template $template, once per pin (about 20 s)" >&2
    # Named as unfinished until it has booted and shut down, so a prepare that
    # dies half way leaves a device the next prepare deletes, not a template
    # whose clones still migrate.
    udid="$(xcrun simctl create "$template-preparing" "$device" "$runtime")" || return 1
    xcrun simctl bootstatus "$udid" -b >/dev/null || return 1
    xcrun simctl shutdown "$udid" >&2 || return 1
    xcrun simctl rename "$udid" "$template" >&2 || return 1
    echo "$udid"
}

# Runs in the command substitution's subshell, so the trap releases the lock
# when that subshell ends, however it ends.
clone_from_template() {
    local template_udid
    ios_sim_lock_acquire || return 1
    trap ios_sim_lock_release EXIT
    template_udid="$(ready_template)" && xcrun simctl clone "$template_udid" "$name"
}

if udid="$(clone_from_template)" && [ -n "$udid" ]; then
    echo "$udid"
else
    echo "⚠ could not clone the simulator template; creating a blank $name instead" >&2
    xcrun simctl create "$name" "$device" "$runtime"
fi
