#!/usr/bin/env bash
# ios-simulator-smoke.sh — Boot a simulator, install the .app, launch it,
# and verify it doesn't crash on startup. Designed for CI (GitHub Actions
# macos-latest runners with Xcode pre-installed).
#
# Usage:
#   scripts/ios-simulator-smoke.sh path/to/Intrada.app
#
# Exit codes:
#   0 — app launched and stayed running for the verification window
#   1 — app crashed, failed to install, or sim failed to boot
#
# Environment:
#   IOS_SIM_DEVICE  — device type (default: "iPhone 16")
#   IOS_SIM_RUNTIME — runtime (default: auto-detect latest installed)
#   SMOKE_WAIT_SECS — seconds to wait before checking app is alive (default: 8)

set -euo pipefail

APP_PATH="${1:?Usage: $0 path/to/App.app}"
DEVICE_TYPE="${IOS_SIM_DEVICE:-iPhone 16}"
WAIT_SECS="${SMOKE_WAIT_SECS:-8}"
BUNDLE_ID="com.intrada.app"

# ─── Helpers ────────────────────────────────────────────────────────────

log() { echo "▸ $*"; }
die() { echo "✗ $*" >&2; exit 1; }

cleanup() {
  if [[ -n "${SIM_UDID:-}" ]]; then
    log "Shutting down simulator $SIM_UDID"
    xcrun simctl shutdown "$SIM_UDID" 2>/dev/null || true
    xcrun simctl delete "$SIM_UDID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

# ─── Resolve runtime ───────────────────────────────────────────────────

if [[ -n "${IOS_SIM_RUNTIME:-}" ]]; then
  RUNTIME="$IOS_SIM_RUNTIME"
else
  RUNTIME=$(xcrun simctl list runtimes -j \
    | python3 -c "
import json, sys
runtimes = json.load(sys.stdin)['runtimes']
ios = [r for r in runtimes if r['platform'] == 'iOS' and r['isAvailable']]
if not ios:
    sys.exit('No available iOS runtimes')
print(sorted(ios, key=lambda r: r['version'], reverse=True)[0]['identifier'])
")
  log "Auto-detected runtime: $RUNTIME"
fi

# ─── Create & boot simulator ──────────────────────────────────────────

log "Creating simulator ($DEVICE_TYPE, $RUNTIME)"
SIM_UDID=$(xcrun simctl create "intrada-ci-smoke" "$DEVICE_TYPE" "$RUNTIME")
log "Simulator UDID: $SIM_UDID"

log "Booting simulator"
xcrun simctl boot "$SIM_UDID"

# Wait for the runtime to be fully ready
xcrun simctl bootstatus "$SIM_UDID" -b

log "Simulator booted"

# ─── Install & launch ─────────────────────────────────────────────────

log "Installing $APP_PATH"
xcrun simctl install "$SIM_UDID" "$APP_PATH"

log "Launching $BUNDLE_ID"
xcrun simctl launch "$SIM_UDID" "$BUNDLE_ID"

# ─── Verify app stays alive ───────────────────────────────────────────

log "Waiting ${WAIT_SECS}s for crash detection..."
sleep "$WAIT_SECS"

# Check if the app process is still running in the simulator
APP_PID=$(xcrun simctl spawn "$SIM_UDID" launchctl list \
  | grep "$BUNDLE_ID" || true)

if [[ -z "$APP_PID" ]]; then
  log "App not found in process list — checking for crash log"

  # Pull crash logs from the simulator
  CRASH_LOG=$(xcrun simctl spawn "$SIM_UDID" log show \
    --predicate "process == 'Intrada' AND eventType == 'logEvent'" \
    --style compact --last "${WAIT_SECS}s" 2>/dev/null | tail -20 || true)

  if [[ -n "$CRASH_LOG" ]]; then
    echo "─── Crash/Log output ───"
    echo "$CRASH_LOG"
    echo "────────────────────────"
  fi

  die "App crashed or failed to stay running after ${WAIT_SECS}s"
fi

log "App is running (process found in launchctl list)"

# ─── Deep link test ───────────────────────────────────────────────────

log "Testing deep link: intrada://library"
xcrun simctl openurl "$SIM_UDID" "intrada://library" 2>/dev/null || \
  log "Warning: deep link open returned non-zero (may not be registered yet)"

sleep 2

# Verify app is still alive after deep link
APP_PID_POST=$(xcrun simctl spawn "$SIM_UDID" launchctl list \
  | grep "$BUNDLE_ID" || true)

if [[ -z "$APP_PID_POST" ]]; then
  die "App crashed after deep link navigation"
fi

log "Deep link handled without crash"

# ─── Screenshot (artifact for debugging) ──────────────────────────────

SCREENSHOT_DIR="${GITHUB_WORKSPACE:-$(pwd)}/ios-screenshots"
mkdir -p "$SCREENSHOT_DIR"

xcrun simctl io "$SIM_UDID" screenshot "$SCREENSHOT_DIR/launch.png" 2>/dev/null || \
  log "Warning: screenshot capture failed (non-fatal)"

if [[ -f "$SCREENSHOT_DIR/launch.png" ]]; then
  log "Screenshot saved: $SCREENSHOT_DIR/launch.png"
fi

# ─── Done ─────────────────────────────────────────────────────────────

log "iOS simulator smoke test passed ✓"
