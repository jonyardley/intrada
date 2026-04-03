# intrada — Product Roadmap

*Updated 2026-04-03*

Everything in intrada serves one of three activities a musician does around their
instrument. This roadmap is organised around those three pillars, not delivery
phases. Each pillar advances independently — a musician benefits from progress in
any of them without waiting for the others.

---

## What's Built Today

| Area | Status |
|------|--------|
| Auth (Clerk/Google, JWT, JWKS refresh, 401 retry) | Done |
| Library CRUD (piece + exercise, tags, key, tempo, notes) | Done |
| Sessions (build, active, summary, timer, crash recovery) | Done |
| Routines (full CRUD, load into builder, save from summary) | Done |
| Analytics (weekly stats, streak, 28-day chart, top items, score trends, weekly summary) | Done |
| Scoring (1-5 per item) and per-item/session notes | Done |
| Search/filter infrastructure in core (ListQuery) | Backend only (no UI) |
| Design system (tokens, typography utilities, component library) | Done |
| Drag-and-drop session builder | Done |
| Composer + tag autocomplete | Done |
| E2E test suite (Playwright, 30 tests) | Done |
| Intentional practice micro-cycle (#47) — session + per-item intentions | Done |
| Focus mode (#48) — minimal UI during active practice | Done |
| Repetition counter (#49) — consecutive correct attempts per item | Done |
| Tempo tracking (#52) — log achieved tempo per item, progress charts (#66) | Done |
| Weekly practice summary (#68) — week-over-week comparison, neglected items, score changes | Done |
| Holistic practice loop (#93) — unified library, sessions & routines flow | Done |
| DB composite indexes (#146) and request timing middleware (#147) | Done |
| Skeleton loading states (#155) — initial load flicker fix, skeleton placeholders | Done |
| Design system token compliance (#163) — all raw Tailwind colours replaced with semantic tokens | Done |
| Practice consistency calendar (#65) — week strip, comeback framing, session dots | Done |
| iOS native shell (#187) — Crux/UniFFI/BCS bridge, SwiftUI, CI pipeline | Done |
| iOS cleanup (#202) — removed dead code and legacy networking layer | Done |
| iOS design system (#194) — tokens, shared components, modifiers, navigation | Done |
| Remove category field (#211) — exercises use tags only, simplified subtitle logic | Done |
| Use mutate response (#148) — updates/deletes use API response directly, no re-fetch | Done |
| Filter clause sync (#152) — extracted subquery constants to prevent user isolation drift | Done |
| iOS library (#195) — browse, search & manage repertoire on iOS | Done |
| iOS session builder (#196) — tap-to-queue setlist construction | Done |
| iOS active session (#197) — focus mode, timer, scoring, rep counter, pause/resume | Done |
| iOS session summary & history (#198) — post-session review, save, history list, detail view | Done |

---

## Current Focus: iOS Feature Parity

Building out the iOS app as the primary user channel. SwiftUI shell using
the shared Crux core via UniFFI/BCS bridge. Shell, CI, and design system are
complete — now building out feature views to reach parity with the web app.

Build order follows the user journey: library first (need items to practise),
then sessions (the core loop), then routines and analytics. After iOS parity,
cross-platform features (Crux core + API) benefit both shells simultaneously.

| # | Feature | Size | Depends on |
|---|---------|------|------------|
| ~~195~~ | ~~**Library** — browse, search & manage repertoire~~ | ~~M~~ | Done |
| ~~196~~ | ~~**Session builder** — construct practice setlists~~ | ~~M~~ | Done |
| ~~197~~ | ~~**Active session** — focus mode, timer & scoring~~ | ~~L~~ | Done |
| ~~198~~ | ~~**Session summary & history**~~ | ~~M~~ | Done |
| 199 | **Routines** — create, edit & manage practice routines | M | #195 |
| 201 | **Analytics dashboard** — practice insights & visualisation | M | #198 |

---

## Three Pillars

### Plan — "Decide what to practise"

Before the instrument comes out. Building sessions, organising the library, setting
goals, and eventually letting the app decide for you.

#### Next (4-12 weeks)

| # | Feature | Size |
|---|---------|------|
| 50 | **Section/passage management** — break pieces into practisable sections with independent mastery scores and tempo targets. | L |
| 46 | **Multi-key practice** — assign multiple keys to items, track mastery per key independently. | XL |
| 45 | **Archive/retire workflow** — handle completed/mastered items so the active library stays focused. | M |
| 53 | **Full-text search** — search across all practice notes to find recurring themes. | M |
| 142 | **Session time budgeting** — declare available time, allocate across items automatically or manually. | M |
| 57 | **One-tap session start** — open the app, tap Start, play. Initial version uses recent routine + items not practised recently. | L |
| 59 | **Session planning** — input available time, get a structured plan with warm-up, focused work, and review segments. | L |

#### Later (12+ weeks)

| # | Feature | Size |
|---|---------|------|
| 55 | **Spaced repetition engine** — modified SM-2 algorithm for optimal review intervals based on mastery scores and time since last practice. | XL |
| 56 | **Interleaved setlist generator** — mixed-type sessions with configurable interleaving intensity. | L |
| 58 | **Mastery decay model** — scores decrease over time without review, creating natural scheduling urgency. | M |
| 42 | **Links to IMSLP** — link pieces to free sheet music on IMSLP. | S |
| 43 | **Open-source exercise library** — built-in scales, arpeggios, Hanon, Czerny etc. | L |
| 71 | **AI setlist generation** — goal-driven practice plans ("I have a gig in 3 weeks — build me a plan"). | L |
| 76 | **AI goal coaching** — realistic goal setting and actionable daily practice plans. (blocked on goals redesign) | L |
| 100 | **Personalisation** — user preferences and customisation. | — |

---

### Practice — "Play with intention"

Instrument is out. Timer running. The app stays out of the way and supports
focus, not admin.

#### Next (4-12 weeks)

| # | Feature | Size |
|---|---------|------|
| 61 | **Encouragement messaging** — data-tied, process-focused messages comparing current ratings to recent history. | S |
| 54 | **Dyslexia-friendly typography** — clean fonts, adequate spacing, sensory-considerate defaults, configurable contrast modes. | S |
| 62 | **Rest & recovery awareness** — flag when practice volume significantly exceeds historical average. | S |
| 166 | **Session tempo targets** — suggest incremental tempo targets based on recent progress toward the item's target BPM. | M |
| 212 | **Short session presets** — 10/15/20/30 minute options | S |

#### Later (12+ weeks)

| # | Feature | Size |
|---|---------|------|
| 69 | **Audio recording & playback** — record a run-through, play it back, compare to weeks ago. | L |
| 70 | **Customisable encouragement preferences** — frequency, tone, delivery controls. Respects neurodiversity. | M |

---

### Track — "See the process working"

After the session. Analytics, insights, visualisation. Evidence that the practice
is actually working.

#### Next (4-12 weeks)

| # | Feature | Size |
|---|---------|------|
| 63 | **Mastery timeline charts** — per-item and aggregate line charts showing mastery improvement over weeks and months. | L |
| 79 | **Calendar view** — calendar-based view of practice sessions. | M |

#### Later (12+ weeks)

| # | Feature | Size |
|---|---------|------|
| 64 | **Circle of fifths widget** — interactive key coverage visualisation showing mastery per key. | L |
| 72 | **AI pattern recognition** — identify systematic weaknesses from accumulated data. | L |
| 73 | **AI session review** — post-session analysis with rebalancing suggestions. | M |
| 74 | **Adaptive interleaving** — AI adjusts mixing intensity based on user patterns. | L |
| 75 | **Teacher integration** — suggested items, progress visibility (with permission). | L |

---

## Cross-Cutting Concerns

These don't belong to a single pillar — they support all three.

| # | Feature | Horizon | Labels |
|---|---------|---------|--------|
| 41 | **Offline-first support** — service worker, IndexedDB, sync | Next | architecture |
| 149 | **Pagination** — add limit/offset to list endpoints | Next | architecture |

---

## Delivery Model

### Labels

| Label | Purpose |
|-------|---------|
| `pillar:plan` | Decide what to practise |
| `pillar:practice` | Play with intention |
| `pillar:track` | See the process working |
| `horizon:now` | Next 4 weeks |
| `horizon:next` | 4-12 weeks |
| `horizon:later` | 12+ weeks / future |
| `architecture` | Technical debt, infrastructure |
| `ux` | User experience (cross-cutting) |
| `accessibility` | Neurodiversity and accessibility (cross-cutting) |

### Project board

Backlog > Ready > In Progress > In Review > Done

The board columns are workflow states, not categories. Use the pillar and horizon
labels to filter and slice the board.

### Prioritisation

Each pillar advances independently. "What's the most important Plan feature?"
is a better question than "What phase are we in?" Within each pillar, the
Priority field (P0/P1/P2) ranks items.

---

## Open Questions

1. **Metronome: built-in or external?** The tempo-building loop needs a metronome.
   Building one is non-trivial (accurate timing in WASM). Tempo tracking (#52) has
   shipped — a built-in metronome would complete the tempo practice workflow.

2. **Offline-first architecture.** Currently API-dependent. What syncs? When?
   Gets harder to retrofit the longer we wait.

3. ~~**iOS native vs web-first.**~~ **Decided: iOS is the primary channel.**
   The native shell is built and feature views are in progress (#195–#201).
   Cross-platform features (Crux core, API) benefit both shells.

4. **Scoring + tempo coupling.** Should every mastery rating require a tempo?
   Or is tempo optional (only for items with tempo targets)?

5. **Teacher integration timing.** Currently horizon:later. Basic sharing
   (routines, item suggestions) could come earlier without AI.

6. **Goals redesign.** The goals feature has been removed for a ground-up rethink.
   What value should goals add to the practice experience? How do they connect
   to the critical path and guided pathways vision?
