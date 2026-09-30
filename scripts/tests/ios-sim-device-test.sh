#!/usr/bin/env bash
# Self-test for scripts/ios-sim-device.sh (#1904): cloning from a template,
# preparing it once, falling back to a blank device on every failure, and two
# worktrees preparing at once. A stub `xcrun` keeps each device as a file, so
# no real simulator is touched.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script="$root/scripts/ios-sim-device.sh"
template=intrada-template-iPhone-16-iOS26-5

tmp="$(mktemp -d)"
live_pids=""
trap '{ [ -z "$live_pids" ] || kill $live_pids; wait; } 2>/dev/null; rm -rf "$tmp"' EXIT

mkdir "$tmp/bin"
cat >"$tmp/bin/xcrun" <<'EOF'
#!/usr/bin/env python3
import json, os, sys, uuid

state = os.environ["SIM_STATE"]
with open(os.environ["XCRUN_LOG"], "a") as log:
    log.write(" ".join(sys.argv[2:]) + "\n")
cmd, args = sys.argv[2], sys.argv[3:]

def read(udid):
    with open(os.path.join(state, udid)) as f:
        return json.load(f)

def write(udid, d):
    with open(os.path.join(state, udid), "w") as f:
        json.dump(d, f)

def new(name):
    udid = str(uuid.uuid4()).upper()
    write(udid, {"name": name, "udid": udid, "state": "Shutdown"})
    print(udid)

if cmd == "list":
    print(json.dumps({"devices": {"iOS-26-5": [read(u) for u in os.listdir(state)]}}))
elif cmd == "create":
    new(args[0])
elif cmd == "clone":
    if os.environ.get("STUB_FAIL_CLONE") or read(args[0])["state"] != "Shutdown":
        sys.exit(1)
    new(args[1])
elif cmd == "bootstatus":
    d = read(args[0]); d["state"] = "Booted"; write(args[0], d)
    if os.environ.get("STUB_FAIL_BOOT"):
        sys.exit(1)
elif cmd == "shutdown":
    d = read(args[0])
    if d["state"] != "Booted":
        sys.exit(1)
    d["state"] = "Shutdown"; write(args[0], d)
elif cmd == "rename":
    d = read(args[0]); d["name"] = args[1]; write(args[0], d)
elif cmd == "delete":
    if read(args[0])["state"] == "Booted":
        sys.exit(1)
    os.remove(os.path.join(state, args[0]))
EOF
chmod +x "$tmp/bin/xcrun"
export PATH="$tmp/bin:$PATH"

export IOS_SIM_TEMPLATE_LOCK_DIR="$tmp/lock"
export IOS_SIM_TEMPLATE_LOCK_TIMEOUT=0
export IOS_SIM_LOCK_POLL=1

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

fresh() {
  export SIM_STATE="$tmp/state-$1" XCRUN_LOG="$tmp/log-$1"
  mkdir "$SIM_STATE"
  : >"$XCRUN_LOG"
}

seed() {
  local udid="$1-seeded"
  printf '{"name": "%s", "udid": "%s", "state": "%s", "isAvailable": %s}' \
    "$1" "$udid" "$2" "${3:-true}" >"$SIM_STATE/$udid"
}

