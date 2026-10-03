# intrada: roadmap

*What gets built next, and in what order. The released build and the current
phase are in [`where-we-are.md`](where-we-are.md); what is being built right
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
library, with exercises linked to the sections they prepare (#50).** Every
later feature reads that shape, so it settles before more is built on it.

## The ranking

Work is ranked here directly, by epic: a group of issues with a working order.
Where this list and an issue's
[horizon label](how-the-work-runs.md#horizons) disagree, this list wins.

### Now: v0.16.0

- **#50 Practise a piece in sections and its variations**, steps 1 to 4:
  sections (#2245), variations replacing steps (#2246), the item screen
  (#2247) and exercises linked to sections (#2248). #2246 also makes a
  variation a real key (#2106) and records the tempo of every repetition
  (#2107), so the saved session in progress changes once. The builder, live
  session and scoring (#2249) and section scores (#2250) lead v0.17.0.
- **Core tidy-ups beside it**: the unshown streak (#2190), the unread session
  start time (#2170) and the saved library sort's version (#2089).
- **The clock that keeps ticking after an item ends** (#2297).
- **Watching three new musicians do a first run** (#2120), with the welcome's
  follow-ups (#2294, #2295, #2296).
- **Moving rules out of the Swift shell** (#2223) carries on, except #2228,
  #2230, #2231 and #2232, which wait for #2246 and #2248 because they rewrite
  the code those replace.

### Next, in order

1. **#50 continued**: the builder, live session and scoring by section and
   variation (#2249), then section scores and charts by kind (#2250).
2. **#1926 A week's practice set with intent.** It replaces the goals feature,
   starting with a two-lesson trial (#1927) and a research note (#1928).
3. **#2134 Every screen holds together at the largest text sizes.**
4. **#1975 Play the session through**, starting with session start feeling
   slow (#1801).
5. **#1974 Build a session your way**, including routines (#1348).
6. **#1970 Practise an exercise in its keys.**
7. **#1972 Add a piece in one pass** and **#1973 the Library**, refined when
   use shows a gap.

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
