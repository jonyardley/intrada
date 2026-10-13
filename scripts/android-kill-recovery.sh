#!/usr/bin/env bash
# The kill half of the recovery check (#2576): KilledSessionRecoveryTest answers
# the item-complete sheet in one instrumentation, `am force-stop` kills the app,
# and a second instrumentation expects Resume with the answers in place.
# Gradle's connected run uninstalls the app after it, so this installs its own
# copy and uninstalls it again.
#
# Run from the repo root by just android-test-ui, after `source android/env.sh`
# and with the emulator lock held. Exits non-zero unless both steps pass.

set -euo pipefail

app=com.intrada.android
class=com.intrada.android.KilledSessionRecoveryTest
out=android/app/build/outputs/androidTest-results/kill-recovery
rm -rf "$out"
mkdir -p "$out"

android/gradlew -q -p android :app:installDebug :app:installDebugAndroidTest

instrumentations="$(adb shell pm list instrumentation | tr -d '\r' || true)"
runner="$(printf '%s\n' "$instrumentations" \
    | sed -n "s/^instrumentation:\([^ ]*\) (target=$app)$/\1/p" | sed -n 1p)"
if [ -z "$runner" ]; then
    echo "✗ no instrumentation targets $app, so the kill check cannot run" >&2
    adb uninstall "$app" >/dev/null || true
    exit 1
fi
trap 'adb uninstall "${runner%%/*}" >/dev/null || true; adb uninstall "$app" >/dev/null || true' EXIT

status=0

# A step passes only on exactly one test reported OK and none failed, skipped or
# errored: `am instrument` exits 0 whatever the tests did.
step() {
    local name="$1" method="$2" log="$out/$1.txt"
    adb shell am instrument -w -r -e killRecovery "$name" -e class "$class#$method" "$runner" \
        | tr -d '\r' >"$log" || true
    if [ "$(grep -c '^INSTRUMENTATION_STATUS_CODE: 0$' "$log" || true)" -eq 1 ] \
        && ! grep -q '^INSTRUMENTATION_STATUS_CODE: -' "$log" \
        && grep -q '^INSTRUMENTATION_CODE: -1$' "$log"; then
        echo "✓ kill recovery: $name"
    else
        echo "✗ kill recovery: $name failed ($log)" >&2
        cat "$log" >&2
        status=1
    fi
}

step answer answerTheSheetBeforeTheKill
adb shell am force-stop "$app"
[ "$status" -eq 0 ] && step resume resumeWithTheAnswersAfterTheKill
exit "$status"
