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
variations and showed their marks back (#50). **The goal from 2026-10-13 is
1.0: intrada public on the App Store and Google Play, used daily by musicians
who have never met its maker.** The order: make the data model sound, give both
apps one look, build Progress, Plan and reflection, get every tab to where it
is used daily in anger, then sync and release. Sync comes after everything that
adds to the record, because every change to the record after it ships has to be
migrated on every device.

## The ranking

There is one sequence of milestones and no parallel streams. From v0.20.0
every milestone ships on iPhone and Android together. Where this list and an
issue's [horizon label](how-the-work-runs.md#horizons) disagree, this list
wins.

### Now: v0.19.0, a data model you can build on

- **Main green first**: the failing CI run on main (#2597).
- **The review** (#2577,
  [review page](https://claude.ai/artifact/9D4tbou13mtMcvY1Jj27tn)): a saved
  session that can be deleted and carries its change time (#2581), one
  migration clearing the leftovers (#2317), the time zone a session was played
  in (#1379), and one planned-item shape for sessions, plans and the week
  (#2582).
- **What is left of #50**: adding a section and a key while building a session
  (#2392), every section on the practice screen (#2394), last time's aim as a
  one-tap start (#2397) and the builder rules the core still has to take over
  (#2404, #2405, #2406).
- **Loose ends**: values an older build does not recognise (#2437), the
  practice defaults version (#2216), the shipped migrations test (#2600), and
  tempo in compound time shown by the piece's beat (#1510, #2543).

### Next, in order

1. **v0.20.0 One look on both apps**: the design audit already running
   (#2599), one component catalogue for both apps (#2620), its fixes (#2607,
   #2608, #2610, #2611, #2201), the design tool (#2603), and the Android polish
   that makes the second app feel native (#2605, #2606, #2613 to #2619, #2547).
2. **v0.21.0 Progress, redesigned**: the four-week look back decided on
   2026-10-04 (#1787), core first (#2375), then the screen (#2376), with
   variation coverage on the exercise (#2377), per-variation trends (#1793,
   #1794) and history showing an item's current name (#2583).
3. **v0.22.0 Plan, in place of Routines**: the fourth tab becomes Plan, the
   week's practice set, and a routine becomes a saved plan (#2579, specified in
   #2622). The week is set by hand or captured from a lesson (#2621, #1926),
   and today's plan starts from it (#2180, #2185). The lesson trial (#1927) and
   research note (#1928) feed the spec.
4. **v0.23.0 Intentions and reflections**: the aim set before playing and the
   note made after, typed or recorded (#2623, #69), with Session complete
   redesigned around them (#1976, #1369, #1728).
5. **v0.24.0 Every tab in daily use**: a written bar per tab, met (#2578), fed
   by three new musicians trying the first run (#2120), the welcome's next pass
   (#2296) and example content (#2115), with the player basics (#1000, #1002)
   and Library housekeeping (#45, #1811).
6. **v0.25.0 Sync and safe keeping**: iCloud sync between iPhone and iPad
   (#2353) on the settled record, and export and restore on both apps (#2624).
7. **v0.26.0 Release candidate**: a build a tester can trust on both stores
   (#1982, #2499, #2566, #2517), a way to reach Jon (#2369, #2625), and the
   privacy policy and store listings (#2626, #2627).
8. **1.0.0 Public on both stores** (#2628).

### Off the milestones

- **Upkeep, picked up when it blocks**: the iOS test gates (#1981), the agent
  harness (#1983), its usage costs (#1849) and a core that is easy to change
  (#1995). An item joins the current milestone only when it stops that
  milestone shipping. The Android epic (#2220) closes at 1.0.
- **Postponed until after 1.0**: every screen at the largest text sizes
  (#2134), set aside again on 2026-10-13 with its children kept open.
- **After 1.0, roughly in this order**: know what to practise next (#1978),
  practise an exercise in its keys (#1970), make it yours, part two (#1980),
  what the audit leaves of one look everywhere (#1979), and chord charts
  (#2025), parked until the design is reopened.

## Three pillars

Specs and designs divide a musician's practice into three parts:

- **Plan**: deciding what to practise, before the instrument comes out.
- **Practice**: playing while the timer runs, with the app out of the way.
- **Track**: seeing the practice working afterwards.

## Platform

intrada is two native apps on a shared Rust core, kept at parity: SwiftUI on
iPhone ([`specs/native-ios.md`](../specs/native-ios.md)) and Compose on Android
([`specs/android-shell.md`](../specs/android-shell.md)). Each keeps everything
on the phone. There is no web app and no sync yet: the server that once
carried sync was removed (#1746), and iCloud sync between iPhone and iPad
(#2353) with export and restore for both apps (#2624) is planned for v0.25.0.

## Open questions

1. **How does today's plan share its time** between the teacher's week, starred
   items and cold ones, and does it show why each item is there (#2185)? The
   Plan spec (#2622) answers it, once one-tap start has been lived with and the
   lesson trial (#1927) has reported.
2. **When do teachers get features of their own?** Sharing plans or suggesting
   items could come before anything smarter. Capturing what the teacher set
   (#267, #2621) solves the musician's side without any teacher-facing
   screens.

Answered questions (the metronome, offline-first storage, tempo on a mark,
goals, lessons and photo storage) are in this file's git history.
