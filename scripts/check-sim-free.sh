#!/usr/bin/env bash
# Preflight: detect whether another agent's iOS Simulator or test run is
# already active on this machine. The Simulator app and CoreSimulatorService
# are machine-global (see CLAUDE.md "Simulator safety"), so two agents
# running iOS tests in parallel worktrees can crash each other's XCUITests
# (#1192).
#
# Usage:
#   bash scripts/check-sim-free.sh              # exit 0 if clear, non-zero if busy
#   source scripts/check-sim-free.sh; check_sim_free   # call as a function
#
# SIMULATE_BUSY=1 forces the busy path, so the error handling can be tested
# without a real booted simulator or xcodebuild run.

check_sim_free() {
    if [ "${SIMULATE_BUSY:-}" = "1" ]; then
        echo "✗ (SIMULATE_BUSY=1) simulating another agent's simulator/tests as active." >&2
        return 1
    fi

    local booted
    booted="$(xcrun simctl list devices --json 2>/dev/null \
        | jq -r '.devices | to_entries[] | .value[] | select(.state == "Booted") | .name' 2>/dev/null \
        | grep -i . || true)"
    if [ -n "$booted" ]; then
        echo "✗ a booted iOS Simulator was found:" >&2
        echo "$booted" | sed 's/^/  /' >&2
        return 1
    fi

    local procs
    # Anchored to the executable: unanchored, this also matched
    # `npm exec xcodebuildmcp@2 mcp` and blocked the gate outright (#1529).
    procs="$(pgrep -fl '(^|/)(xcodebuild|XCTestAgent)( |$)' 2>/dev/null || true)"
    if [ -n "$procs" ]; then
        echo "✗ an xcodebuild/XCTestAgent process is already running:" >&2
        echo "$procs" | sed 's/^/  /' >&2
        return 1
    fi

    return 0
}

# Run automatically when executed directly; stay silent when sourced so a
# caller can use the `check_sim_free` function on its own terms.
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    check_sim_free
fi
