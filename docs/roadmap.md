# intrada — Product Roadmap

*This is a vision and prioritisation doc, not a project plan. For what's
actually shipping right now, see the [project board](https://github.com/users/jonyardley/projects/2)
and [open issues](https://github.com/jonyardley/intrada/issues).*

---

## How to read this doc

intrada serves the five layers of the [Product Vision](../VISION.md):
**Capture → Plan → Space → Show → Guide**. The layers build on each other
— you can't schedule what you haven't captured, can't space what you
haven't scheduled, can't show progress on what you haven't tracked.

The roadmap is also organised around three activity pillars — **Plan**
(decide what to practise), **Practice** (play with intention), and
**Track** (see the process working) — that cut across the layers.

This doc names the layers, pillars, and prioritisation principle.
**Issues on the [project board](https://github.com/users/jonyardley/projects/2)
are the source of truth for scope and timing.** When this doc and the
issues disagree, the issues are right.

---

## Where the product is today

Capture, Plan, Space (manual), and Show (analytics) all have working
surfaces. Auth, library, sessions, routines, scoring, focus mode, tempo
tracking, design system, multi-device shell, E2E tests — all done. The
native SwiftUI iOS app (on the Crux core — see
[`specs/native-ios.md`](../specs/native-ios.md)) is the active iOS path;
the Tauri/Leptos shells are paused.

The active gaps are deeper Layer-1 capture (multi-key, sections, archive),
the Space layer (mastery decay, spaced repetition), and parts of Show
(timeline charts, calendar). Layer 5 (Guide / AI) is intentionally on the
horizon.

Filter `is:open is:issue` on the board to see what's currently in flight.

### Current focus (2026-07)

The lesson-to-mastery loop (see the
[epic, #1087](https://github.com/jonyardley/intrada/issues/1087)) has
workstream **B: track exercises per piece** (#1081) merged (B1 #1095, B2
#1097). **C: exercise steps** (#1083) is under way: C1 (the core variant
mechanism + GRDB child table, spec in `specs/exercise-variants.md`) has
landed; C2 (Steps UI + reflection picker), C3 (12-keys preset, closes #46 and
unblocks the #1107 twelve-key scaffold ladder) and C4 (step management) follow.
R (suggest the next session, #1082) remains queued. Workstream **A (capture a
lesson in one pass, #1080) is on hold**: it's being rethought before more is
built on it, and its merged core event was reverted (#1092). Check the
[project board](https://github.com/users/jonyardley/projects/2) for what's
actually in flight.

---

## Three pillars

### Plan — "Decide what to practise"

Before the instrument comes out. Building sessions, organising the
library, and eventually letting the app decide for you. Spans Layers 1
(Capture), 2 (Plan), and 3 (Space).

→ [`pillar:plan`](https://github.com/jonyardley/intrada/labels/pillar%3Aplan)

### Practice — "Play with intention"

Instrument is out. Timer running. The app stays out of the way and
supports focus, not admin. Lives mainly in Layer 4 (Show — encouragement,
feedback) with cross-cutting accessibility concerns.

→ [`pillar:practice`](https://github.com/jonyardley/intrada/labels/pillar%3Apractice)

### Track — "See the process working"

After the session. Analytics, insights, visualisation. Evidence the
practice is actually working. Mostly Layer 4 (Show); Layer 5 (Guide) when
AI work becomes timely.

→ [`pillar:track`](https://github.com/jonyardley/intrada/labels/pillar%3Atrack)

---

## Prioritisation principle

**Vision layer first, then pillar.** Layer 1 features take precedence
over Layer 2, which take precedence over Layer 3, and so on. Within a
layer, the three pillars advance independently.

The question is *"what's the most important thing I haven't captured
yet?"* before *"what's the smartest way to schedule it?"*

### Now / Next / Later in your head

- **Now** — a real musician hits a wall *because* this isn't built.
- **Next** — a real musician notices it's missing after a month of use.
- **Later** — the app could live without it for a year.

These are tags on issues, not buckets in this doc. They drift as work
ships and as we learn.

---

## Mobile shell

**Pivoting to a native SwiftUI app (2026-05-31).** iOS is moving off the
Tauri 2 + Leptos WKWebView to a **fully native SwiftUI app on the shared
Crux (Rust) core** — finally using Crux as designed (pure core + thin
native shell over FFI/typegen). This is app-first (native iOS now, native
Android later) and **local-first** (on-device SQLite is the source of
truth; the Axum+Turso backend becomes a sync target). The web app stays on
Leptos, untouched.

This resurrects-and-modernizes the SwiftUI shell that was removed in #382,
rather than starting from scratch. The Tauri shell stays shipping until the
native app reaches parity, then retires.

See [`specs/native-ios.md`](../specs/native-ios.md) for the phased plan
(Phase A bridge spike → B local-first persistence → C screen parity → D
sync + retire Tauri). Active work tracks under the
[`ios`](https://github.com/jonyardley/intrada/labels/ios) label and the
native-iOS [`epic`](https://github.com/jonyardley/intrada/labels/epic).

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
| `ios` | iOS-specific (Tauri shell) |
| `epic` | Umbrella issue with sub-items |

### Board

[Project board](https://github.com/users/jonyardley/projects/2) columns
are workflow states (Backlog → Ready → In Progress → In Review → Done),
not categories. Use the labels above to filter and slice.

A `Priority` custom field (P0/P1/P2) ranks within a pillar/layer when
multiple items share the same horizon.

---

## Open questions

These are unresolved product questions. Each one likely produces issues
(or a Tier-3 spec) once answered.

1. **Metronome: built-in or external?** Tempo tracking (#52) shipped — a
   built-in metronome would complete the tempo-building loop. Building
   one in WASM is non-trivial (timing accuracy in a browser/WebView).

2. **Offline-first architecture.** Currently API-dependent. What syncs?
   When? Gets harder to retrofit the longer we wait. Tracked at #41.

3. **Scoring + tempo coupling.** Should every mastery rating require a
   tempo? Or is tempo optional (only for items with tempo targets)?

4. **Teacher integration timing.** Currently a Layer-5 horizon. Basic
   sharing (routines, item suggestions) could come earlier without AI.
   The teacher-assignment capture (#267) addressed the immediate capture
   problem without teacher-facing features.

5. **Goals (re-resolved 2026-07-14).** History: goals were built twice
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
   part of the vision/journey audit. R2 photo storage hardening (#281)
   survives independently as a security item.
