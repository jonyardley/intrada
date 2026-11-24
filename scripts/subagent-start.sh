#!/usr/bin/env bash
# Hand every helper agent the plain git rule before its first command (#2976).
# Claude Code refuses a git command chained with other steps in a worktree,
# because it cannot check where the chain writes, and a refused call is a
# wasted one.
#
# Run by the SubagentStart hook in .claude/settings.json.

set -euo pipefail

jq -nc --arg c "In this repo, run each git command as its own plain call: never chain it with &&, ;, a pipe, a redirection, a loop or a file step such as cp or mv. Claude Code refuses a compound command with git in it that it cannot check, and the retry costs a call. Run the steps one call at a time; a loop over git goes in a scratch script, written before the first try." \
  '{hookSpecificOutput: {hookEventName: "SubagentStart", additionalContext: $c}}'
