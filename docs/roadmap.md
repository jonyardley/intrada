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

Capturing material is done, and v0.16.0 to v0.18.0 put pieces in sections and
variations and showed their marks back (#50). **The order from 2026-10-13: make
the data model sound, then get every tab to where intrada is used daily in
anger, then sync.** Sync moved behind daily use because every change to the
record after it ships has to be migrated on every device.

## The ranking

Work is ranked here directly, by milestone and epic. Where this list and an
issue's [horizon label](how-the-work-runs.md#horizons) disagree, this list
wins.

### Now: v0.19.0, a data model you can build on

- **The review** (#2577,
  [review page](https://claude.ai/artifact/9D4tbou13mtMcvY1Jj27tn)): the record
  checked against Progress, routines, the week and sync, with nine decisions
  for Jon. Each accepted one becomes an issue in this milestone.
- **What is left of #50**: adding a section and a key while building a session
  (#2392), every section on the practice screen (#2394), last time's aim as a
  one-tap start (#2397) and the builder rules the core still has to take over
  (#2404, #2405, #2406).
- **Leftovers and loose ends**: the coach-era tables (#2317), values an older
  build does not recognise (#2437), the practice defaults version (#2216),
  tempo in compound time (#1510, #2543) and the time zone a session was played
  in (#1379).

### Next, in order

1. **v0.20.0 Progress, reworked**: the four-week look back decided on
   2026-10-04 (#1787), core first (#2375), then the screen (#2376), with
   variation coverage moved onto the exercise (#2377).
2. **v0.21.0 The fourth tab earns its place**: decide between routines on their
   own tab and a Week tab holding the week's practice set with routines inside
   it (#2579), then build the one chosen (#1348, #2180, and #1926 if Week).
3. **v0.22.0 Every tab usable in anger**: a written bar per tab, met (#2578),
   fed by three new musicians trying the first run (#2120) and the welcome's
   next pass (#2296). The issues that block the bar join that milestone from
   #1972, #1973, #1974 and #1975.
4. **v0.23.0 Sync between iPhone and iPad through iCloud** (#2353), on the
   settled record.

### Later

- **#1926 A week's practice set with intent**, unless v0.21.0 makes it the
  fourth tab; the lesson trial (#1927) and research note (#1928) carry on.
- **#1978 Know what to practise next**: priorities and spacing.
- **#1976 Score how it went** and what remains of **#1977** after v0.20.0.
- **#1970 Practise an exercise in its keys.**
- **#1979 One look everywhere** and **#1980 Make it yours, part two.**
- **#2134 Every screen holds together at the largest text sizes**, set aside
  on 2026-10-06 along with its snapshots.
- **#2025 Chord charts**, parked until the design is reopened.

### Alongside: Android parity

The Android app on the same core (#2220) is pressed toward parity with the
iPhone app as its own stream, tracked in the
[Android parity milestone](https://github.com/jonyardley/intrada/milestone/13)
and not ranked against the product work. The last rules come out of the Swift
shell first (#2223: #2230, #2231, #2352). Then, from
[`specs/android-shell.md`](../specs/android-shell.md): adding and editing pieces
kept after the app closes (#2421, which can start now), building and playing a
session (#2422, after #2223), and capture, Progress and Play testing (#2423).

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
There is no web app and no sync yet: the server that once carried sync was
removed (#1746), and iCloud sync between iPhone and iPad (#2353) is planned for
v0.23.0. An Android app on the same core is being built
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
