---
description: Run the retro, the pre-push gates and the self-review before opening or updating a PR; `/ship loop` is a crew session's mode, which stops at the draft. The mechanics live in the intrada-shipping skill; this fixes the order.
---

Prepare the current branch to ship. Read `.claude/skills/intrada-shipping/SKILL.md` first: it holds the why and the body templates; this file holds the order.

1. **Retro**: before the PR first opens, add two or three lessons from this session to `docs/lessons.md`, one per kind its header names (a mistake, a slow or costly step, a tool or doc that misled), marking repeats and archiving at the limit as its header says, and commit them, so the gates and the self-review cover them. A later run for the same PR adds only a lesson not yet recorded.
2. **Gates**, through the `test-runner` agent: `just check`, plus `just ios-fmt-check` and `just ios-test` when `ios/` changed, plus `just ios-test-ui-class <Class>` for each UI test class the diff adds or edits; CI runs the full UI tier (#2114). Stop at the first failure and report its output.
3. **Self-review** before the push: `reviewer` over the local diff, briefed with the worktree and the issue, on any code or test change, spawned at effort high when the diff adds over 200 lines or touches a surface `.claude/rules/sensitive-surfaces.md` lists, at its pinned medium otherwise; a docs or harness-text diff takes `/code-review` inline. Fix blockers inline. A fix touching `ios/` reruns `just ios-test` plus `just ios-test-ui-class` for the class it touched (#1884); a fix with no UI test to name just reruns `just ios-test`.
4. **Deferred items**: an issue for each before the review comment.
5. **Open** with `just pr-open ... --draft` and the body template from the skill, and post the review summary naming the agent.
6. **Watch CI** to a conclusion in the same turn and read the PR's mergeability, not just the jobs. Reproduce a red UI test with `just ios-test-ui-class <Class>`, not the full tier.
7. **Ready** with `just pr-ready PR ISSUE` only once every check is green on the last commit and each review finding is fixed or answered in the PR. A screen change also waits for the owner's yes on its screenshot.

**Loop mode**, when the arguments say `loop`: a crew session (`.claude/commands/crew.md`), the same steps with five changes.

- Retro: post the lessons as one issue comment headed `**Lessons**` and add the `lessons-pending` label to the issue; never edit `docs/lessons.md`, so one run's drafts never conflict.
- After step 3, the `qa` agent, briefed with the worktree, the issue and the latest `test-runner` report. A failing criterion is fixed like a blocker, then the gates and `qa` run again.
- Before step 5, the park step on the diff: `git diff origin/main...HEAD > "$TMPDIR/crew-diff-N"`, then `scripts/crew-park.sh` on that file. Exit 3 (a sensitive path) or 2 (unreadable) parks the issue.
- Notes on the issue, each only on success: `**Crew: reviewed**` once step 3 leaves no blocker, `**Crew: QA**` with the `qa` report once every criterion passes, `**Crew: draft**` with the PR link once CI is green on the last commit. A failure is fixed and rerun, or parks the issue at its second failed round.
- Stop after step 6. Step 7 is the owner's, and the guards refuse it.

$ARGUMENTS
