#!/usr/bin/env bash
# The subagent start hook still hands each helper agent the plain git rule
# (#2976), and settings.json still runs it.

set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
fail=0
ok() { printf '✓ %s\n' "$1"; }
bad() { printf '✗ %s\n' "$1" >&2; fail=1; }

out="$(bash "$root/scripts/subagent-start.sh" </dev/null)"
event="$(jq -r '.hookSpecificOutput.hookEventName' <<<"$out")"
context="$(jq -r '.hookSpecificOutput.additionalContext' <<<"$out")"
[ "$event" = SubagentStart ] && ok "names the SubagentStart event" || bad "event is $event, not SubagentStart"
grep -q 'its own plain call' <<<"$context" && ok "the context names plain git calls" || bad "the context does not name plain git calls"

wired="$(jq -r '.hooks.SubagentStart[]?.hooks[]?.command' "$root/.claude/settings.json")"
grep -q 'scripts/subagent-start.sh' <<<"$wired" && ok "settings.json runs it at subagent start" || bad "settings.json does not run it at subagent start"

exit "$fail"
