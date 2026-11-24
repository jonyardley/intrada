---
description: Run the retro, the pre-push gates and the self-review before opening or updating a PR. The mechanics live in the intrada-shipping skill; this fixes the order.
---

Prepare the current branch to ship. Read `.claude/skills/intrada-shipping/SKILL.md` first: it holds the why and the body templates; this file holds the order. Every agent spawn below names its model and effort from the role table in `~/.claude/CLAUDE.md`: definitions carry no pins, and a spawn with no model named stops for a permission prompt.

1. **Retro**: before the PR first opens, find two or three lessons from this session, one per kind the header of `docs/lessons.md` names (a mistake, a slow or costly step, a tool or doc that misled), and give each its outcome first, as that header's "Acting on a lesson" says, which also lists the files a fix may touch: fixed in this PR in its own commit, an issue through `just issue-open`, or kept. Then write each one where its outcome sends it, marking repeats and archiving at the limit as the header says, and commit, so the gates and the self-review cover it all. List the fixes on the PR body's "Process fixes" line. A later run for the same PR adds only a lesson not yet recorded.
2. **Gates**, through the `test-runner` agent: `just pre-review` (`just check`, plus the iOS format gate), plus `just ios-test` when `ios/` changed, plus `just ios-test-ui-class <Class>` for each UI test class the diff adds or edits; CI runs the full UI tier (#2114). Stop at the first failure and report its output.
3. **Self-review** before the push, only once step 2 passes: `reviewer` over the local diff, briefed with the worktree and the issue, or `/code-review` inline, or none, as `just review-route` says by `docs/working-with-agents.md` section 6; `reviewer` is spawned at the model and effort the role table in `~/.claude/CLAUDE.md` (Subagents, review and lane size) gives, with rounds as that section and "Speed and reliability" say. Fix blockers inline. A fix touching `ios/` reruns `just ios-test` plus `just ios-test-ui-class` for the class it touched (#1884); a fix with no UI test to name just reruns `just ios-test`.
4. **Deferred items**: an issue for each before the review comment.
5. **Open** a draft with `just pr-open ...` and the body template from the skill, and post the review summary naming the agent.
6. **Watch CI** to a conclusion in the same turn and read the PR's mergeability, not just the jobs. Reproduce a red UI test with `just ios-test-ui-class <Class>`, not the full tier.
7. **Ready** with `just pr-ready PR ISSUE` only once every check is green on the last commit and each review finding is fixed or answered in the PR. A screen change also waits for the owner's yes on its screenshot.
8. **After ready**, wait for the merge in the background: poll the state every three minutes for up to an hour, `for i in $(seq 20); do s=$(gh pr view PR --json state -q .state); [ "$s" != OPEN ] && break; sleep 180; done; echo "$s"`. Do not use `gh event listen`: event subscriptions are not enabled on this account, and it fails at once with HTTP 403.
   - **Merged**: write the handover for the next issue, unasked, from the main checkout (the first path in `git worktree list --porcelain`), since `just handover N` reads the current branch: `cd <main checkout> && just handover N`.
   - **Closed unmerged**: stop, write no handover, and say the PR was closed without merging.
   - **Timed out**: end the reply saying the session will check the merge itself on the owner's next message, and on that message read the PR state first.
   - Never ask the owner to say merged. Removing the worktree waits for #2918.

$ARGUMENTS
