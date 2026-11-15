#!/usr/bin/env bash
# Is the self-hosted Mac fit to run the iOS gate (#2876)? A full disk and an
# Xcode the gate did not expect each failed every draft for days, read as red
# pull requests (72.5 draft hours). Run first by the gate and hourly by the
# Runner health workflow; each problem prints a GitHub error annotation.
#
# The pins come from ci.yml's env, read from the file when the caller has not
# set them, so the hourly run never holds a second copy. To force a failure
# locally: RUNNER_MIN_FREE_GB=100000 bash scripts/runner-health.sh
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
volume="${RUNNER_HEALTH_VOLUME:-/System/Volumes/Data}"
min_gb="${RUNNER_MIN_FREE_GB:-25}"

pin() {
  sed -n "s/^  $1: \"\(.*\)\"$/\1/p" "$root/.github/workflows/ci.yml" | head -1
}
xcode="${SELFHOSTED_XCODE:-$(pin SELFHOSTED_XCODE)}"
runtime="${SELFHOSTED_IOS_RUNTIME:-$(pin SELFHOSTED_IOS_RUNTIME)}"

fix="Fix: prune the runner (run the Runner housekeeping workflow, or free space by hand), or set the repository variable IOS_RUNNER=hosted to move the gate to GitHub's Mac."
problems=0
broken() {
  problems=$((problems + 1))
  echo "::error title=Mac runner broken%2C not your code::$1 $fix"
}

free_kb="$(df -Pk "$volume" 2>/dev/null | awk 'NR==2{print $4}')"
if ! [[ "$free_kb" =~ ^[0-9]+$ ]]; then
  broken "Could not read free space on $volume."
elif [ $((free_kb / 1024 / 1024)) -lt "$min_gb" ]; then
  broken "Only $((free_kb / 1024 / 1024)) GB free on $volume; the gate needs $min_gb GB."
fi

have="$(xcodebuild -version 2>/dev/null | head -1)"
if [ -z "$xcode" ]; then
  broken "No Xcode pin found in ci.yml."
elif [ "$have" != "Xcode $xcode" ]; then
  broken "The runner is on ${have:-no Xcode}; the gate expects Xcode $xcode (#1637)."
fi

if [ -z "$runtime" ]; then
  broken "No iOS runtime pin found in ci.yml."
elif ! grep -q "iOS $runtime " <<<"$(xcrun simctl list runtimes 2>/dev/null)"; then
  broken "The iOS $runtime simulator runtime the snapshot references pin is missing."
fi

if [ "$problems" -gt 0 ]; then
  exit 1
fi
echo "runner healthy: $((free_kb / 1024 / 1024)) GB free, Xcode $xcode, iOS $runtime runtime"
