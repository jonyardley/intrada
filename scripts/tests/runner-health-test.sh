#!/usr/bin/env bash
# Self-test for scripts/runner-health.sh (#2876): a healthy machine passes, and
# a low disk, the wrong Xcode and a missing runtime each fail with the "not
# your code" annotation. Stub `df`, `xcodebuild` and `xcrun` stand in for the
# Mac, so it runs anywhere. A fixture ci.yml holds the pins, so the script's
# own parser is what reads them; one case reads the real ci.yml.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir "$tmp/bin"
cat >"$tmp/bin/df" <<'EOF'
#!/usr/bin/env bash
echo "Filesystem 1024-blocks Used Available Capacity Mounted on"
echo "/dev/disk3s5 971298980 1 $FREE_KB 1% /System/Volumes/Data"
EOF
cat >"$tmp/bin/xcodebuild" <<'EOF'
#!/usr/bin/env bash
[ -z "${HANG:-}" ] || exec sleep 30
printf 'Xcode %s\nBuild version 1\n' "$HAVE_XCODE"
EOF
cat >"$tmp/bin/xcrun" <<'EOF'
#!/usr/bin/env bash
[ -z "${HANG_SIMCTL:-}" ] || exec sleep 30
echo "iOS $HAVE_RUNTIME (1) - com.apple.CoreSimulator.SimRuntime.iOS"
EOF
chmod +x "$tmp/bin/"*
export PATH="$tmp/bin:$PATH"

xcode="27.2"
runtime="26.5"
printf 'env:\n  SELFHOSTED_XCODE: "%s"\n  SELFHOSTED_IOS_RUNTIME: "%s"\n' "$xcode" "$runtime" >"$tmp/ci.yml"
printf 'env:\n  OTHER: "1"\n' >"$tmp/no-pins.yml"
unset SELFHOSTED_XCODE SELFHOSTED_IOS_RUNTIME RUNNER_MIN_FREE_GB RUNNER_HEALTH_TIMEOUT

pass=0
fail=0
# expect <exit status> <text the output must hold> <name> <env assignments...>
expect() {
  local want="$1" text="$2" name="$3" out status
  shift 3
  set +e
  out="$(env FREE_KB=$((100 * 1024 * 1024)) HAVE_XCODE="$xcode" HAVE_RUNTIME="$runtime" RUNNER_HEALTH_CI_FILE="$tmp/ci.yml" "$@" bash "$root/scripts/runner-health.sh" 2>&1)"
  status=$?
  set -e
  if [ "$status" = "$want" ] && grep -qF -- "$text" <<<"$out"; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "✗ $name: exit $status, output:" >&2
    sed 's/^/    /' <<<"$out" >&2
  fi
}

title="::error title=Mac runner broken%2C not your code::"
expect 0 "runner healthy" "healthy machine passes"
expect 1 "${title}Only 3 GB free" "low disk fails" FREE_KB=$((3 * 1024 * 1024))
expect 0 "runner healthy" "12 GB free passes, as the gate does" FREE_KB=$((12 * 1024 * 1024))
expect 0 "runner healthy" "exactly the 8 GB floor passes" FREE_KB=$((8 * 1024 * 1024))
expect 1 "${title}Only 7 GB free" "just under the 8 GB floor fails" FREE_KB=$((8 * 1024 * 1024 - 1))
expect 1 "IOS_RUNNER=hosted" "the failure names the hosted fallback" FREE_KB=$((3 * 1024 * 1024))
expect 1 "$title" "forced threshold fails" RUNNER_MIN_FREE_GB=100000
expect 1 "${title}The runner is on Xcode 1.0" "wrong Xcode fails" HAVE_XCODE=1.0
expect 1 "${title}The iOS $runtime simulator runtime" "missing runtime fails" HAVE_RUNTIME=1.0
expect 1 "Could not read free space" "unreadable disk fails" FREE_KB=
expect 0 "Xcode 9.9" "an env pin wins over ci.yml" SELFHOSTED_XCODE=9.9 HAVE_XCODE=9.9
expect 0 "iOS 9.9 runtime" "an env runtime pin wins over ci.yml" SELFHOSTED_IOS_RUNTIME=9.9 HAVE_RUNTIME=9.9
expect 1 "No Xcode pin found" "a missing Xcode pin fails" RUNNER_HEALTH_CI_FILE="$tmp/no-pins.yml"
expect 1 "No iOS runtime pin found" "a missing runtime pin fails" RUNNER_HEALTH_CI_FILE="$tmp/no-pins.yml"
expect 1 "The iOS $runtime simulator runtime" "the runtime pin is not a regex" HAVE_RUNTIME=2605
expect 1 "xcodebuild did not answer within 1 seconds" "a wedged xcodebuild fails" HANG=1 RUNNER_HEALTH_TIMEOUT=1
expect 1 "simctl did not answer within 1 seconds" "a wedged simctl fails" HANG_SIMCTL=1 RUNNER_HEALTH_TIMEOUT=1
# The real ci.yml: both pins found, so each mismatch names the pin it expects.
expect 1 "the gate expects Xcode " "the real ci.yml holds the Xcode pin" RUNNER_HEALTH_CI_FILE= HAVE_XCODE=0
expect 1 "simulator runtime the snapshot" "the real ci.yml holds the runtime pin" RUNNER_HEALTH_CI_FILE= HAVE_RUNTIME=0

