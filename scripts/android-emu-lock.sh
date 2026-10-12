#!/usr/bin/env bash
# Machine-wide lock on the one intrada-api36 emulator, taken by just android-run
# and just android-test-ui (#2562): up to four Android sessions run side by side,
# and an install from one session under another's UI tests fails both. A second
# caller waits rather than refusing, and `mkdir` is the primitive because macOS
# ships no `flock`.
#
# Usage, after `source android/env.sh`:
#   source scripts/android-emu-lock.sh
#   android_emu_lock_acquire
#   trap android_emu_lock_release EXIT
#   android_emu_boot
#
# ANDROID_EMU_LOCK_DIR, ANDROID_EMU_LOCK_TIMEOUT and ANDROID_EMU_LOCK_POLL
# override the path, the wait and the poll in seconds.

ANDROID_EMU_LOCK_DIR="${ANDROID_EMU_LOCK_DIR:-/tmp/intrada-android-emu.lock}"
ANDROID_EMU_LOCK_TIMEOUT="${ANDROID_EMU_LOCK_TIMEOUT:-1800}"
ANDROID_EMU_LOCK_POLL="${ANDROID_EMU_LOCK_POLL:-5}"
ANDROID_EMU_AVD="${ANDROID_EMU_AVD:-intrada-api36}"

android_emu_lock_acquire() {
    local waited=0 announced=0 holder
    while ! mkdir "$ANDROID_EMU_LOCK_DIR" 2>/dev/null; do
        holder="$(cat "$ANDROID_EMU_LOCK_DIR/holder" 2>/dev/null || echo "another session")"
        # A killed holder never runs its EXIT trap, so a dead PID means the lock is stale.
        if [ -f "$ANDROID_EMU_LOCK_DIR/pid" ] \
            && ! kill -0 "$(cat "$ANDROID_EMU_LOCK_DIR/pid" 2>/dev/null || echo 0)" 2>/dev/null; then
            echo "⚠ stale Android emulator lock ($holder, holder process gone): clearing it" >&2
            rm -rf "$ANDROID_EMU_LOCK_DIR"
            continue
        fi
        if [ "$waited" -ge "$ANDROID_EMU_LOCK_TIMEOUT" ]; then
            echo "✗ timed out after ${ANDROID_EMU_LOCK_TIMEOUT}s waiting for the Android emulator lock, held by $holder" >&2
            return 1
        fi
        if [ "$announced" = 0 ]; then
            echo "… waiting for the Android emulator lock, held by $holder" >&2
            announced=1
        fi
        sleep "$ANDROID_EMU_LOCK_POLL"
        waited=$((waited + ANDROID_EMU_LOCK_POLL))
    done
    pwd > "$ANDROID_EMU_LOCK_DIR/holder"
    echo "$$" > "$ANDROID_EMU_LOCK_DIR/pid"
}

android_emu_lock_release() {
    rm -rf "$ANDROID_EMU_LOCK_DIR"
}

android_emu_boot() {
    local avd="$ANDROID_EMU_AVD" avdmanager
    if adb devices | grep -q '^emulator-.*device$'; then
        return 0
    fi
    if ! "$ANDROID_HOME/emulator/emulator" -list-avds | grep -qx "$avd"; then
        avdmanager="$(command -v avdmanager || echo "$ANDROID_HOME/cmdline-tools/latest/bin/avdmanager")"
        echo "↻ creating the $avd emulator"
        # A Homebrew avdmanager looks for system images beside itself unless told where the SDK is.
        echo no | AVDMANAGER_OPTS="-Dcom.android.sdkmanager.toolsdir=$ANDROID_HOME/cmdline-tools/latest" "$avdmanager" create avd -n "$avd" -k "system-images;android-36;google_apis;arm64-v8a" -d pixel_7
    fi
    echo "↻ booting $avd"
    nohup "$ANDROID_HOME/emulator/emulator" -avd "$avd" -no-snapshot-save -no-audio -no-boot-anim >/dev/null 2>&1 &
    adb wait-for-device
    until [ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = 1 ]; do sleep 2; done
}
