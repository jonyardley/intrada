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

Capturing material is done: v0.10.0 added a piece with its chord chart and
exercises in one pass (#1390), and v0.11.0 to v0.14.0 built on it. **Next is a
first run that gets a new musician to their first marked session (#2121).** So
far the person building the app is its only regular user, and every later
decision needs evidence from other musicians.

## The ranking

Work is ranked here directly, by epic: a group of issues with a working order.
Where this list and an issue's
[horizon label](how-the-work-runs.md#horizons) disagree, this list wins.

### Now: the next release

- **#2121 A first run that gets a new musician to their first marked
  session**, ending with watching three new musicians try it (#2120).
- **One-tap start (#57) and today's plan (#999) shipped** (#2182, #2202): with
  a preferred length set, the Practice screen holds a plan filled to it, and
  Start plays it in one tap. The Start here card (#2118) will share the top of
  the Practice screen with it: a new musician sees Start here until there is
  something to plan from.
- **Closing out** the September audit backlog (#1967) and making the core
  easier to change (#1995), each with a handful of issues left.

### Next, in order

1. **#1926 A week's practice set with intent.** It replaces the goals feature,
   starting with a two-lesson trial (#1927) and a research note (#1928).
2. **#2134 Every screen holds together at the largest text sizes.**
3. **#1975 Play the session through**, starting with session start feeling
   slow (#1801).
4. **#1974 Build a session your way**, including routines (#1348).
5. **#1970 Practise an exercise in its keys.**
6. **#1972 Add a piece in one pass** and **#1973 the Library**, refined when
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
would start from a new spec. An Android app on the same core is specced for later
([`specs/android-shell.md`](../specs/android-shell.md)), with no build started.

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
