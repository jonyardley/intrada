---
name: intrada-shipping
description: The route from finished code to merge in intrada, in order (retro when something went wrong, gates, review, draft, CI, ready, the merge wait), with the PR, issue and plan comment templates. Read before opening or updating any PR or writing an issue.
---

## The route

Every PR follows these steps in order, whatever order a brief gives (#2921).
Point at a step by its name, never its number, which goes stale when the
route changes.
Each spawn names its model and effort from the role table in
`~/.claude/CLAUDE.md`; rounds, and who fixes findings, follow the global
review rule there.

1. **Retro, only when something went wrong** that no test or check caught,
   once the code is done and before the gates, so any lesson and its fix are
   checked, reviewed and built with the code (#2962): one lesson per miss,
   given an outcome (below), committed. First read the session's row in
   `just usage 1 --quality`, found by the first eight characters of the
   session id (the folder name in the scratchpad path), not by branch: a `cor`
   of two or more, a `peak` past 200k, or a model or effort that did not fit
   the work counts as something that went wrong, and its lesson names what to
   do differently. The row covers the whole session over the last day, so it
   is a rough signal; no row means the session was not among the costliest,
   and corrections are counted by hand. Judge the fit by the work, not only by
   the role table: when the table itself looks wrong, the lesson says which
   row and why. Anything is in scope for a lesson, the role table, global
   rules, guards and prompts included. Nothing went wrong, nothing is
   written; `just pr-ready` needs no lessons commit (#3034).
2. **Gates**, through `test-runner`: `just pre-review`, plus `just ios-test`
   when `ios/` changed, plus `just ios-test-ui-class <Class>` for each UI test
   class the diff adds or edits, and for each class that drives a form the
   diff reshapes (#2993); CI runs the full UI tier (#2114). Stop at the
   first failure. `test-runner` cannot edit, so the lead runs a mutation
   check itself (#2519).
3. **Review** before the push, because one left until after gets skipped
   (#1550). `just review-route` decides: `reviewer` on code or tests, Tier 1
   included, launched from the builder's worktree so it can run tests (#2960),
   briefed with that worktree's absolute path and the issue (a session
   started outside the repo has no project agent types: spawn a
   general-purpose agent told to follow `.claude/agents/reviewer.md`); a second
   `reviewer` paired from the core diff on a sensitive diff, and on auth, which
   is judged by hand; `/code-review` inline on agent instructions, saying
   "comment-policy violations are Blockers"; none on documentation only. A fix
   touching `ios/` reruns `just ios-test` and the UI class it touched (#1884).
   Read a fix agent's diff before the re-review; its report is not the diff.
   A new or reshaped bridge type ships with a `LiveBridge` test (#846, #1953);
   `scripts/check-bridge-tests.sh` checks one names it (#2101), and the
   reviewer reads the assertion.
4. **Out-of-scope findings** are fixed here, or listed under "Not fixed" in
   the review comment with one line of why. Only one with horizon now or next
   becomes an issue, through `just issue-open` and `just epic-add EPIC N`, run
   from a worktree, since the main checkout refuses them as scripts. The
   review comment ends `Deferred items tracked: #N` or `none`.
5. **Open** a draft with `just pr-open` once the branch is pushed (it
   refuses an unpushed branch), never a bare `gh pr create`, with the
   body below, written to the scratchpad with the Write tool and passed as
   text, `"$(cat <scratchpad>/body.md)"`; the review summary goes on verbatim
   as the first comment, naming the agent.
6. **Watch CI** to a conclusion in the same turn and read the PR's
   mergeability. Reproduce a red UI test with `just ios-test-ui-class`. Compare
   Codecov's patch comment with the body's Coverage line; an unexpected gap
   gets a test or a PR comment.
7. **Ready**: `just pr-ready <pr> <issue>` refuses until the route is done. A
   screen change also waits for the owner's yes on its pictures.
8. **The merge wait**, in the background: poll every three minutes for up to
   an hour, `for i in $(seq 20); do s=$(gh pr view PR --json state -q
   .state); [ "$s" != OPEN ] && break; sleep 180; done; echo "$s"`
   (`gh event listen` fails with HTTP 403 on this account). A lesson found
   after the retro (gates, review, CI or the wait) is posted on the PR as a
   comment when found, first line "Late lesson", so it survives a session
   that ends early (#2968). Merged: write the handover for the next issue
   from the main checkout (`just handover N` reads the current branch),
   copying the PR's late lessons for the next retro, then
   `just worktree-done`.
   Closed unmerged: stop and say so. Timed out: read the PR state first on
   the owner's next message. Never ask the owner to say merged.

**Retro outcomes** (#2934). After checking for a repeat as the lessons header
says, each lesson gets one of two outcomes, and either one reshapes before it
adds: if a test or check already caught the mistake, nothing new is written;
otherwise sharpen, merge or remove an existing line before adding one, so the
rules stay short enough to be read. A lesson seen a second time (its
line reaches "(seen again 1)") always takes Issue, and that issue first
changes what the session saw when it went wrong (the message, refusal or
recipe output) so it names the right move; any doc line comes after (#2967):

- **Fixed here**: a prose change of about 30 lines or fewer to the doc, rule,
  skill, agent brief or command that would have prevented it, in its own
  commit, named on the body's Process fixes line. The lesson goes straight to
  the archive with "fix:" and the file in front.
- **Issue** under the harness epic #1983: anything bigger, and any change to a
  script, recipe, hook, settings, CI workflow, `CLAUDE.md`, app code,
  frontmatter or a sensitive surface path. The lesson stays under Lessons
  ending "(issue #N)" while the issue is open. Its What to do comes from
  rerunning the failing command or reading the saved output, not from memory,
  and Where it lives says when the file is machine-local, outside the repo:
  #2997 blamed a script that already read crashes, and #3021 a `/dev/null`
  refusal that never fired.

## Verifying a change on the simulator

Every feature ships in one PR, core commits first: a merged core with no
caller is shell-dead (#1374). On the sensitive surfaces `reviewer` reads the
core commits alone before the screens are added.
Brief a `task` spawn on a bridge change for the core commits only, then a
second spawn for the screens: `task` stops after the core and runs no gates
(`.claude/agents/task.md`), so a whole-stack brief comes back half built (#2788).

Drive a UI change on iPhone (`just ios-run`, then the runtime tools) and say
in What I checked what you drove and saw: a snapshot proves a frame renders,
not that a tap works. `needs-device` is only for what a simulator cannot do
(camera, haptics, some gestures, background audio, Live Activities), and
names what a person does by hand.

**Eyes before ready.** A screen change carries iPhone before and after images
from `just pr-visuals`, run after the push since its URLs pin the pushed
commit (#1631), and the line "Waiting for the owner's word on the
screenshot", which becomes "Approved on YYYY-MM-DD" on a yes; a no is a push
to the open PR (#1891). A change no snapshot shows has nothing for
`pr-visuals`: capture the screens with a throwaway UI test, and take the
"before" app code from `git merge-base origin/main HEAD`, since `origin/main`
moves under you when another session fetches (#2519).

## PR bodies and issues

Both are read cold, months later; the headings in
`.github/pull_request_template.md` and `.github/ISSUE_TEMPLATE/issue.md` are
fixed.

- **Look at**: one line naming the decision or risk the owner should judge;
  on Tier 3 or a sensitive surface, each residual risk ending in its issue or
  "deliberate, not tracked", never spelling out an exploitable gap. Under it,
  **Process fixes**: each retro fix with its lesson, or "none", the usual case.
- **What this fixes**: what the affected person notices, in the first
  sentence, then one sentence on why this approach over the obvious other. No
  paths, symbols or code.
- **What I checked**: the evidence each check produced, the screen pictures
  included; "gates green" is one line with counts, and the check that could
  have failed earns the space. Coverage names the expected gaps (Tier 2 and
  up). Reviewer findings go in the review comment.

A Tier 1 or 2 body runs at most 350 words before the checklist; `just
pr-open` enforces it (`SKIP_BODY_CHECK=1` for a justified case). Checklist
lines that do not apply are deleted.

Issues: what you would notice, why it matters, what to do, done looks like,
where it lives. A bug gives the steps, what was expected and what happened,
the build and the device; logs and measurements go last.

## Plan comments

A Tier 2 or 3 plan goes on the issue before the first commit, about 150
words, and the owner approves it before the build. A Tier 2 issue that is the
next child of an epic whose body names it posts its plan and starts, unless
it is on the sensitive surfaces or names a choice the owner has not made.

```markdown
**Done looks like**: two or three checks, as the musician or the owner sees them.
**Time budget**: one session, or two; past it, stop and re-plan.
**Decisions taken**: each choice, with the option rejected and why; "storage unchanged" names the writes checked, since a new default is a stored value (#2788).
**Files**: the files and lines it touches, core and `ios/`.
**Tests**: what proves it, and the line a deletion should break.
**Sketch**: for a new screen, the SwiftUI sketch page and the owner's word on it, or why none is needed.
**Rabbit holes**: the risk most likely to eat the budget, and how it is avoided.
**Out of scope**: what it deliberately leaves alone.
**Routing**: the epic from `just claim`, sensitive surfaces or not, and any
parallel stream.
```

A plan needing more than two sessions, or well past 150 words, is Tier 3. A
term outside the glossary (`docs/reference.md`) is said plainly or added; an
internal name carries the document it resolves in on first use. **Tier**
means ceremony level; the test gates are "the fast tier" and "the full tier".
No tick marks, symbols or emoji, except the harness's attribution footer.
