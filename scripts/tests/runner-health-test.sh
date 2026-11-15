#!/usr/bin/env bash
# Self-test for scripts/runner-health.sh (#2876): a healthy machine passes, and
# a low disk, the wrong Xcode and a missing runtime each fail with the "not
# your code" annotation. Stub `df`, `xcodebuild` and `xcrun` stand in for the
# Mac, so it runs anywhere.
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
printf 'Xcode %s\nBuild version 1\n' "$HAVE_XCODE"
EOF
cat >"$tmp/bin/xcrun" <<'EOF'
#!/usr/bin/env bash
echo "iOS $HAVE_RUNTIME (1) - com.apple.CoreSimulator.SimRuntime.iOS"
EOF
chmod +x "$tmp/bin/"*
export PATH="$tmp/bin:$PATH"

pin() { sed -n "s/^  $1: \"\(.*\)\"$/\1/p" "$root/.github/workflows/ci.yml" | head -1; }
xcode="$(pin SELFHOSTED_XCODE)"
runtime="$(pin SELFHOSTED_IOS_RUNTIME)"
[ -n "$xcode" ] && [ -n "$runtime" ] || { echo "✗ pins not found in ci.yml" >&2; exit 1; }
unset SELFHOSTED_XCODE SELFHOSTED_IOS_RUNTIME RUNNER_MIN_FREE_GB

pass=0
fail=0
# expect <exit status> <text the output must hold> <name> <env assignments...>
expect() {
  local want="$1" text="$2" name="$3" out status
  shift 3
  set +e
  out="$(env FREE_KB=$((100 * 1024 * 1024)) HAVE_XCODE="$xcode" HAVE_RUNTIME="$runtime" "$@" bash "$root/scripts/runner-health.sh" 2>&1)"
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

echo "runner-health: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
