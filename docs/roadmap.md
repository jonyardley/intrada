# intrada — Product Roadmap

*This is a vision and prioritisation doc, not a project plan. For what's
actually shipping right now, see the [project board](https://github.com/users/jonyardley/projects/2)
and [open issues](https://github.com/jonyardley/intrada/issues).*

---

> ## ⚠️ THE 2026-07 PIVOT: THE PRACTICE COACH
>
> As of July 2026 the product direction is the **practice coach** — the app
> decides what you practise, gates every block on evidence, and tells you
> when you're done. **Revised 4 Aug 2026: machine listening is deferred**
> (design doc v6, decision 18) — v1 evidence is the tap-verdict against
> countable criteria the app controls (tempo, key, scope, cold-test
> intervals); the MIDI scoring path returns later as an evidence upgrade,
> play-to-input first. The vision, pedagogy model, and phased build plan live
> in
> [`specs/intrada-practice-coach-design.md`](../specs/intrada-practice-coach-design.md);
> the codebase assessment and pivot strategy (pivot in place, new engine
> module, notebook-era machinery deleted at Phase 2a) in
> [`docs/rebuild-review.md`](rebuild-review.md).
>
> **The notebook-era backlog below is superseded.** Issues carrying the
> [`superseded-by-pivot`](https://github.com/jonyardley/intrada/labels/superseded-by-pivot)
> label (the lesson-to-mastery epic #1087, session-builder work #1101, the
> twelve-key ladder #1107) are parked — don't pick them up without revisiting
> the pivot plan. Keep-column assets (chord theory in `chart.rs`, the GRDB
> persistence pattern, the design system, the FFI toolchain) carry forward.
>
> Current phase: **Phase 0** (paper teacher — content authored under
> [`content/`](../content/README.md), practice logs in progress, no code),
> then **Phase 2a**. Phase 1 closed early under decision 18: the capture
> harness, click and segmentation spike are merged and inert; scoring
> deferred.
>
> **The document map**, so there is one answer per question:
>
> | Question | Document |
> |---|---|
> | What are we building, and why? | [`specs/intrada-practice-coach-design.md`](../specs/intrada-practice-coach-design.md) (v7) |
> | What does it feel like to use? | [`docs/coach-user-journeys.md`](coach-user-journeys.md) |
> | What survives the pivot, what dies when? | [`docs/rebuild-review.md`](rebuild-review.md) |
> | Can I implement from this old spec? | [`specs/README.md`](../specs/README.md) |
> | What order does the work happen in? | The phase plan below |
> | Can I see it all at a glance? | [`docs/coach-orientation.html`](coach-orientation.html) — visual snapshot, open in a browser |
> | What do I hand to Claude Design? | [`design/briefs/2026-08-coach-drill-loop.md`](../design/briefs/2026-08-coach-drill-loop.md) |
>
> Pre-pivot specs now carry a superseded banner, and
> [`docs/journeys.md`](journeys.md) — the notebook-era "ideal journey" that
> claimed authority over prioritisation — is explicitly retired.

---

## The phase plan (the live sequence)

Coach work follows these phases in order. The layer/pillar framing further down
still describes the surviving library surfaces, but **it is not the sequence for
coach work** — this is.

| Phase | What lands | State |
|---|---|---|
| **0 · Paper teacher** | Content authored (`content/`); the fortnight of practice + four logs | Content done; fortnight outstanding (#1143) |
| **1 · The listening gate** | Capture harness + click → segmentation spike → lick-transposition scoring → one drill screen | **Closed early (decision 18, 4 Aug 2026).** PR 2 merged (jonyardley/intrada#1157); PR 3 done (#1161) — the `engine` module segments all five real takes, findings in [`segmentation-findings.md`](segmentation-findings.md). Lick-transposition scoring and the machine-verdict drill screen defer to the scoring path's return (trigger: play-to-input). USB comparison stays open-ended (#1156) |
| **2a · Prescribe and run** | Planner as a pure function, press-start, gated blocks on tap-verdicts, stuck ladder, soft-landing exit, builder deleted | **In flight.** Mastery function specified (#1155); evidence = tap-verdicts, source-tagged (decision 17 as amended). The drill screen (A2 during play + A3 tap-verdict) and the seven coach primitives are **built** (#1178). The **session state machine and the tap-verdict bridge surface are built** (#1176, spec §4/§6): `engine/session.rs` owns counting, gating, the stuck ladder and the ceiling; `DrillScreen` renders the core's `CoachView` and the Swift harness is deleted. Still open in 2a: the planner (§5), press-start (#1182), the `gates.toml` parser (#1180), persisting block records (#1181) |
| **2b · Steer and guard** | Declaration surfaces (goal / campaign / steer), three-way target resolution + user-created items + the built session (decision 19), back-chaining, gap read, circling check, grind trade, off-piano queue, unmonitored play, circle tally, the judgement track | After 2a |
| **3 · The voice** | LLM behind Axum: summaries, whys, stuck coaching, goal interpretation | Deliberately late |
| **4 · Widen** | Spacing, difficulty auto-adjust, statistical gap read, planner bias, audio path, placement, second user | Ongoing |

**Phase 2 was split (3 Aug 2026).** It had accumulated twelve deliverables under
a "3–4 weeks" estimate — planner, three declaration surfaces, back-chaining, gap
read, circling check, grind trade, off-piano queue, unmonitored mode, circle
tally, soft-landing exit, *and* deleting the session builder. That is not one
phase. 2a is the loop working end to end; 2b is everything that makes it kind.

**Prerequisite met (updated 4 Aug 2026):** the mastery update function is
specified in [`specs/intrada-coach-engine.md`](../specs/intrada-coach-engine.md)
§2 (#1155). Its inputs are settled by decision 17 **as amended**: tap-verdicts
against countable criteria feed the Beta update, source-tagged so machine
scoring arrives later as a higher-weight evidence class. Self-report *judgement*
(feel, opaque targets) stays in the separate judgement track — may retire a
target, may never satisfy a prerequisite. The divergence log (and with it the
judgement-track weighting question) defers until machine listening returns.

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
[`specs/native-ios.md`](../specs/native-ios.md)) is the only shell; the
Leptos web app and the Tauri iOS host were **deleted** in #1133, not paused.

The active gaps are deeper Layer-1 capture (multi-key, sections, archive),
the Space layer (mastery decay, spaced repetition), and parts of Show
(timeline charts, calendar). Layer 5 (Guide / AI) is intentionally on the
horizon.

Filter `is:open is:issue` on the board to see what's currently in flight.

### Current focus (2026-07)

The practice-coach pivot (see the banner at the top). The lesson-to-mastery
loop that was the previous focus (epic #1087; workstreams B and C largely
landed, A reverted) is parked under `superseded-by-pivot`. Phase 1's harness,
click and segmentation spike are merged (#1157, #1161) and the phase closed
early under decision 18. Active work: the Phase 0 fortnight (#1143), then
Phase 2a — the coaching loop on tap-verdict evidence. Check the
[project board](https://github.com/users/jonyardley/projects/2)
for what's actually in flight.

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

## Mobile shell — settled

The native SwiftUI app on the shared Crux core **is** the shell, and has been
since the 2026-05-31 decision completed: app-first, **local-first** (on-device
SQLite is the source of truth; Axum + Turso is a future sync target, not the live
read path). The Tauri 2 + Leptos WKWebView host and the Leptos web app were
deleted in #1133; two pieces of Swift worth reusing were mined into
`ios/Reference/` first — the background-audio session handling (**a Phase 1
input** for the click track) and a Live Activity implementation.

See [`specs/native-ios.md`](../specs/native-ios.md) for the shell spec. Work
tracks under [`ios`](https://github.com/jonyardley/intrada/labels/ios).

Historical note: the sentences that used to live here ("the web app stays on
Leptos, untouched", "the Tauri shell stays shipping until parity") were true in
May and false by July. Statements about what is *currently* shipping belong in
one place — the banner at the top of this doc.

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
