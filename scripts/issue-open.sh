#!/usr/bin/env bash
# Wrap `gh issue create`, refusing an issue without a horizon label and a kind
# label (#2696): 8 of the 30 issues opened to 15 October 2026 had no horizon
# label, so they dropped out of the horizon views and nobody planned them.
#
# Called via `just issue-open <title> <body file> <labels> [extra flags]`,
# labels comma separated: just issue-open "Title" body.md "horizon:next,ux".

set -euo pipefail
set -f

kinds="bug ux architecture accessibility documentation ios android security epic"

# Labels are collected, stripped of spaces and forwarded as one list, since gh
# matches names exactly and would reject " bug".
labels=""
passthrough=()
while [ $# -gt 0 ]; do
  case "$1" in
    --label)
      labels="$labels,${2:-}"
      shift 2 || shift
      ;;
    *)
      passthrough+=("$1")
      shift
      ;;
  esac
done
labels="$(printf '%s' "$labels" | tr -d ' ' | tr -s ',' | sed 's/^,//; s/,$//')"

has_horizon=""
has_kind=""
for name in $(printf '%s' "$labels" | tr ',' ' '); do
  case "$name" in horizon:now | horizon:next | horizon:later) has_horizon=1 ;; esac
  for kind in $kinds; do
    [ "$name" = "$kind" ] && has_kind=1
  done
done

if [ -z "$has_horizon" ]; then
  echo "✗ no horizon label: add one of horizon:now, horizon:next or horizon:later." >&2
  exit 1
fi
if [ -z "$has_kind" ]; then
  echo "✗ no kind label: add one of $(printf '%s' "$kinds" | sed 's/ /, /g') (docs/how-the-work-runs.md, Labels)." >&2
  exit 1
fi

gh issue create ${passthrough[@]+"${passthrough[@]}"} --label "$labels"
