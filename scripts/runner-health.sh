#!/usr/bin/env bash
# Is the self-hosted Mac fit to run the iOS gate (#2876)? A full disk and an
# Xcode the gate did not expect each failed every draft for days, read as red
# pull requests (72.5 draft hours). Run by the gate after checkout and hourly
# by the Runner health workflow; each problem prints a GitHub error annotation.
#
# The pins come from ci.yml's env, read from the file when the caller has not
# set them, so the hourly run never holds a second copy. To force a failure
# locally: RUNNER_MIN_FREE_GB=100000 bash scripts/runner-health.sh
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ci="${RUNNER_HEALTH_CI_FILE:-$root/.github/workflows/ci.yml}"
volume="${RUNNER_HEALTH_VOLUME:-/System/Volumes/Data}"
# The floor warns while there is still room: the UI tests failed at 11 GB free
# (#2983), and with the gate's tidy after each run (runner-tidy.sh) the Mac
# sits near 120 GB free, so 40 GB means something is piling up again.
min_gb="${RUNNER_MIN_FREE_GB:-40}"
limit="${RUNNER_HEALTH_TIMEOUT:-60}"

pin() {
  sed -n "s/^  $1: \"\(.*\)\"$/\1/p" "$ci" 2>/dev/null | head -1
}

# bounded <command...>: its output into $out, stopped after $limit seconds
# with status 124. macOS has no timeout(1), and a wedged xcodebuild or simctl
# would otherwise hang until the job is cancelled, which files no issue. The
# output goes to a file, not a pipe, so a child left behind cannot hold it.
out="$(mktemp)"
trap 'rm -f "$out"' EXIT
bounded() {
  "$@" >"$out" 2>/dev/null &
  local pid=$! status=0 watch
  ( sleep "$limit"; kill -9 "$pid" ) >/dev/null 2>&1 &
  watch=$!
  { wait "$pid"; } 2>/dev/null || status=$?
  kill "$watch" 2>/dev/null
  { wait "$watch"; } 2>/dev/null
  [ "$status" -ne 137 ] || status=124
  return "$status"
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

bounded xcodebuild -version
xcode_status=$?
have="$(head -1 "$out")"
if [ "$xcode_status" -eq 124 ]; then
  broken "xcodebuild did not answer within $limit seconds."
elif [ -z "$xcode" ]; then
  broken "No Xcode pin found in ci.yml."
elif [ "$have" != "Xcode $xcode" ]; then
  broken "The runner is on ${have:-no Xcode}; the gate expects Xcode $xcode (#1637)."
fi

bounded xcrun simctl list runtimes
runtime_status=$?
if [ "$runtime_status" -eq 124 ]; then
  broken "simctl did not answer within $limit seconds."
elif [ -z "$runtime" ]; then
  broken "No iOS runtime pin found in ci.yml."
elif ! grep -qF "iOS $runtime " "$out"; then
  broken "The iOS $runtime simulator runtime the snapshot references pin is missing."
fi

if [ "$problems" -gt 0 ]; then
  exit 1
fi
echo "runner healthy: $((free_kb / 1024 / 1024)) GB free, Xcode $xcode, iOS $runtime runtime"
