# The whole-app audit

*How intrada checks, as a whole, that it is still simple, true to its
principles, secure, consistent, well designed and able to cope with years of
practice. Per-PR review asks whether a change is right; this asks whether the
thing the changes joined is still right. Set up in #2092 from the method of the
September audit (`docs/audit-2026-09.md`, #1719). Every run uses this page, so
one run can be compared with the next.*

## When it runs

- **The sweep, at every release.** When a milestone is cut (`docs/roadmap.md`,
  Cutting a release), run `just audit-sweep` and commit the file it writes with
  the release. It takes about a minute and needs no agents.
- **The full audit, at the end of each phase or every quarter,** whichever
  comes first, and never alongside in-flight branches on the same files. Its
  cost is about fifteen agents, the same order as September's run: say so to
  Jon before starting and let him choose.
- **A targeted run** takes one or more lane groups (below) when one area has
  changed a lot, for example after a bridge rework.

## The three layers

1. **Sweep** (`scripts/audit-sweep.sh`): counts that need no judgement, written
   to `docs/audit-metrics/<date>.json` and compared with the newest earlier
   file. A probe whose tool is missing records null, never zero.
2. **Review** (`.claude/workflows/audit.js`): one agent per lane group against
   the questions below. After all of them finish, the proposed mutations are run
   one at a time; then one verifier per group tries to refute every finding.
   A synthesis writes the report. A follow-up agent re-checks the last audit's
   issues: fixed, still open, or back again.
3. **Walk**: the design group reads the committed snapshot images and a seeded
   and an empty `just ios-run` screenshot of each tab. Then Jon spends 30
   minutes on the device with the checklist below, and his findings are added
   to the report before its PR merges.

**The ratchet.** Any finding a script could catch is filed as a gate for
`just check` (and CI), not as something to find again next time. Anything
already gated leaves the audit. Over time the audit shrinks to the questions
that need judgement.

## The lanes

Twelve lanes in six groups; one agent reviews each group. Every finding answers
one of its lane's questions, or it is out of scope. Sweep metrics are named by
their key in the metrics file.

### Group: boundary

**Principles.** Does the core own every domain decision, and is the shell still
a dumb pipe?

1. Does any Swift make a domain decision, validate, or hold domain state beyond
   UI interaction state?
2. Does every wire variant and view field have a reader on the other side?
3. Is anything cached in Swift that the core already works out?
4. Do the non-negotiables in `CLAUDE.md` still describe the code?

Sweep: `principles.*`.

**Simplicity.** Is there one way to do each thing?

1. Where are there two ways of doing the same thing that each look fine alone?
2. Which code has no reader (#1176), and which helpers duplicate each other?
3. Are the hotspots (largest and most churned files) big because they must be?

Sweep: `simplicity.*`, `activity.*`, and the `hotspots_*` and `churn` lists.

### Group: data

**Data integrity.** Can the musician's record be lost or corrupted without
anyone being told?

1. Where does a failed write go quiet: the offline path, the bridge, the GRDB
   migrations, the crash-recovery blob?
2. Does every migration replay on a database from each earlier release?
3. Do the timestamps and the last-write-wins merge agree between shell and
   core?
4. Does every field inside the crash-recovery blob graph have a version bump
   and a pin (#1345)?

Sweep: `data.*`.

**Resilience.** What happens when the world misbehaves?

1. The app killed mid-item, mid-save, mid-migration: what survives?
2. A full disk, a denied permission, a time zone or clock change across
   midnight (the UTC day-boundary class, #1357).
3. There is no sync: what survives a lost or replaced phone, and does the
   musician know?

### Group: tests

**Tests that bite.** Do the tests fail when the behaviour goes away?

1. Which behaviours named in the specs have no test that would fail without
   them? Propose mutations to prove it: deleting a line, never inverting it
   (#1423), each with the test you expect to fail. Include two controls a
   known test should catch.
2. Which tests assert a value that was the same one line earlier (#1223)?
3. Does each screen have its snapshot, and are the fixtures real (#1256)?

Sweep: `tests.*`.

### Group: screens

**Design.** Does each screen meet the quality bar
(`.claude/rules/ios-quality.md`) and the journey (`docs/journeys.md`)?

1. Does each screen have its empty, loading and error states, and do they say
   what to do next?
2. Can each of the ten journey steps be done without a dead end?
3. Does anything on screen contradict `docs/design-principles.md`?

**Consistency.** Does the app look and read like one app?

1. Is every colour, font, spacing and radius a token from `Theme.swift`?
2. Is one pattern used for one job: sheets, confirmations, pills (only for
   inputs), destructive actions?
3. Does the copy match `docs/tone-of-voice.md`, in British English, with no
   dashes?

Sweep: `consistency.*`.

**Accessibility.** Can every musician use it?

1. Does every control have a VoiceOver label that says what it does?
2. Does every screen hold together at the largest Dynamic Type size?
3. Contrast of every token pair actually drawn together, Reduce Motion, and
   tap targets of at least 44 points.

Sweep: `accessibility.*`.

### Group: safety

**Security and privacy.** What leaves the device, and what is exposed on it?

1. What is sent off the device (crash reports, analytics), and does the
   privacy manifest and App Store label say so?
2. Is the database file protected while the phone is locked, and is anything
   sensitive in UserDefaults or logs?
3. Are the entitlements and permissions the least the app needs?
4. Has any secret ever been committed? Never spell out an exploitable gap in a
   public report: name it and route the detail to Jon.

Sweep: `security.*`.

**Scale.** Does it still work after five years of daily practice?

1. With about 2,000 sessions and 500 items seeded: how long do launch, the
   Library, the Progress tab and a tap in the player take?
2. How big is the view model that crosses the bridge on each render, and does
   it grow with the library?
3. Do the database queries have the indexes they need?

### Group: upkeep

**Dependencies and toolchain.** What are we standing on?

1. Which crates and Swift packages are behind, unmaintained, or carry an
   advisory? Report only; never bump inside an audit.
2. How much depends on beta Xcode or iOS, and what breaks when that moves?
3. Licences: anything incompatible with shipping on the App Store?

Sweep: `dependencies.*`.

**Drift.** Do the docs, specs and harness still describe what exists?

1. Which claims in `CLAUDE.md`, `.claude/rules/` and the specs are no longer
   true?
2. Which specs describe something that did not ship, or shipped differently?
3. Which hooks, recipes or agents cost more than they catch?

Sweep: `drift.*`.

## Ratings

Each lane gets one rating, plus its trend against the last audit's rating.

- **Red:** any band 1 finding, or a security finding that exposes the
  musician's data.
- **Amber:** any band 2 finding; or a sweep metric that got worse with no reason
  given; or a finding from the last audit in this lane still open past its
  phase.
- **Green:** otherwise.

## Bands

Findings are ordered by cost to the musician, then by likelihood, the same
bands as September:

1. Silent loss or corruption of the musician's data.
2. Wrong on screen: the musician sees something untrue, or cannot do a step.
3. Engineering drag: a cost every future change pays.
4. Cosmetic.

## Running a full audit

1. Make a worktree at a fixed commit (`just worktree-new audit-YYYY-MM`) and
   run `just audit-sweep` there.
2. Tell Jon the cost and wait for his go.
3. Run the saved workflow (below).
4. Read the report. Make sure every finding has been verified, and every
   mutation has been reverted (`git status` clean).
5. File one issue per finding under a new `epic`-labelled parent
   (`just epic-add`), and one per gate the ratchet names; re-rank existing
   issues with a comment rather than filing them again.
6. Jon does the walk; add his findings to the report.
7. Open the PR with the report and the metrics file. When the report and the
   issues disagree, the issues are right.

Say "run the audit workflow", which counts as the opt-in, with these
arguments:

```json
{
  "worktree": "/Users/jonyardley/Dev/intrada-worktrees/audit-2026-12",
  "commit": "abc1234",
  "date": "2026-12-01",
  "sweep": "docs/audit-metrics/2026-12-01.json",
  "previousReport": "docs/audit-2026-09.md",
  "previousEpic": 1967,
  "groups": ["boundary", "data", "tests", "screens", "safety", "upkeep"]
}
```

Leave out `groups` for a full audit; name some for a targeted run.

## The walk checklist

Jon, about 30 minutes, on a TestFlight build of the audited commit:

1. Delete and reinstall. Add a first piece and a first exercise from nothing.
2. Build a session, play it through with the timer and reps, score it. Force
   quit mid-item and reopen.
3. Turn on the largest text size and VoiceOver, and do step 2 again for one
   item.
4. Switch to dark mode and airplane mode. Read each tab's words aloud against
   the tone of voice.
5. Note anything that surprised you, and where you hesitated.

## The report

`docs/audit-YYYY-MM.md`, with these headings in this order:

- The italic header: commit, date, epic, and "when this document and the issues
  disagree, the issues are right".
- **How it ran:** groups, models, what was verified and how, and what was
  deliberately not done.
- **Scorecard:** a table with one row per lane: rating, last rating, trend, and
  one line of why.
- **Sweep:** the metrics that changed since the last audit, with a reason for
  each that got worse.
- **Last audit's findings:** fixed, still open, or back again, one line each.
- **Mutations:** what was deleted, what was expected, and what happened.
- **Findings, by cost:** by band, each with the question it answers, how it was
  verified, the cost, the fix, the tier, where it is, and its issue.
- **Gates to add:** the ratchet's list.
- **Build order**, **Decisions for Jon**, **Dropped**, **Not covered**, and
  **The walk**.

## Changing the rubric

Change a lane's questions only between audits, in its own PR, so that a trend
always compares like with like. A new sweep metric starts its trend from the
next sweep; a removed one says why in the PR.
