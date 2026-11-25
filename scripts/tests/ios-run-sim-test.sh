#!/usr/bin/env bash
# Self-test for the device choice in scripts/ios-run-sim.sh (#2747): the named
# device on the newest iOS, created when missing, never another iPhone such as
# the two-screen iPhone Duo. A stub `xcrun` keeps the devices in a JSON file, so
# no real simulator is touched. The script runs from a copy in a scratch tree,
# since it reads its built app relative to its own folder.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/tree/scripts" "$tmp/tree/ios/build/dd/Build/Products/Debug-iphonesimulator/Intrada.app" "$tmp/bin"
cp "$root/scripts/ios-run-sim.sh" "$root/scripts/ios-sim-lock.sh" "$tmp/tree/scripts/"
script="$tmp/tree/scripts/ios-run-sim.sh"

cat >"$tmp/bin/xcrun" <<'EOF'
#!/usr/bin/env python3
import json, os, sys

state = os.environ["SIM_STATE"]
with open(os.environ["XCRUN_LOG"], "a") as log:
    log.write(" ".join(sys.argv[2:]) + "\n")
cmd, args = sys.argv[2], sys.argv[3:]

def load(name):
    with open(os.path.join(state, name)) as f:
        return json.load(f)

if cmd == "list" and args[0] == "devices":
    print(json.dumps({"devices": load("devices.json")}))
elif cmd == "list" and args[0] == "runtimes":
    print(json.dumps({"runtimes": load("runtimes.json")}))
elif cmd == "create":
    if os.environ.get("STUB_FAIL_CREATE"):
        sys.exit("stub simctl: create refused")
    devices = load("devices.json")
    udid = "created-" + args[2].rsplit(".", 1)[-1]
    devices.setdefault(args[2], []).append({"name": args[0], "udid": udid})
    with open(os.path.join(state, "devices.json"), "w") as f:
        json.dump(devices, f)
    print(udid)
elif cmd == "io":
    open(args[-1], "w").close()
EOF
chmod +x "$tmp/bin/xcrun"
export PATH="$tmp/bin:$PATH"

export REUSE_BUILD=1
export IOS_SIM_LOCK_DIR="$tmp/lock"
export IOS_SIM_LOCK_TIMEOUT=0
export IOS_SIM_BOOT_TIMEOUT=5
unset SIM_DEVICE

pass=0
fail=0

expect() {
  local desc="$1" want="$2"
  shift 2
  local got=false
  if "$@" 2>/dev/null; then got=true; fi
  if [ "$got" = "$want" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "FAIL: $desc (expected $want, got $got)" >&2
  fi
}

# fresh <name> <devices json> <runtimes json>
fresh() {
  export SIM_STATE="$tmp/state-$1" XCRUN_LOG="$tmp/log-$1"
  mkdir "$SIM_STATE"
  printf '%s' "$2" >"$SIM_STATE/devices.json"
  printf '%s' "$3" >"$SIM_STATE/runtimes.json"
  : >"$XCRUN_LOG"
}

run() {
  bash "$script" "$tmp/shot-$1.png" >"$tmp/stdout" 2>"$tmp/stderr"
}

unset SELFHOSTED_IOS_RUNTIME

rt=com.apple.CoreSimulator.SimRuntime
i16='"supportedDeviceTypes": [{"name": "iPhone 16"}, {"name": "iPhone Duo"}]'
runtimes='[
  {"identifier": "'$rt'.iOS-18-9", "platform": "iOS", "version": "18.9", '"$i16"'},
  {"identifier": "'$rt'.watchOS-27-0", "platform": "watchOS", "version": "27.0"},
  {"identifier": "'$rt'.iOS-18-10", "platform": "iOS", "version": "18.10", '"$i16"'}
]'

# ── An iPhone 16 exists: the newest one wins, never the Duo ──

