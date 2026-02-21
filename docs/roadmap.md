# intrada — Product Roadmap

*Updated 2026-02-21*

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
| Analytics (weekly stats, streak, 28-day chart, top items, score trends) | Done |
| Scoring (1-5 per item) and per-item/session notes | Done |
| Search/filter infrastructure in core (ListQuery) | Backend only (no UI) |
| Design system (tokens, typography utilities, component library) | Done |
| Drag-and-drop session builder | Done |
| Composer + tag autocomplete | Done |
| E2E test suite (Playwright, 30 tests) | Done |

---

## Three Pillars

### Plan — "Decide what to practise"

Before the instrument comes out. Building sessions, organising the library, setting
goals, and eventually letting the app decide for you.

#### Now (next 4 weeks)

| # | Feature | Size |
|---|---------|------|
| 47 | **Intentional practice micro-cycle** — set a focus goal before each item, reflect after. Lightweight: one tap minimum, notes optional. Completes the goal-practise-reflect loop. | S |
| 93 | **Holistic practice loop** — parent issue unifying library, sessions, routines and goals into a coherent flow. | — |

#### Next (4-12 weeks)

| # | Feature | Size |
|---|---------|------|
| 50 | **Section/passage management** — break pieces into practisable sections with independent mastery scores and tempo targets. | L |
| 46 | **Multi-key practice** — assign multiple keys to items, track mastery per key independently. | XL |
| 45 | **Archive/retire workflow** — handle completed/mastered items so the active library stays focused. | M |
| 53 | **Full-text search** — search across all practice notes to find recurring themes. | M |
| 60 | **Basic goal setting** — session frequency, practice time, item mastery, and milestone goals. Progress computed from session data. | L |
| 59 | **Session planning** — input available time, get a structured plan with warm-up, focused work, and review segments. | L |

#### Later (12+ weeks)

| # | Feature | Size |
|---|---------|------|
| 55 | **Spaced repetition engine** — modified SM-2 algorithm for optimal review intervals based on mastery scores and time since last practice. | XL |
| 56 | **Interleaved setlist generator** — mixed-type sessions with configurable interleaving intensity. | L |
| 57 | **One-tap session start** — open the app, tap Start, play. Zero other decisions. | L |
| 58 | **Mastery decay model** — scores decrease over time without review, creating natural scheduling urgency. | M |
| 42 | **Links to IMSLP** — link pieces to free sheet music on IMSLP. | S |
| 43 | **Open-source exercise library** — built-in scales, arpeggios, Hanon, Czerny etc. | L |
| 71 | **AI setlist generation** — goal-driven practice plans ("I have a gig in 3 weeks — build me a plan"). | L |
| 76 | **AI goal coaching** — realistic goal setting and actionable daily practice plans. | L |
| 100 | **Personalisation** — user preferences and customisation. | — |

---

### Practice — "Play with intention"

Instrument is out. Timer running. The app stays out of the way and supports
focus, not admin.

#### Now (next 4 weeks)

| # | Feature | Size |
|---|---------|------|
| 48 | **Focus mode** — strip the UI to essentials during active practice. Current item + timer only. Navigation hidden but accessible. | M |
| 49 | **Repetition counter** — optional per-item counter tracking consecutive correct repetitions toward a configurable target. | M |
| 52 | **Tempo tracking** — log achieved tempo per item per session, working toward a target BPM. | M |

#### Next (4-12 weeks)

| # | Feature | Size |
|---|---------|------|
| 54 | **Dyslexia-friendly typography** — clean fonts, adequate spacing, sensory-considerate defaults, configurable contrast modes. | S |
| 61 | **Encouragement messaging** — data-tied, process-focused messages. "Your Db mastery went from 2 to 4 over three weeks." Configurable frequency, tone, delivery. | M |
| 62 | **Rest & recovery awareness** — flag when practice volume significantly exceeds historical average. | S |
| 79 | **Calendar view** — calendar-based view of practice sessions. | M |

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
| 65 | **Practice consistency calendar** — comeback framing: "4 of the last 7 days." Celebrate returns, never shame gaps. | M |
| 67 | **Milestone goals & progress dashboard** — percentage completion for active goals. | M |
| 68 | **Weekly practice summary** — comparison to prior week, shown on next app open. | M |

#### Later (12+ weeks)

| # | Feature | Size |
|---|---------|------|
| 64 | **Circle of fifths widget** — interactive key coverage visualisation showing mastery per key. | L |
| 66 | **Tempo progress charts** — tempo increase over time toward target BPM per item. | M |
| 72 | **AI pattern recognition** — identify systematic weaknesses from accumulated data. | L |
| 73 | **AI session review** — post-session analysis with rebalancing suggestions. | M |
| 74 | **Adaptive interleaving** — AI adjusts mixing intensity based on user patterns. | L |
| 75 | **Teacher integration** — shared goals, suggested items, progress visibility (with permission). | L |

---

## Cross-Cutting Concerns

These don't belong to a single pillar — they support all three.

| # | Feature | Horizon | Labels |
|---|---------|---------|--------|
| 99 | **Review use of Crux** — evaluate the core/shell architecture | Now | — |
| 41 | **Offline-first support** — service worker, IndexedDB, sync | Next | architecture |

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
   Building one is non-trivial (accurate timing in WASM). Decision point: before
   tempo tracking (#52) ships.

2. **Offline-first architecture.** Currently API-dependent. What syncs? When?
   Gets harder to retrofit the longer we wait.

3. **iOS native vs web-first.** Web shell is fine for Plan and Track pillars.
   For Practice (audio recording, metronome), native APIs would be significantly
   better. Decision point: before audio recording (#69).

4. **Scoring + tempo coupling.** Should every mastery rating require a tempo?
   Or is tempo optional (only for items with tempo targets)?

5. **Teacher integration timing.** Currently horizon:later. Basic sharing
   (routines, item suggestions) could come earlier without AI.
