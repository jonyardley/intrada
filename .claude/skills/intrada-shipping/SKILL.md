---
name: intrada-shipping
description: How a PR or issue ships in intrada: the pre-push gate and self-review order, draft until reviewed, Codecov expectations, the deferred-issue protocol, and the PR and issue body templates. Read before opening or updating any PR or writing an issue.
---

## The pre-push gate

**Every non-trivial PR goes through `/ship`**, which runs the gates and the
self-review together, in this order: local gates through the `test-runner`
agent, then the `reviewer` agent over the local diff, then triage and fix, then
push and open the PR with the review summary posted at creation. Review as a
separate step after the push is how it gets skipped: #1550 put four defects on
main while its reviewer was still thinking.

- **Who reviews.** `reviewer` runs on every diff that changes code or tests,
  Tier 1 included (#1665). A diff of docs, roadmap or harness text only takes
  `/code-review` inline.
- **A new or reshaped bridge type ships with a `LiveBridge` test** in
  `LibraryBridgeTests`, `SessionBridgeTests` or `VariationPlayBridgeTests`.
  A Rust round trip cannot see Rust and Swift disagreeing about the wire
  (#846, #1953); the reviewer blocks a bridge diff without one.
  `scripts/check-bridge-tests.sh` fails a new type or ViewModel field that no
  such test names (#2101); it sees a name, not an assertion, so the reviewer
  still reads what the test checks.
- **`reviewer` is pinned to Opus 5.5 high**, briefed with the worktree and the
  issue so it checks the diff against done looks like. No diff lifts the
  model: Fable was retired 2026-09-23 (#2044).

**The reviewer never waits, and the lead posts the comment.** Brief it to
report and yield: no `gh pr comment`, no holding for a PR number, since no PR
exists yet (2026-09-07: one review parked 35 minutes, one cancelled at its 45
minute limit, both on three-file diffs). The lead posts the summary verbatim at
PR creation, saying which agent produced it. While the review runs the lead
writes the PR body, opens the deferred issues and re-reads the diff; it does
not sit in a wait.

**Open every PR through `just pr-open "Title (#N)" "Body text" --draft`**, never
a bare `gh pr create`: it refuses an issue number in the title with no claim
naming this branch (#1702, the second way #1694 got built twice), and a Tier 1
or 2 body over the word limit below. Once the review comment is posted, its
blockers fixed and the deferred issues exist, `just pr-ready PR ISSUE` marks it
ready and moves the board to In review (#1659); a screen change also waits for
Jon's yes on its screenshot. The body's `Closes #N` closes the issue on merge,
and the board follows.

## Verifying a change on the simulator

A UI or interaction change is driven on the simulator before it ships, not
described: `just ios-run` launches the app and the runtime tools tap, type and
screenshot it. A snapshot proves a settled frame renders, not that a tap
reveals anything or that a rejected write puts the old value back. Name in
**What I checked** what you drove and what you saw.

`needs-device` is for behaviour that cannot exist on a simulator: camera,
haptics, gestures a synthetic touch cannot reproduce, background audio, Live
Activities. Those PRs carry the label and their **What I checked** names exactly
what a person has to do by hand. Reaching for it because driving the app looked
like effort is the failure this rule exists to stop.

A screen change ships with its before and after image, not prose describing
one, and the draft's after image is the one Jon approves
(`.claude/rules/ios-ui.md`: eyes before ready); a no is fixed by a push to the
open PR. After pushing the branch, `just pr-visuals` reads the
references it changed and prints the before/after markdown to paste under
**What it looks like** (#1631); the images are raw.githubusercontent.com URLs
pinned to the pushed commit, so run it after the push or they point at a
commit GitHub has never seen.

## Codecov (Tier 2+)

After CI, compare the patch-coverage comment with the **Coverage** line in the
PR description. Unexpected gaps get tests or a PR comment before the PR is
marked ready.

## Deferred issues

Every deferred or out-of-scope item becomes a tracked issue, labelled
(`horizon:now|next|later` plus a kind: `ux`, `architecture`, `bug`,
`accessibility`, `ios`, `pillar:*`), opened before the self-review comment is
posted. PR descriptions are not tracking. The comment ends with
`Deferred items tracked: #N, #M` or `none, all flagged items addressed inline`.

A deferred issue that belongs to the same body of work joins the PR's epic
with `just epic-add EPIC N`; one that belongs elsewhere joins that epic, and a
genuine one-off stays single (the Epics rules in `docs/how-the-work-runs.md`).

## PR bodies and issues

Both are read cold, months later, by someone deciding whether to merge and
working out what changed, or deciding what to build from the body alone. The
templates are `.github/pull_request_template.md` and
`.github/ISSUE_TEMPLATE/issue.md`; the headings there are the headings, not a
suggestion.

0. **Look at**: one line above the headings naming the decision or risk Jon
   should judge. Stating the feedback wanted best predicts acceptance and
   reviewer engagement (Pirouzkhah, Wurzel Gonçalves and Bacchelli, arXiv 2602.14611).
1. **What this fixes**: the situation the affected person would notice, in the
   first sentence. For a screen change that is the
   musician; for tooling or docs it is Jon or an agent, and the body says what
   they hit rather than that no musician is affected. It ends with one sentence
   on why this approach over the obvious other one. No paths, symbols or code; a `just` recipe name once,
   when the recipe is the outcome.
2. **Where this could bite** (Tier 3 and the sensitive surfaces only): residual
   risk in what ships, present tense, one paragraph per risk, each ending in
   its tracking issue or "deliberate, not tracked". A risk an earlier PR of the same feature stated gets its issue
   number, not a restatement. Never the branch's history. Name the fault class,
   not the fumble. Never spell out an exploitable gap in auth, tokens or user
   data on a public repo; say one exists and route the detail to Jon.
3. **What it looks like**: screen changes only, the `just pr-visuals` output.
   Deleted on a PR that touches no screen.
4. **What I checked**: the evidence each check produced. "Gates green" is one line
   with counts; the check that could have failed earns the space. The
   reviewer's findings, the defects fixed on the branch and why a check was
   worth running belong in the self-review comment, not here.
5. **What changed where** (Tier 3 and the sensitive surfaces only): at most five
   lines, the files worth opening first, each with why. Identifiers welcome; the diff lists the rest.

A Tier 1 or Tier 2 body runs at most 350 words before the checklist, comments
not counted: `just pr-open` refuses a longer one that has neither Tier 3
heading (`SKIP_BODY_CHECK=1` for a justified case), and warns on any sentence
over 25 words (`docs/style-guide.md`). Tier 3 and sensitive work may run
longer, and every paragraph past that is a distinct shipped risk or a check
that could have failed. The three longest
bodies of 2026-09 (#1612, #1628, #1601) each carried over 100 words of branch
history in **What I checked**, which is the failure this rule exists to stop.
Checklist lines that do not apply are deleted, not annotated.

Issues: what you would notice, why it matters, what to do, done looks like,
then where it lives. A bug's notice section gives the steps (the tap path, for a screen), what was
expected and what happened; measurements, logs and fuller reproduction steps
go last even when they are the reason the issue exists.

## Plan comments

A Tier 2 or Tier 3 plan goes on the issue before the first commit, about 150
words, under these headings. Jon approves it before the build, except for a
Tier 2 issue that is the next child of an epic whose body names it: that plan is posted and the
build starts, and Jon reads it with the PR, unless the work is on the
sensitive surfaces or the plan names a choice he has not made. The build and
the reviewer read it.

```markdown
**Done looks like**: two or three checks, as the musician or Jon sees them.
**Time budget**: one session, or two; past it, stop and re-plan.
**Decisions taken**: each choice, with the option rejected and why.
**Files**: the files and lines it touches.
**Tests**: what proves it, and the line a deletion should break.
**Rabbit holes**: the risk most likely to eat the budget, and how it is avoided.
**Out of scope**: what it deliberately leaves alone.
**Routing**: the epic from `just claim`, sensitive surfaces or not, where it
runs, and any parallel stream.
```

The routing line follows the rules in `docs/working-with-agents.md`. A plan that
needs more than two sessions, or runs well past 150 words, is Tier 3.

Any term not in the glossary (`docs/reference.md`) is said plainly or added to
it. A phase letter, decision number or internal name ("Phase R", "T22", "the
lane") carries the document it resolves in, the first time it appears in a
body. **Tier** means ceremony level; the test gates are "the fast tier" and
"the full tier".

No tick marks, symbols or emoji in either, with one exemption: the attribution
footer the harness requires a session to append.
