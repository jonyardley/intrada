#!/usr/bin/env bash
# Creates a worktree's test simulator and prints its UDID. It clones a template
# that has already booted once, so the first boot skips the data migration a
# blank device runs: ready in 8 s against 19 s, measured in #1904. Any failure
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

templates() {
    xcrun simctl list devices --json | python3 -c '
import json, sys
mode, prefix, template = sys.argv[1:4]
for group in json.load(sys.stdin)["devices"].values():
    for d in group:
        ready = d["name"] == template and d.get("isAvailable", True)
        stale = d["name"].startswith(prefix) and not ready
        if (ready, stale)[mode == "stale"]:
            print(d["udid"])
' "$1" "$template_prefix" "$template"
}

delete_stale_templates() {
    local udid
    for udid in $(templates stale); do
        xcrun simctl shutdown "$udid" >/dev/null 2>&1 || true
        xcrun simctl delete "$udid" >&2 || return 1
    done
}

ready_template() {
    local udid
    udid="$(templates ready | head -1)" || return 1
    if [ -n "$udid" ]; then
        xcrun simctl shutdown "$udid" >/dev/null 2>&1 || true
        echo "$udid"
        return 0
    fi
    delete_stale_templates || return 1
    echo "… preparing the simulator template $template, once per pin (about 30 s)" >&2
    # Unfinished until renamed, so a prepare killed mid-boot leaves a device the
    # next prepare deletes, never a template whose clones still migrate (#1904).
    udid="$(xcrun simctl create "$template-preparing" "$device" "$runtime")" || return 1
    if ! xcrun simctl bootstatus "$udid" -b >/dev/null; then
        xcrun simctl shutdown "$udid" >/dev/null 2>&1 || true
        xcrun simctl delete "$udid" >/dev/null 2>&1 || true
        return 1
    fi
    xcrun simctl shutdown "$udid" >&2 || return 1
    xcrun simctl rename "$udid" "$template" >&2 || return 1
    echo "$udid"
}

# Runs in the command substitution's subshell, so the trap releases the lock
# when that subshell ends.
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
