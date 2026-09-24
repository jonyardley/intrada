#!/usr/bin/env bash
# Sidebar progress and a finished alert for the long local gates (#2088).
# Every call never fails its caller: a gate's verdict is its own exit status,
# not whether the sidebar answered.

cmux_gate_active() {
    [ -n "${CMUX_WORKSPACE_ID:-}" ] && command -v "${CMUX_GATE_BIN:-cmux}" >/dev/null 2>&1
}

cmux_gate_start() {
    CMUX_GATE_NAME="$1"
    CMUX_GATE_STARTED=$SECONDS
}

cmux_gate_step() {
    cmux_gate_active || return 0
    "${CMUX_GATE_BIN:-cmux}" set-progress "$1" --label "$CMUX_GATE_NAME: $2" >/dev/null 2>&1 || true
}

cmux_gate_finish() {
    local status="$1" elapsed title
    cmux_gate_active || return 0
    "${CMUX_GATE_BIN:-cmux}" clear-progress >/dev/null 2>&1 || true
    elapsed=$((SECONDS - ${CMUX_GATE_STARTED:-$SECONDS}))
    if [ "$status" -eq 0 ]; then
        title="✓ $CMUX_GATE_NAME passed"
    else
        title="✗ $CMUX_GATE_NAME failed"
    fi
    "${CMUX_GATE_BIN:-cmux}" notify --title "$title" \
        --body "$((elapsed / 60))m $((elapsed % 60))s in $(basename "$PWD")" >/dev/null 2>&1 || true
}
