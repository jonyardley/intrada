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
expect 1 "${title}Only 10 GB free" "low disk fails" FREE_KB=$((10 * 1024 * 1024))
expect 1 "IOS_RUNNER=hosted" "the failure names the hosted fallback" FREE_KB=$((10 * 1024 * 1024))
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

echo "runner-health: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
