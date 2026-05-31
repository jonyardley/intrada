#!/usr/bin/env bash
# Build the native iOS app, launch it on a simulator, and screenshot.
# Assumes the Xcode project + generated packages already exist (the `ios-run`
# just recipe runs `_ios-sync` + xcodegen first). Reused by CI in A5.
#
# Usage: scripts/ios-run-sim.sh [screenshot-path]
set -euo pipefail

cd "$(dirname "$0")/../ios"

APP_ID="com.intrada.native"
DD="build/dd"
SHOT="${1:-/tmp/intrada-native.png}"
mkdir -p "$(dirname "$SHOT")"

# Newest available iPhone simulator.
UDID=$(xcrun simctl list devices available --json | python3 -c "
import json, sys
d = json.load(sys.stdin)['devices']
devs = [x for k in sorted(d) if 'iOS' in k for x in d[k] if 'iPhone' in x['name']]
print(devs[-1]['udid'] if devs else '')
")
if [ -z "$UDID" ]; then
    echo "✗ No iPhone simulator available (Xcode → Settings → Platforms → iOS)" >&2
    exit 1
fi

xcrun simctl boot "$UDID" 2>/dev/null || true

# REUSE_BUILD=1 reuses an existing build/dd .app (e.g. CI, right after the
# snapshot-test step already built it) — avoids a second xcodebuild. Local
# `just ios-run` leaves it unset and always builds fresh.
if [ -z "${REUSE_BUILD:-}" ]; then
    echo "building…"
    xcodebuild -project Intrada.xcodeproj -scheme Intrada -sdk iphonesimulator \
        -destination "id=$UDID" -derivedDataPath "$DD" -configuration Debug \
        build CODE_SIGNING_ALLOWED=NO >/tmp/ios-build.log 2>&1 || {
        echo "✗ build failed:" >&2
        grep -E "error:" /tmp/ios-build.log | tail -20 >&2
        exit 1
    }
fi

APP=$(find "$DD/Build/Products" -name "Intrada.app" -type d | head -1)
[ -n "$APP" ] || { echo "✗ no Intrada.app in $DD (build first)" >&2; exit 1; }
xcrun simctl install "$UDID" "$APP"
xcrun simctl launch "$UDID" "$APP_ID" >/dev/null
sleep 3
xcrun simctl io "$UDID" screenshot "$SHOT" >/dev/null
echo "✓ launched on $UDID — screenshot: $SHOT"
