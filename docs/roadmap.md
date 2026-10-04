# intrada: roadmap

*What gets built next, and in what order. The current phase is in
[`where-we-are.md`](where-we-are.md) and the released build in
`gh release list`; what is being built right
now is `just status`, the
[project board](https://github.com/users/jonyardley/projects/2) and the
[open issues](https://github.com/jonyardley/intrada/issues). How issues, epics
and releases are organised is in [`how-the-work-runs.md`](how-the-work-runs.md).*

## Direction

intrada is a practice notebook. **The musician decides what matters; the app
decides where today starts, keeps the record honest and can always be
overruled** ([`VISION.md`](../VISION.md), 2026-09-30).

Capturing material is done, and v0.15.0 shipped a first run that gets a new
musician to their first marked session (#2121). **Next is finishing the data
model: a piece in sections, practised in variations shared across the
library, with exercises linked to the sections they prepare (#50).** v0.16.0
settled the shape; v0.17.0 practises and scores by it, and sync follows in
v0.18.0 on that settled record.

## The ranking

Work is ranked here directly, by epic: a group of issues with a working order.
Where this list and an issue's
[horizon label](how-the-work-runs.md#horizons) disagree, this list wins.

### Now: v0.17.0, practise by section

v0.16.0 shipped #50 steps 1 to 4: sections, shared variations, the item
screen and exercises linked to sections.

- **#50 continued: build, play and score a session by section and variation**
  (#2249), two PRs, core first. Each section gets its own minutes (#2315), and
  the same change records how an item felt (#2308), leaves time away out
  (#2306) and sets a focus and a target (#2303), so the saved session in
  progress changes once. Then each section's mark and each variation's marks
  (#2250).
- **Moving rules out of the Swift shell** (#2223), now unblocked: #2228, #2230,
  #2231, #2352, #2372 and #2379.
- **Small fixes**: the profile icon that changes on save (#2347), the profile
  editor's small Change button (#2318), grouped block titles cut at the
  largest text size (#2334) and Progress claiming a climb when every mark fell
  (#2374).
- **The first run**: three new musicians try it (#2120), with the welcome's
  follow-ups (#2294, #2295, #2296) and solid keys and variations (#2366).
- **Ready for sync, alongside**: the iCloud container (#2359), the two screens
  in Claude Design (#2360) and a trial on two devices (#2361).

### Next, in order

1. **#2353 Library and practice history sync between iPhone and iPad through
   iCloud** (v0.18.0, chosen 2026-10-04): until it lands, deleting the app
   loses a tester's notebook.
2. **#1926 A week's practice set with intent.** It replaces the goals feature,
   starting with a two-lesson trial (#1927, running now) and a research note
   (#1928).
3. **#2134 Every screen holds together at the largest text sizes.**
4. **#1975 Play the session through.**
5. **#1974 Build a session your way**, including routines (#1348).
6. **#1970 Practise an exercise in its keys.**
7. **#1972 Add a piece in one pass** and **#1973 the Library**, refined when
   use shows a gap. Picking bars, tempos and targets out of a note (#2307)
   waits here.

### Later

- **#1978 Know what to practise next**: priorities and spacing.
- **#1977 See the practice working** and **#1976 Score how it went.**
- **#1979 One look everywhere** and **#1980 Make it yours, part two.**
- **#2025 Chord charts**, parked until the design is reopened.

### Alongside: tooling

The build, the test gates and the agents that write much of the code are
worked on beside the product and not ranked against it: a build a tester can
trust (#1982, which matters more once testers arrive with #2121), the iOS test
gates (#1981), the agent tooling (#1983) and its usage costs (#1849).

## Three pillars

Specs and designs divide a musician's practice into three parts:

- **Plan**: deciding what to practise, before the instrument comes out.
- **Practice**: playing while the timer runs, with the app out of the way.
- **Track**: seeing the practice working afterwards.

## Platform

intrada is a native iPhone app, built in SwiftUI on a shared Rust core, that
keeps everything on the phone ([`specs/native-ios.md`](../specs/native-ios.md)).
There is no web app and no sync: the server that once carried sync was removed
(#1746), so the spec's sync plans are history, and syncing between devices
would start from a new spec. An Android app on the same core is being built
alongside (#2220, [`specs/android-shell.md`](../specs/android-shell.md)), and is
not ranked here.

## Open questions

1. **How does today's plan share its time** between the teacher's week, starred
   items and cold ones, and does it show why each item is there (#2185)? This
   runs once one-tap start has been lived with and the lesson trial (#1927) has
   reported.
2. **When do teachers get features of their own?** Sharing routines or
   suggesting items could come before anything smarter. Capturing what the
   teacher set (#267) solved the musician's side without any teacher-facing
   screens.

Answered questions (the metronome, offline-first storage, tempo on a mark,
goals, lessons and photo storage) are in this file's git history.
