#!/usr/bin/env bash
# Keep the self-hosted Mac's simulators from filling its disk (#2983). On
# 24 November it was 98% full: 37 unused simulators, four old iOS runtimes and
# an old checkout had piled up, and the UI tests failed on a PR that changed
# no app code. The gate runs this after every run: it deletes every simulator
# not named intrada-test- and every iOS runtime but the pinned one, in seconds
# when there is nothing to delete. The weekly erase of the test simulators
# lives in runner-housekeeping.yml.
#
# The runtime pin comes from ci.yml's env, as in runner-health.sh. A failed
# delete is a warning, never a red gate: the next run tries again.
set -euo pipefail

if [ "$#" -gt 0 ]; then
  echo "usage: runner-tidy.sh" >&2
  exit 2
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ci="${RUNNER_HEALTH_CI_FILE:-$root/.github/workflows/ci.yml}"
runtime="${SELFHOSTED_IOS_RUNTIME:-$(sed -n 's/^  SELFHOSTED_IOS_RUNTIME: "\(.*\)"$/\1/p' "$ci" 2>/dev/null | head -1)}"
# With no pin every runtime would look old, so refuse rather than take them all.
if [ -z "$runtime" ]; then
  echo "::error::No iOS runtime pin found in ci.yml, so nothing is deleted."
  exit 1
fi

for udid in $(xcrun simctl list devices --json | python3 -c "
import json, sys
for group in json.load(sys.stdin)['devices'].values():
    for d in group:
        if not d['name'].startswith('intrada-test-'):
            print(d['udid'])
"); do
  echo "deleting simulator $udid"
  xcrun simctl shutdown "$udid" >/dev/null 2>&1 || true
  xcrun simctl delete "$udid" || echo "::warning::Could not delete simulator $udid."
done

for id in $(xcrun simctl runtime list -j | python3 -c "
import json, sys
for r in json.load(sys.stdin).values():
    if r.get('platformIdentifier') == 'com.apple.platform.iphonesimulator' \
            and r.get('version') != sys.argv[1] and r.get('deletable'):
        print(r['identifier'])
" "$runtime"); do
  echo "deleting iOS runtime $id"
  xcrun simctl runtime delete "$id" || echo "::warning::Could not delete iOS runtime $id."
done

df -h /System/Volumes/Data 2>/dev/null | tail -1 || true
