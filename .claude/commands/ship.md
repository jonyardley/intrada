---
description: Run the pre-push gates and the self-review before opening or updating a PR. The mechanics live in the intrada-shipping skill; this fixes the order.
---

Prepare the current branch to ship. Read `.claude/skills/intrada-shipping/SKILL.md` first: it is the source for every step below.

1. **Gates**, through the `test-runner` agent: `just check`, plus `just ios-fmt-check` and `just ios-test` when files under `ios/` changed, plus `just ios-test-ui-class <Class>` for each UI test class the diff adds or edits. CI's gate on the draft is the first full UI run, six simulators at a time (#2114). Stop at the first failure and report its actual output.
2. **Self-review**, before the push: the `reviewer` agent over the local diff, pinned to Opus 5.5 high and briefed with the worktree and the issue, on every diff including the sensitive surfaces. Fix blockers inline. A fix touching `ios/` reruns `just ios-test` plus `just ios-test-ui-class <Class>` for the class the fix touched (#1884); a fix with no UI test to name just reruns `just ios-test`.
3. **Deferred items**: open an issue for each before posting the review comment.
4. **Open the PR** as a draft with the body template from the skill, post the review summary citing the agent, then mark it ready.
5. **Watch CI** to a conclusion in the same turn, and read the PR's mergeability, not just the job list. A red UI test is reproduced with `just ios-test-ui-class <Class>` for the failing class, not the full tier.

$ARGUMENTS