fresh named '{
  "'$rt'.iOS-17-5": [{"name": "iPhone 16", "udid": "sixteen-old"}],
  "'$rt'.iOS-18-2": [{"name": "iPhone 16", "udid": "sixteen-new"}, {"name": "iPhone Duo", "udid": "duo-old"}],
  "'$rt'.iOS-26-0": [{"name": "iPhone Duo", "udid": "duo-new"}]
}' "$runtimes"
expect "a run with an iPhone 16 succeeds" true run named
expect "the iPhone 16 on the newest iOS is launched" true grep -qx "launch sixteen-new com.intrada.native --seed-sample-data" "$XCRUN_LOG"
expect "the Duo is never launched" false grep -Eq "^(install|launch) duo-" "$XCRUN_LOG"
expect "nothing is created when an iPhone 16 exists" false grep -q "^create " "$XCRUN_LOG"

# ── Only a Duo exists: an iPhone 16 is made on the newest iOS ──

fresh duo-only '{
  "'$rt'.iOS-26-0": [{"name": "iPhone Duo", "udid": "duo-new"}]
}' "$runtimes"
expect "a run with only a Duo succeeds" true run duo-only
expect "an iPhone 16 is created on the newest iOS, not a watchOS" true \
  grep -qx "create iPhone 16 iPhone 16 $rt.iOS-18-10" "$XCRUN_LOG"
expect "the created device is launched" true grep -qx "launch created-iOS-18-10 com.intrada.native --seed-sample-data" "$XCRUN_LOG"
expect "the created device is screenshotted" true test -f "$tmp/shot-duo-only.png"
expect "the Duo is never launched" false grep -Eq "^(install|launch) duo-" "$XCRUN_LOG"

# ── Two runtimes share a version, one only for a newer iPhone (#2984) ──

split_runtimes='[
  {"identifier": "'$rt'.iOS-26-5", "platform": "iOS", "version": "26.5", '"$i16"'},
  {"identifier": "'$rt'.iOS-27-2", "platform": "iOS", "version": "27.2", '"$i16"'},
  {"identifier": "'$rt'.iOS-27-2-iPhone19-4", "platform": "iOS", "version": "27.2",
   "supportedDeviceTypes": [{"name": "iPhone 19"}]}
]'
fresh split '{}' "$split_runtimes"
expect "a run with a device-only runtime succeeds" true run split
expect "the iPhone 16 is made on the general runtime, not the device-only one" true \
  grep -qx "create iPhone 16 iPhone 16 $rt.iOS-27-2" "$XCRUN_LOG"

# ── The pinned CI runtime wins when it is installed ──

fresh pinned '{}' "$split_runtimes"
export SELFHOSTED_IOS_RUNTIME=26.5
expect "a run with a pinned runtime succeeds" true run pinned
expect "the iPhone 16 is made on the pinned runtime" true \
  grep -qx "create iPhone 16 iPhone 16 $rt.iOS-26-5" "$XCRUN_LOG"

fresh pin-missing '{}' "$split_runtimes"
export SELFHOSTED_IOS_RUNTIME=25.0
expect "a run whose pin is not installed succeeds" true run pin-missing
expect "a missing pin falls back to the newest that fits" true \
  grep -qx "create iPhone 16 iPhone 16 $rt.iOS-27-2" "$XCRUN_LOG"
unset SELFHOSTED_IOS_RUNTIME

# ── Nothing can be made: the run stops and says so ──

fresh no-runtime '{
  "'$rt'.iOS-26-0": [{"name": "iPhone Duo", "udid": "duo-new"}]
}' '[{"identifier": "'$rt'.watchOS-27-0", "platform": "watchOS", "version": "27.0"}]'
expect "a run with no iOS runtime fails" false run no-runtime
expect "no iOS runtime says none could be made" true grep -q "none could be made" "$tmp/stderr"
expect "no iOS runtime launches nothing" false grep -q "^launch " "$XCRUN_LOG"

fresh create-fails '{
  "'$rt'.iOS-26-0": [{"name": "iPhone Duo", "udid": "duo-new"}]
}' "$runtimes"
export STUB_FAIL_CREATE=1
expect "a run whose create fails fails" false run create-fails
unset STUB_FAIL_CREATE
expect "a failed create says none could be made" true grep -q "none could be made" "$tmp/stderr"
expect "a failed create shows simctl's own error" true grep -q "stub simctl: create refused" "$tmp/stderr"
expect "a failed create launches nothing" false grep -q "^launch " "$XCRUN_LOG"

echo "ios-run-sim-test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
