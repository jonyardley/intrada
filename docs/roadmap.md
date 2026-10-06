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
settled the shape, v0.17.0 practises and scores by it, and v0.18.0 shows those
scores back. Sync follows in v0.19.0 on that settled record.

## The ranking

Work is ranked here directly, by epic: a group of issues with a working order.
Where this list and an issue's
[horizon label](how-the-work-runs.md#horizons) disagree, this list wins.

### Now: v0.18.0, see what you practised by section

v0.17.0 shipped practice by section and variation (#2249): each section with
its own minutes, how an item felt, time away left out, and a focus and a target.

- **Each section's mark on the piece and each variation's marks** (#2250): the
  piece's own score beside its sections', the weakest section suggested, and a
  variation pooled across the library on Progress.
- **How many keys and variations are solid** (#2366), and which items use each
  variation.
- **The first run**: a clearer label than Type a piece (#2294) and the welcome
  from a setting while we beta test it (#2295). Three new musicians try it
  (#2120), which feeds another pass at the welcome (#2296).
- **Ready for sync, alongside**: the two screens in Claude Design (#2360) and a
  trial on two devices (#2361), so v0.19.0 starts on code.

### Next, in order

1. **#2353 Library and practice history sync between iPhone and iPad through
   iCloud** (v0.19.0, moved from v0.18.0 on 2026-10-06): until it lands,
   deleting the app loses a tester's notebook.
2. **#1926 A week's practice set with intent.** It replaces the goals feature,
   starting with a two-lesson trial (#1927, running now) and a research note
   (#1928).
3. **#1975 Play the session through.**
4. **#1974 Build a session your way**, including routines (#1348).
5. **#1970 Practise an exercise in its keys.**
6. **#1972 Add a piece in one pass** and **#1973 the Library**, refined when
   use shows a gap.

### Later

- **#1978 Know what to practise next**: priorities and spacing.
- **#1977 See the practice working** and **#1976 Score how it went.**
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
There is no web app and no sync: the server that once carried sync was removed
(#1746), so the spec's sync plans are history, and syncing between devices
starts from a new spec (#2353). An Android app on the same core is being built
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
