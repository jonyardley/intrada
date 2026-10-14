---
description: Run the pre-push gates and the self-review before opening or updating a PR. The mechanics live in the intrada-shipping skill; this fixes the order.
---

Prepare the current branch to ship. Read `.claude/skills/intrada-shipping/SKILL.md` first: it holds the why and the body templates; this file holds the order.

1. **Gates**, through the `test-runner` agent: `just check`, plus `just ios-fmt-check` and `just ios-test` when `ios/` changed, plus `just ios-test-ui-class <Class>` for each UI test class the diff adds or edits; CI runs the full UI tier (#2114). Stop at the first failure and report its output.
2. **Self-review** before the push: `reviewer` over the local diff, briefed with the worktree and the issue, on any code or test change; a docs or harness-text diff takes `/code-review` inline. Fix blockers inline. A fix touching `ios/` reruns `just ios-test` plus `just ios-test-ui-class` for the class it touched (#1884).
3. **Deferred items**: an issue for each before the review comment.
4. **Open** with `just pr-open ... --draft`, post the review summary naming the agent, then `just pr-ready PR ISSUE`. A screen change waits for Jon's yes on its screenshot.
5. **Watch CI** to a conclusion in the same turn and read the PR's mergeability, not just the jobs. Reproduce a red UI test with `just ios-test-ui-class <Class>`, not the full tier.

$ARGUMENTS
