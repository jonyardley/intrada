#!/usr/bin/env bash
# Should the Runner health workflow raise the runner-broken issue (#2916)?
# From 15 to 20 November it opened and closed 14 issues, none real: GitHub
# delivered the hourly schedule only every four to ten hours, so the stale job
# always found the last pass over three hours old and opened the issue, and
# the same run's passing check closed it seconds later. So neither job raises
# on its own run alone: each first asks whether the run before it passed.
#
# Usage: runner-alarm.sh stale|failed RUN_ID < runs.json
#   runs.json: `gh run list --workflow runner-health.yml --json
#   databaseId,status,conclusion,createdAt`, newest first.
#   stale: no check has reached the script. Raise when the run before this
#     one did not pass and no earlier run passed in the last three hours.
#   failed: this run's check said the Mac is broken. Raise when the run
#     before this one did not pass either, so one blip waits for a second.
# Prints "raise" or "hold" and why. NOW (epoch seconds) is for the self-test.
set -euo pipefail

mode="${1:-}"
run_id="${2:-}"
case "$mode" in stale | failed) ;; *)
  echo "usage: runner-alarm.sh stale|failed RUN_ID < runs.json" >&2
  exit 2
  ;;
esac
[[ "$run_id" =~ ^[0-9]+$ ]] || {
  echo "runner-alarm: RUN_ID must be a run number, got '$run_id'" >&2
  exit 2
}
now="${NOW:-$(date -u +%s)}"

# Run ids only grow, so the runs before this one are those with a lower id;
# a run queued behind this one is not evidence about it. A run still queued
# or cancelled counts as not passed: that is what an offline Mac leaves.
jq -r --arg mode "$mode" --argjson id "$run_id" --argjson now "$now" '
  [.[] | select(.databaseId < $id)] | sort_by(.databaseId) | reverse
  | (first // null) as $prev
  | (map(select(.conclusion == "success")) | first // null) as $pass
  | if $prev == null then "hold: no earlier health run to compare with"
    elif $prev.conclusion == "success" then
      "hold: the run before this one passed (\($prev.databaseId))"
    elif $mode == "failed" then
      "raise: the run before this one did not pass either (\($prev.databaseId), \(if ($prev.conclusion // "") == "" then $prev.status else $prev.conclusion end))"
    elif $pass == null then
      "raise: no earlier run in the list passed, and the last one did not either"
    elif ($now - ($pass.createdAt | fromdateiso8601)) > 10800 then
      "raise: no run has passed since \($pass.createdAt), and the last one did not either"
    else "hold: a run passed within three hours (\($pass.createdAt))"
    end
'
