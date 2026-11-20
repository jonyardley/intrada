#!/usr/bin/env bash
# Should the Runner health workflow raise the runner-broken issue (#2916)?
# GitHub delivers the hourly schedule hours late, so the age of the last pass
# alone is no evidence; one run alone never raises.
#
# Usage: runner-alarm.sh stale|failed RUN_ID < runs.json
#   runs.json: `gh run list --workflow runner-health.yml --json
#   databaseId,status,conclusion,createdAt`, newest first.
#   stale: no check has reached the script. Raise when the two runs before
#     this one did not pass and none passed in the last three hours, so one
#     blip followed by a late schedule does not raise.
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

# Earlier runs have lower ids; a queued or cancelled run counts as not
# passed, which is what an offline Mac leaves (#2916).
jq -r --arg mode "$mode" --argjson id "$run_id" --argjson now "$now" '
  [.[] | select(.databaseId < $id)] | sort_by(.databaseId) | reverse
  | (first // null) as $prev
  | (.[1] // null) as $prev2
  | (map(select(.conclusion == "success")) | first // null) as $pass
  | if $prev == null then "hold: no earlier health run to compare with"
    elif $prev.conclusion == "success" then
      "hold: the run before this one passed (\($prev.databaseId))"
    elif $mode == "failed" then
      "raise: the run before this one did not pass either (\($prev.databaseId), \(if ($prev.conclusion // "") == "" then $prev.status else $prev.conclusion end))"
    elif $prev2 == null or $prev2.conclusion == "success" then
      "hold: only one run before this one did not pass (\($prev.databaseId))"
    elif $pass == null then
      "raise: no earlier run in the list passed, and the last two did not either"
    elif ($now - ($pass.createdAt | fromdateiso8601)) > 10800 then
      "raise: no run has passed since \($pass.createdAt), and the last two did not either"
    else "hold: a run passed within three hours (\($pass.createdAt))"
    end
'
