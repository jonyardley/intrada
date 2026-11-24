#!/usr/bin/env bash
# Keeps the self-hosted Mac's disk clear after each gate run (#2983): deletes
# spare simulators every run and, on a push to main only (TIDY_RUNTIMES=true),
# old iOS runtimes. A failed delete is a warning; the next run tries again.
set -euo pipefail

if [ "$#" -gt 0 ]; then
  echo "usage: runner-tidy.sh" >&2
  exit 2
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ci="${RUNNER_HEALTH_CI_FILE:-$root/.github/workflows/ci.yml}"
runtime="${SELFHOSTED_IOS_RUNTIME:-$(sed -n 's/^  SELFHOSTED_IOS_RUNTIME: "\(.*\)"$/\1/p' "$ci" 2>/dev/null | head -1)}"
# With no pin every runtime would look old, so refuse rather than take them all.
if [ "${TIDY_RUNTIMES:-}" = "true" ] && [ -z "$runtime" ]; then
  echo "::error::No iOS runtime pin found in ci.yml, so nothing is deleted."
  exit 1
fi

# Kept: the test devices, the clone template (#1904) and main's smoke device.
for udid in $(xcrun simctl list devices --json | python3 -c "
import json, sys
for group in json.load(sys.stdin)['devices'].values():
    for d in group:
        n = d['name']
        if not (n.startswith(('intrada-test-', 'intrada-template-')) or n == 'iPhone 16'):
            print(d['udid'])
"); do
  echo "deleting simulator $udid"
  xcrun simctl shutdown "$udid" >/dev/null 2>&1 || true
  xcrun simctl delete "$udid" || echo "::warning::Could not delete simulator $udid."
done

# Main only: a pull request may change the pin, and its pin would take main's
# runtime. Kept: the pin and its patch releases, and the newest, in case Xcode
# needs its own.
if [ "${TIDY_RUNTIMES:-}" = "true" ]; then
  for id in $(xcrun simctl runtime list -j | python3 -c "
import json, sys
pin = sys.argv[1]
def key(v):
    return tuple(int(p) if p.isdigit() else 0 for p in v.split('.'))
ios = [r for r in json.load(sys.stdin).values()
       if r.get('platformIdentifier') == 'com.apple.platform.iphonesimulator']
newest = max((key(r.get('version', '')) for r in ios), default=None)
for r in ios:
    v = r.get('version', '')
    if v == pin or v.startswith(pin + '.') or key(v) == newest or not r.get('deletable'):
        continue
    print(r['identifier'])
" "$runtime"); do
    echo "deleting iOS runtime $id"
    xcrun simctl runtime delete "$id" || echo "::warning::Could not delete iOS runtime $id."
  done
fi

df -h /System/Volumes/Data 2>/dev/null | tail -1 || true
