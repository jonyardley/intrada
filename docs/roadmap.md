# intrada: Product Roadmap

*The direction and the ranking of the work. What is in flight right now:
`just status` (one screen, read from GitHub), the
[project board](https://github.com/users/jonyardley/projects/2) and the
[open issues](https://github.com/jonyardley/intrada/issues). The released
build and the phase: [`where-we-are.md`](where-we-are.md).*

---

## Direction

intrada is a **practice notebook**: the musician plans, the app remembers,
keeps the record honest and nudges ([`VISION.md`](../VISION.md), v3,
2026-09-30). The 2026-07 practice coach, which planned for the musician, was
removed on 2026-08-13 (#1344); the notebook is what every release since has
built on. A suggested session stays on the far horizon, as a proposal the
musician can ignore, never a plan the app imposes.

**The capture line is delivered.** The direction chosen on 2026-09-07 shipped
through v0.10.0 to v0.14.0: one-pass create, a piece from a photo, keys and
variations, the Focus Player and its click, a tempo recorded only on evidence,
and practice defaults. **The next headline is a first run that gets a new
musician to their first marked session (#2121)**, because so far the only
regular user is the person building the app, and every later call needs
evidence from others.

---

## The ranking

Work is picked from epics (see [Epics](#epics)), ranked here directly. The
five layers and three pillars in the vision explain the product; they are not
a ranking rule. When this list and an epic's horizon label disagree, this list
is the intent and the label is the one to fix.

### Now: the next release's headline

- **#2121 A first run that gets a new musician to their first marked
  session.** Ends by watching three new musicians try it (#2120).
- Finishing, not new: the September audit backlog (#1967) and the core
  maintainability epic (#1995), each with a handful of children left.

### Next: the product, in order

1. **#1926 A week's practice set with intent.** The week's intent replaces
   the goals feature (Open question 5); starts from the lesson trial (#1927)
   and its research note (#1928).
2. **#2134 Every screen holds together at the largest text sizes.**
3. **#1975 Play the session through**, led by session start feeling laggy
   (#1801).
4. **#1974 Build a session your way.**
5. **#1970 Practise an exercise in its keys.**
6. **#1972 Add a piece in one pass** and **#1973 the Library**: refinement
   of the capture line, pulled when use shows a gap.

### Later

- **#1978 Know what to practise next**: priorities and spacing. Waits on the
  week's intent, which is what it reads.
- **#1977 See the practice working** and **#1976 Score how it went.**
- **#1979 One look everywhere** and **#1980 Make it yours, part two.**
- **#2025 Chord charts**: parked until the design is reopened.

### Alongside: tooling

The build, the test gates and the agent harness run beside the product and
are not ranked against it: **#1982** a build a tester can trust (it matters
more once testers arrive with #2121), **#1981** the iOS test gates, **#1983**
the agent harness and **#1849** Claude usage.

### Horizons

`horizon:now`, `:next` and `:later` stay on each issue for filtering:

- **Now**: a real musician hits a wall *because* this isn't built.
- **Next**: a real musician notices it's missing after a month of use.
- **Later**: the app could live without it for a year.

---

## Three pillars

Filters across the epics, not a ranking.

- **Plan**, deciding what to practise, before the instrument comes out:
  [`pillar:plan`](https://github.com/jonyardley/intrada/labels/pillar%3Aplan).
- **Practice**, playing with intention while the timer runs; the app stays
  out of the way:
  [`pillar:practice`](https://github.com/jonyardley/intrada/labels/pillar%3Apractice).
- **Track**, seeing the practice working afterwards:
  [`pillar:track`](https://github.com/jonyardley/intrada/labels/pillar%3Atrack).

---

## Mobile shell — settled

The native SwiftUI app on the shared Crux core **is** the shell, and has been
since the 2026-05-31 decision completed: app-first, **local-first** (on-device
SQLite is the source of truth; the API server that once carried sync was
removed in #1746, and any future sync starts from a new spec, not by reviving
it). The Tauri 2 + Leptos WKWebView host and the Leptos web app were
deleted in #1133; two pieces of Swift worth reusing were mined into
`ios/Reference/` first — the background-audio session handling and a Live
Activity implementation. That reference Swift was itself removed in #1745,
recoverable from git history.

See [`specs/native-ios.md`](../specs/native-ios.md) for the shell spec. Work
tracks under [`ios`](https://github.com/jonyardley/intrada/labels/ios).

Android is a later-horizon second native shell on the same core, specced in
[`specs/android-shell.md`](../specs/android-shell.md) (#1774); no build phase
has started.

Historical note: the sentences that used to live here ("the web app stays on
Leptos, untouched", "the Tauri shell stays shipping until parity") were true in
May and false by July. Statements about what is *currently* shipping belong in
one place: [`where-we-are.md`](where-we-are.md).

---

## Delivery model

### Labels

| Label | Purpose |
|-------|---------|
| `pillar:plan` / `pillar:practice` / `pillar:track` | Activity pillar |
| `layer:capture` / `:plan` / `:space` / `:show` / `:guide` | Vision layer |
| `horizon:now` / `:next` / `:later` | Rough timing — 4 wk / 4–12 wk / 12+ wk |
| `architecture` | Technical debt, infrastructure |
| `ux` / `accessibility` | Cross-cutting |
| `security` | Security-relevant |
| `ios` | iOS-specific (native SwiftUI shell) |
| `epic` | A body of work whose issues are its sub-issues ([Epics](#epics)) |

### Epics

An epic is the container for a body of work: a roadmap line, an audit's
backlog, a spec's phases, a tooling programme. The next slice is picked from
one, not from the flat list of open issues: `just status` shows every open
epic with its done count and its next open, unclaimed child.

- **What earns one**: three or more issues that ship as separate PRs and have
  an order worth writing down. A pillar, layer or journey label is a filter,
  not an epic. Loose tidy-ups with no order between them stay single issues.
- **Membership is a GitHub sub-issue**, attached with
  `just epic-add EPIC N...` in working order; `just epic-move` takes an issue
  from the epic that holds it. A checklist in the body is not membership. The
  parent carries the `epic` label, and no epic sits under another.
- **One parent per issue**, which GitHub enforces. Where two epics fit, the
  one that says when to build it wins and the other names it in prose. An
  audit epic keeps its findings until it closes.
- **The body** follows the issue template, and What to do names the working
  order, which the sub-issue list mirrors. The title names the outcome, with
  no "Epic:" prefix; the label says it. #1967 is the model.
- **Horizon stays on each issue.** The epic carries the nearest of its
  children's, so filtering on a horizon still finds it.
- **Closing**: the session that closes the last child closes the epic in the
  same turn, after checking the body for work that never became an issue. An
  epic never closes with children open; they move or close first. When the
  direction changes, Jon closes it with a comment naming where the rest went.

### Board

[Project board](https://github.com/users/jonyardley/projects/2) columns
are workflow states (Backlog → Ready → In Progress → In Review → Done),
not categories. Use the labels above to filter and slice.

**Ready is the pull queue, not the release.** An issue earns Ready when
someone could start it without asking a question first: the shape is
settled and what it points at has been read. It says nothing about which
build it lands in.

**The milestone is the release.** Everything meant for the next cut carries
that release's milestone, which the board already holds as a field and
filters on. Keeping the two apart is the point: an issue can be ready to
start and deliberately out of the cut, or in the cut and not yet
understood, and neither has to lie about the other.

A `Priority` custom field (P0/P1/P2) ranks within a pillar/layer when
multiple items share the same horizon.

### Cutting a release

A milestone is one headline plus whatever rides along. **Cut when the
headline works on the phone**, not on a day of the week: the work here is
sporadic, so a calendar trigger fires on the empty weeks and misses the
busy ones. v0.9.0 was the photo release and v0.10.0 was capture, both cut
that way before it was written down.

Write the headline on the milestone description's first line. `just status`
reads it from there, so a milestone nobody described says so rather than
looking like a release with no work in it.

Whatever is still open in the milestone at that point rolls to the next
one. If the headline keeps growing, cut anyway and rename the milestone: a
version number costs nothing pre beta.

**Getting a build onto a device does not need a release.** `gh workflow run
release-testflight.yml --ref <branch>` puts a signed build on TestFlight
from any branch in about 15 minutes. It carries no Sentry release, because
that step is tag only, and it takes the version sitting in
`ios/project.yml`, so its crashes arrive unattributed and it lands beside
the released build under the same version string.

**A release's write-up is its GitHub release**, generated from the
milestone. [`where-we-are.md`](where-we-are.md) changes when the phase
changes, not when the release does.

---

## Open questions

These are unresolved product questions. Each one likely produces issues
(or a Tier-3 spec) once answered.

1. **Metronome (answered 2026-08-14).** The builder product does want one:
   tracked as #1366 (audit Phase 4), with tempo as a tracked unit of
   measure. The coach-era click (removed in #1344, recoverable from
   history) and the `ios/Reference/` audio-session Swift (removed in
   #1745, also recoverable from history) are the starting points.

2. **Offline-first architecture (resolved 2026-07).** The native app is
   offline-first by design — on-device SQLite is the source of truth, with
   test-enforced invariants (see CLAUDE.md). #41 closed. What remains is the
   future paid sync tier, deliberately deferred.

3. **Scoring + tempo coupling (answered 2026-08-27).** The question as
   written (must every mark carry a tempo, or is tempo optional for items
   with targets?) was overtaken by the code: tempo capture shipped inside
   the metronome work (#1398, #1401) and made the stepper appear on every
   item, always saving, pre-filled from the click's tempo even when the
   click never sounded. So every mark already carries a tempo and many of
   those numbers are an untouched default.

   **A tempo is recorded when there is evidence behind it, and never
   otherwise** (Jon): either the user moved the stepper, or the click was
   sounding, so the number measures what they actually played to. Tempo is
   neither required nor optional-by-target; it is recorded when it was
   measured. This matters for the tempo trend (#1420), which draws the
   history as a chart, and a chart looks like measurement.

   Costs no new field: `UpdateEntryTempo` already carries `Option<u16>` and
   `SetlistEntry.achieved_tempo` already skips `None`. Keep it that way, because
   `SetlistEntry` sits inside the `ActiveSession` crash-recovery blob. The
   ViewModel projection is the opposite: `tempo_trend` deliberately carries a
   slot for every practised session, so the chart can break its line where
   nothing was measured.

   **Implemented 2026-08-28** (#1420, the first of its three PRs) as design-principles
   **T16**. One correction to the framing above: the untouched pre-fill is not
   always the neutral 96. `ClickController` seeds from the item's own declared
   target, so for an item marked ♩ = 132 the app was recording 132 as achieved. The shell forwards two observed facts (`TempoObservation`) and the
   core rules on them; no new field, no crash-recovery key bump. Pre-fix
   history on device is accepted rather than migrated away: there was
   essentially none, so nulling it would have been destructive work with
   nothing to show for it.

4. **Teacher integration timing.** Currently a Layer-5 horizon. Basic
   sharing (routines, item suggestions) could come earlier without AI.
   The teacher-assignment capture (#267) addressed the immediate capture
   problem without teacher-facing features.

5. **Goals (superseded 2026-09-30 by the week's intent).** Goals are not
   rebuilt. What the ruling below wanted, a named aim that planning reads,
   becomes the week's intent: one sentence, possibly from a lesson, linked to
   the items it concerns (#1926, VISION.md "Intent, Not Goals"). The July
   ruling stays below as the record.

   **Goals (re-resolved 2026-07-14).** History: goals were built twice
   (an early goals feature, removed in #213 for a ground-up redesign;
   then Goals #711–#740, removed in #769), with the lessons vertical
   (#273) in between, leaving the per-item `priority` flag (#765,
   `specs/priority-items.md`; priority UI and "neglected priority"
   signal tracked in #763 / #764). The vision/journey audit reopened the
   question: the ideal journey requires goals that *drive planning*, and
   the ruling is to rebuild them deliberately small: an outcome statement
   linked to library items with an optional target date, consumed by
   session planning, and none of the confidence/photo apparatus that sank
   the previous versions. The priority star stays as the zero-ceremony
   layer beneath. See VISION.md "The Scheduling Intelligence" and
   `docs/journeys.md` step 4.

6. **Lessons / photos / R2 surface (resolved 2026-07-14).** Rolled back:
   the lessons vertical was superseded by Goals in #711 (migrations
   0067–0068 dropped the lesson tables); Goals were then removed in #769
   (migrations 0081–0083). Nothing dormant remains, so #570 closes as
   part of the vision/journey audit. R2 photo storage hardening (#281) was
   closed as stale on 2026-09-06 and the R2 module deleted: the bucket was
   empty and photos are on-device files now.