udid_of() {
  grep -l "\"name\": \"$1\"" "$SIM_STATE"/* 2>/dev/null | xargs -n1 basename 2>/dev/null
}

state_of() {
  python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["state"])' "$SIM_STATE/$(udid_of "$1")"
}

calls() {
  grep -c "^$1" "$XCRUN_LOG" || true
}

run() {
  bash "$script" "$1" 2>"$tmp/stderr"
}

# ── First worktree prepares the template, the next only clones ──

fresh first
out="$(run wt-a)"
expect "the first device prints its own UDID" true test "$out" = "$(udid_of wt-a)"
expect "the first device is cloned from the template" true grep -q "^clone .* wt-a$" "$XCRUN_LOG"
expect "the first device is not a blank fallback" false grep -q "^create wt-a " "$XCRUN_LOG"
expect "the template is booted once while it is prepared" true test "$(calls bootstatus)" = 1
expect "the prepared template carries its finished name" true test -n "$(udid_of "$template")"
expect "the prepared template is left shut down" true test "$(state_of "$template")" = Shutdown
expect "the template lock is released after a clone" false test -d "$IOS_SIM_TEMPLATE_LOCK_DIR"

: >"$XCRUN_LOG"
out="$(run wt-b)"
expect "a second device prints its own UDID" true test "$out" = "$(udid_of wt-b)"
expect "a second device reuses the template without creating anything" true test "$(calls create)" = 0
expect "a second device does not boot the template" true test "$(calls bootstatus)" = 0

# ── A template someone left booted is shut down before the clone ──

fresh booted
seed "$template" Booted
out="$(run wt-c)"
expect "a booted template still yields a clone" true grep -q "^clone .* wt-c$" "$XCRUN_LOG"
expect "a booted template is not replaced by a blank fallback" false grep -q "^create wt-c " "$XCRUN_LOG"
expect "a booted template is shut down for the clone" true test "$(state_of "$template")" = Shutdown

# ── Templates from an old pin or a dead prepare are deleted ──

fresh stale
seed intrada-template-iPhone-15-iOS26-0 Shutdown
seed "$template-preparing" Booted
seed intrada-test-26-5-other-worktree Booted
run wt-e >/dev/null
expect "an old pin's template is deleted" true test -z "$(udid_of intrada-template-iPhone-15-iOS26-0)"
expect "a half-prepared template is deleted" true test -z "$(udid_of "$template-preparing")"
expect "another worktree's device is left alone" true test -n "$(udid_of intrada-test-26-5-other-worktree)"

fresh unavailable
seed "$template" Shutdown false
run wt-u >/dev/null
expect "a template its runtime no longer supports is replaced" true \
  test "$(udid_of "$template")" != "$template-seeded"
expect "a clone comes from the replacement template" true grep -q "^clone .* wt-u$" "$XCRUN_LOG"

# ── Every failure falls back to a blank device ──

fresh clone-fails
out="$(STUB_FAIL_CLONE=1 run wt-d)"
expect "a failed clone still prints a device" true test "$out" = "$(udid_of wt-d)"
expect "a failed clone creates a blank device" true grep -q "^create wt-d " "$XCRUN_LOG"
expect "a failed clone releases the template lock" false test -d "$IOS_SIM_TEMPLATE_LOCK_DIR"

fresh boot-fails
out="$(STUB_FAIL_BOOT=1 run wt-f)"
expect "a failed prepare still prints a device" true test "$out" = "$(udid_of wt-f)"
expect "a failed prepare leaves no finished template" true test -z "$(udid_of "$template")"
expect "a failed prepare deletes its unfinished device" true test -z "$(udid_of "$template-preparing")"
expect "a failed prepare releases the template lock" false test -d "$IOS_SIM_TEMPLATE_LOCK_DIR"

fresh lock-held
sleep 60 &
holder=$!
live_pids="$holder"
mkdir "$IOS_SIM_TEMPLATE_LOCK_DIR"
echo "$holder" >"$IOS_SIM_TEMPLATE_LOCK_DIR/pid"
out="$(run wt-g)"
expect "a lock held past the timeout still prints a device" true test "$out" = "$(udid_of wt-g)"
expect "a lock held past the timeout creates a blank device" true grep -q "^create wt-g " "$XCRUN_LOG"
expect "a lock held by another prepare is not taken from it" true \
  test "$(cat "$IOS_SIM_TEMPLATE_LOCK_DIR/pid")" = "$holder"
kill "$holder"
wait "$holder" 2>/dev/null || true
live_pids=""
rm -rf "$IOS_SIM_TEMPLATE_LOCK_DIR"

# ── Two worktrees at once prepare the template only once ──

fresh concurrent
IOS_SIM_TEMPLATE_LOCK_TIMEOUT=30 bash "$script" wt-h >"$tmp/out-h" 2>/dev/null &
first=$!
IOS_SIM_TEMPLATE_LOCK_TIMEOUT=30 bash "$script" wt-i >"$tmp/out-i" 2>/dev/null &
second=$!
wait "$first" "$second"
expect "two worktrees at once create the template only once" true \
  test "$(grep -c "^create $template-preparing " "$XCRUN_LOG")" = 1
expect "two worktrees at once both get clones" true \
  test "$(grep -c "^clone " "$XCRUN_LOG")" = 2
expect "two worktrees at once each print their own device" true \
  test "$(cat "$tmp/out-h") $(cat "$tmp/out-i")" = "$(udid_of wt-h) $(udid_of wt-i)"

echo "ios-sim-device-test: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
