---
name: intrada-shipping
description: How a PR or issue ships in intrada: the pre-push gate and self-review order, draft until reviewed, Codecov expectations, the deferred-issue protocol, and the PR and issue body templates. Read before opening or updating any PR or writing an issue.
---

## The pre-push gate

**Every non-trivial PR goes through `/ship`**, which owns the order. Review
runs before the push, because a review left until after it gets skipped
(#1550).

- **Who reviews.** `reviewer`, on every diff that changes code or tests, Tier
  1 included (#1665), briefed with the worktree and the issue. A docs, roadmap
  or harness-text diff takes `/code-review` inline. No diff lifts the model
  (#2044).
- **A new or reshaped bridge type ships with a `LiveBridge` test**
  (`LibraryBridgeTests`, `SessionBridgeTests` or `VariationPlayBridgeTests`):
  a Rust round trip cannot see Rust and Swift disagree about the wire (#846,
  #1953); the reviewer blocks a bridge diff without one.
  `scripts/check-bridge-tests.sh` checks that a test names each new
  type or ViewModel field (#2101); it cannot read the assertion, so the
  reviewer does.
- **The lead posts the review.** The reviewer reports and yields
  (`.claude/agents/reviewer.md`); the lead posts its summary verbatim at PR
  creation, naming the agent, and writes the body and deferred issues while
  it runs.
- **Open through `just pr-open "Title (#N)" "Body" --draft`**, never a bare
  `gh pr create`: it refuses an issue number with no claim naming this branch
  (#1702). Once the review is posted, its blockers fixed and the deferred
  issues open, `just pr-ready PR ISSUE` marks it ready and moves the board
  (#1659). `Closes #N` in the body closes the issue on merge.

## Verifying a change on the simulator and the emulator

Every feature ships on both apps in one PR, core commits first, because a
merged core with no caller is shell-dead (#1374). On the sensitive surfaces
`reviewer` reads the core commits alone before the screens are added.

Drive a UI or interaction change on both apps (`just ios-run`,
`just android-run`, then the runtime tools) and name in **What I checked**
what you drove and saw: a snapshot proves a frame renders, not that a tap
works or a rejected write restores the old value.

`needs-device` is only for what a simulator cannot do: camera, haptics,
gestures a synthetic touch cannot reproduce, background audio, Live
Activities. Its **What I checked** names exactly what a person does by hand.
It is never a way to skip driving the app.

**Eyes before ready.** A screen change opens as a draft with iPhone and
Android before and after images and the line "Waiting for Jon's word on the
screenshot". It goes ready only after Jon's yes, when that line becomes "Jon
approved the screenshot on YYYY-MM-DD"; a no is a push to the open PR
(#1891). Run `just pr-visuals` after the push (#1631): its image URLs pin the
pushed commit, so an earlier run points at a commit GitHub has never seen.

## Codecov (Tier 2+)

The PR's **Coverage** line names expected gaps before CI finishes (70% patch
target, informational; `ios/` ignored). After CI, compare the patch-coverage
comment with it. An unexpected gap gets a test or a PR comment before ready.

## Deferred issues

Every deferred or out-of-scope item becomes an issue before the self-review
comment is posted. Open it with `just issue-open`, which refuses one without
a horizon and a kind label;
a PR description is not tracking. The comment ends `Deferred items tracked:
#N, #M` or `none, all flagged items addressed inline`. A deferred issue joins
its epic with `just epic-add EPIC N`; a genuine one-off stays single
(`docs/how-the-work-runs.md`, Epics).

## PR bodies and issues

Both are read cold, months later. The headings in
`.github/pull_request_template.md` and `.github/ISSUE_TEMPLATE/issue.md` are
fixed.

0. **Look at**: one line above the headings naming the decision or risk Jon
   should judge.
1. **What this fixes**: what the affected person notices, in the first
   sentence (the musician, or Jon or an agent for tooling, saying what they
   hit rather than that no musician is affected), then one sentence
   on why this approach over the obvious other. No paths, symbols or code; a
   `just` recipe once, when it is the outcome.
2. **Where this could bite** (Tier 3 and sensitive surfaces only): one
   present-tense paragraph per residual risk, ending in its issue or
   "deliberate, not tracked"; a risk an earlier PR stated gets its number.
   Name the fault class, never the branch's history. Never spell out an
   exploitable gap in auth, tokens or user data; say one exists and route the
   detail to Jon.
3. **What it looks like**: the `just pr-visuals` output; deleted when no
   screen changed.
4. **What I checked**: the evidence each check produced. "Gates green" is one
   line with counts; the check that could have failed earns the space.
   Reviewer findings and branch fixes go in the self-review comment.
5. **What changed where** (Tier 3 and sensitive surfaces only): at most five
   files worth opening first, each with why; identifiers welcome.

A Tier 1 or 2 body runs at most 350 words before the checklist, comments not
counted; `just pr-open` enforces it (`SKIP_BODY_CHECK=1` for a justified
case) and warns on a sentence over 25 words. In a longer Tier 3 or sensitive
body, each extra paragraph is a shipped risk or a check that could have
failed. Checklist lines that do not apply are deleted.

Issues: what you would notice, why it matters, what to do, done looks like,
where it lives. A bug gives the steps (the tap path, for a screen), what was
expected and what happened. It names the app (iPhone, Android or both), the
build and the device. Logs and measurements go last.

## Plan comments

A Tier 2 or 3 plan goes on the issue before the first commit, about 150
words, and Jon approves it before the build. A Tier 2 issue that is the next
child of an epic whose body names it posts its plan and starts, and Jon reads
the plan with the PR, unless it is on the sensitive surfaces or names a
choice he has not made. The build and the reviewer read it.

```markdown
**Done looks like**: two or three checks, as the musician or Jon sees them.
**Time budget**: one session, or two; past it, stop and re-plan.
**Decisions taken**: each choice, with the option rejected and why.
**Files**: the files and lines it touches, core, `ios/` and `android/`.
**Tests**: what proves it on each shell, and the line a deletion should break.
**Rabbit holes**: the risk most likely to eat the budget, and how it is avoided.
**Out of scope**: what it deliberately leaves alone.
**Routing**: the epic from `just claim`, sensitive surfaces or not, where it
runs on iOS and Android, and any parallel stream.
```

Routing follows `docs/working-with-agents.md`. A plan needing more than two
sessions, or well past 150 words, is Tier 3.

A term outside the glossary (`docs/reference.md`) is said plainly or added.
An internal name ("Phase R", "T22", "the lane") carries the document it
resolves in on first use. **Tier** means ceremony level; the test gates are
"the fast tier" and "the full tier". No tick marks, symbols or emoji, except
the harness's attribution footer.