# The alarm (#2916): whether the stale and report jobs raise the issue, given
# the run list. Run 100 is this run; ids only grow. NOW is 2026-11-20 03:26.
now=$(date -u -j -f '%Y-%m-%dT%H:%M:%SZ' 2026-11-20T03:26:30Z +%s 2>/dev/null || date -u -d 2026-11-20T03:26:30Z +%s)
run() { printf '{"databaseId":%s,"status":"%s","conclusion":"%s","createdAt":"%s"}' "$@"; }
this="$(run 100 in_progress '' 2026-11-20T03:26:27Z)"
# alarm <want: raise|hold> <mode> <name> <runs...>
alarm() {
  local want="$1" mode="$2" name="$3" out
  shift 3
  out="$( (IFS=,; echo "[$*]") | NOW="$now" bash "$root/scripts/runner-alarm.sh" "$mode" 100 2>&1)" || true
  if [ "${out%%:*}" = "$want" ]; then
    pass=$((pass + 1))
  else
    fail=$((fail + 1))
    echo "✗ alarm, $name: want $want, got: $out" >&2
  fi
}
passed_6h="$(run 99 completed success 2026-11-19T21:41:11Z)"
alarm hold stale "a delayed schedule with the last run passing holds (the false alarms)" "$this" "$passed_6h" "$(run 98 completed success 2026-11-19T13:22:25Z)"
alarm hold stale "a pass within three hours holds" "$this" "$(run 99 completed failure 2026-11-20T02:23:00Z)" "$(run 98 completed success 2026-11-20T01:23:00Z)"
alarm raise stale "a queued run after a pass over three hours old raises" "$this" "$(run 99 queued '' 2026-11-20T00:23:00Z)" "$(run 98 completed success 2026-11-19T21:41:11Z)"
alarm raise stale "a cancelled run after an old pass raises" "$this" "$(run 99 completed cancelled 2026-11-20T00:23:00Z)" "$(run 98 completed success 2026-11-19T21:41:11Z)"
alarm raise stale "no pass anywhere in the list raises" "$this" "$(run 99 completed failure 2026-11-20T00:23:00Z)" "$(run 98 completed cancelled 2026-11-19T21:41:11Z)"
alarm hold stale "a newer run queued behind this one is ignored" "$(run 101 queued '' 2026-11-20T04:23:00Z)" "$this" "$passed_6h"
alarm hold stale "no earlier run holds" "$this"
alarm hold failed "one failed check after a pass holds" "$this" "$passed_6h"
alarm raise failed "a second failure in a row raises" "$this" "$(run 99 completed failure 2026-11-19T21:41:11Z)" "$(run 98 completed success 2026-11-19T13:22:25Z)"
alarm raise failed "a failure after a queued run raises" "$this" "$(run 99 queued '' 2026-11-19T21:41:11Z)"
alarm hold failed "the first run ever holds" "$this"
usage="$(bash "$root/scripts/runner-alarm.sh" other 100 </dev/null 2>&1)" && status=0 || status=$?
if [ "$status" = 2 ] && grep -qF usage <<<"$usage"; then pass=$((pass + 1)); else
  fail=$((fail + 1))
  echo "✗ alarm, an unknown mode is refused: exit $status" >&2
fi

echo "runner-health: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
