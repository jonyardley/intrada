# intrada — Product Roadmap

*Updated 2026-04-25*

Everything in intrada serves the five layers described in the [Product Vision](../VISION.md):
**Capture → Plan → Space → Show → Guide**. These layers build on each other — you
can't schedule what you haven't captured, you can't space what you haven't scheduled,
and you can't show progress on what you haven't tracked.

The roadmap is also organised around three activity pillars — **Plan** (decide what to
practise), **Practice** (play with intention), and **Track** (see the process working) —
which cut across the layers. Features are placed on a rolling horizon: **Now** (next
4 weeks), **Next** (4–12 weeks), and **Later** (12+ weeks).

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
| Tauri/Leptos iOS shell — Phase 0 (#302) — scaffold, CI, CLAUDE.md, simulator running | Done |
| SwiftUI shell removed — committed to single Leptos UI codebase across web + iOS | Done |
| Remove category field (#211) — exercises use tags only, simplified subtitle logic | Done |
| Use mutate response (#148) — updates/deletes use API response directly, no re-fetch | Done |
| Filter clause sync (#152) — extracted subquery constants to prevent user isolation drift | Done |

---

## Prioritisation Model

Features are prioritised by **vision layer first, then pillar**. Layer 1 (Capture)
features take precedence over Layer 2 (Plan), which take precedence over Layer 3
(Space), and so on. Within a layer, the three pillars advance independently.

The question to ask is: "What's the most important thing I haven't captured yet?"
before "What's the smartest way to schedule it?"

---

## Three Pillars

### Plan — "Decide what to practise"

Before the instrument comes out. Building sessions, organising the library, and
eventually letting the app decide for you.

#### Now (next 4 weeks)

| # | Feature | Size | Vision Layer |
|---|---------|------|-------------|
| 46 | **Multi-key practice** — assign multiple keys to items, track mastery per key independently. | XL | 1: Capture |
| 267 | **Teacher assignment capture** — quick-capture flow optimised for post-lesson entry. Structured fields for exercise type, keys, voicing details, teacher notes. Designed to replace the notebook. | M | 1: Capture |
| 212 | **Short session presets** — 10/15/20/30 minute options. | S | 2: Plan |

#### Next (4-12 weeks)

| # | Feature | Size | Vision Layer |
|---|---------|------|-------------|
| 50 | **Section/passage management** — break pieces into practisable sections with independent mastery scores and tempo targets. | L | 1: Capture |
| 53 | **Full-text search** — search across all practice notes to find recurring themes. | M | 1: Capture |
| 45 | **Archive/retire workflow** — handle completed/mastered items so the active library stays focused. | M | 1: Capture |
| 57 | **One-tap session start** — open the app, tap Start, play. Initial version uses recent routine + items not practised recently + decayed mastery scores. Upgraded when spacing engine ships. | L | 2: Plan |
| 142 | **Session time budgeting** — declare available time, allocate across items automatically or manually. | M | 2: Plan |
| 59 | **Session planning** — input available time, get a structured plan with warm-up, focused work, and review segments. | L | 2: Plan |
| 58 | **Mastery decay model** — scores decrease over time without review, creating natural scheduling urgency. Gives one-tap start smarter item selection without the full spacing engine. | M | 3: Space |

#### Later (12+ weeks)

| # | Feature | Size | Vision Layer |
|---|---------|------|-------------|
| 55 | **Spaced repetition engine** — modified SM-2 algorithm for optimal review intervals based on mastery scores and time since last practice. First feature built when Later begins. | XL | 3: Space |
| 56 | **Interleaved setlist generator** — mixed-type sessions with configurable interleaving intensity. | L | 3: Space |
| 42 | **Links to IMSLP** — link pieces to free sheet music on IMSLP. | S | 1: Capture |
| 43 | **Open-source exercise library** — built-in scales, arpeggios, Hanon, Czerny etc. | L | 1: Capture |
| 71 | **AI setlist generation** — goal-driven practice plans ("I have a gig in 3 weeks — build me a plan"). | L | 5: Guide |
| 76 | **AI goal coaching** — realistic goal setting and actionable daily practice plans. (blocked on goals redesign) | L | 5: Guide |
| 100 | **Personalisation** — user preferences and customisation. | — | — |

---

### Practice — "Play with intention"

Instrument is out. Timer running. The app stays out of the way and supports
focus, not admin.

#### Next (4-12 weeks)

| # | Feature | Size | Vision Layer |
|---|---------|------|-------------|
| 61 | **Encouragement messaging** — data-tied, process-focused messages comparing current ratings to recent history. | S | 4: Show |
| 54 | **Dyslexia-friendly typography** — clean fonts, adequate spacing, sensory-considerate defaults, configurable contrast modes. | S | Cross-cutting |
| 62 | **Rest & recovery awareness** — flag when practice volume significantly exceeds historical average. | S | 4: Show |
| 166 | **Session tempo targets** — suggest incremental tempo targets based on recent progress toward the item's target BPM. | M | 2: Plan |

#### Later (12+ weeks)

| # | Feature | Size | Vision Layer |
|---|---------|------|-------------|
| 69 | **Audio recording & playback** — record a run-through, play it back, compare to weeks ago. | L | 4: Show |
| 70 | **Customisable encouragement preferences** — frequency, tone, delivery controls. Respects neurodiversity. | M | 4: Show |

---

### Track — "See the process working"

After the session. Analytics, insights, visualisation. Evidence that the practice
is actually working.

#### Next (4-12 weeks)

| # | Feature | Size | Vision Layer |
|---|---------|------|-------------|
| 63 | **Mastery timeline charts** — per-item and aggregate line charts showing mastery improvement over weeks and months. | L | 4: Show |
| 79 | **Calendar view** — calendar-based view of practice sessions. | M | 4: Show |

#### Later (12+ weeks)

| # | Feature | Size | Vision Layer |
|---|---------|------|-------------|
| 64 | **Circle of fifths widget** — interactive key coverage visualisation showing mastery per key. Depends on multi-key (#46). | L | 4: Show |
| 72 | **AI pattern recognition** — identify systematic weaknesses from accumulated data. | L | 5: Guide |
| 73 | **AI session review** — post-session analysis with rebalancing suggestions. | M | 5: Guide |
| 74 | **Adaptive interleaving** — AI adjusts mixing intensity based on user patterns. | L | 5: Guide |
| 75 | **Teacher integration** — suggested items, progress visibility (with permission). | L | 5: Guide |

---

## Mobile Shell (Tauri/Leptos)

The SwiftUI iOS shell (#187–#202) is on hold. Active iOS development now goes
through the Tauri 2 + Leptos shell. See `specs/tauri-leptos-ios-shell.md`
for the full plan.

### Done
| Phase | Description | PR |
|-------|-------------|-----|
| Phase 0 | Scaffold, CI, CLAUDE.md, simulator running | #302 |

### Now
| Phase | Description | Size |
|-------|-------------|------|
| Phase 1a | Foundation CSS — iOS reset, SF Pro, safe areas, no-zoom inputs | S |
| Phase 1b | Interactivity — View Transitions, haptics, spring animations | M |
| Phase 1c | `<SplitView>` primitive for iPad list→detail layouts | M |

### Next
| Phase | Description | Size |
|-------|-------------|------|
| Phase 2a | Component audit — classify all views/components for iOS parity | S |
| Phase 2b | iOS-shaped component variants — sheets, segmented control, list rows, pull-to-refresh | L |
| Phase 2c | Re-skin Practice pillar — focus mode, active session, scoring, rep counter | L |
| Phase 2d | Background audio Swift plugin (P0 — lock-screen timers) | M |

### Later
| Phase | Description | Size |
|-------|-------------|------|
| Phase 3 | Plan + Track pillars at parity | L |
| Phase 4 | Dogfood on physical device (Xcode sideload) | S |
| Phase 5 | Shell-of-record cutover (CLAUDE.md, on-hold markers) | S |
| [#300](https://github.com/jonyardley/intrada/issues/300) | TestFlight enablement | S |
| [#301](https://github.com/jonyardley/intrada/issues/301) | App Store submission | M |
| [#299](https://github.com/jonyardley/intrada/issues/299) | MusicKit integration | L |
| [#303](https://github.com/jonyardley/intrada/issues/303) | CSP hardening | S |

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
| `layer:capture` | Vision Layer 1 — capture and remember |
| `layer:plan` | Vision Layer 2 — plan what to do today |
| `layer:space` | Vision Layer 3 — space it for retention |
| `layer:show` | Vision Layer 4 — show that it's working |
| `layer:guide` | Vision Layer 5 — identify gaps and guide |
| `horizon:now` | Next 4 weeks |
| `horizon:next` | 4-12 weeks |
| `horizon:later` | 12+ weeks / future |
| `architecture` | Technical debt, infrastructure |
| `ux` | User experience (cross-cutting) |
| `accessibility` | Neurodiversity and accessibility (cross-cutting) |

### Project board

Backlog > Ready > In Progress > In Review > Done

The board columns are workflow states, not categories. Use the pillar, layer, and
horizon labels to filter and slice the board.

### Prioritisation

**Vision layer first, then pillar.** Layer 1 features take precedence over Layer 2,
and so on. Within a layer, "What's the most important Plan feature?" is a better
question than "What phase are we in?" Within each pillar, the Priority field
(P0/P1/P2) ranks items.

---

## Open Questions

1. **Metronome: built-in or external?** The tempo-building loop needs a metronome.
   Building one is non-trivial (accurate timing in WASM). Tempo tracking (#52) has
   shipped — a built-in metronome would complete the tempo practice workflow.

2. **Offline-first architecture.** Currently API-dependent. What syncs? When?
   Gets harder to retrofit the longer we wait.

3. ~~**iOS native vs web-first.**~~ **Decided: Tauri/Leptos is the primary
   channel for both web and iOS.** The SwiftUI shell is on hold. The Leptos
   shell runs in a Tauri WKWebView on iOS, shipping features on both platforms
   simultaneously. See `specs/tauri-leptos-ios-shell.md`.

4. **Scoring + tempo coupling.** Should every mastery rating require a tempo?
   Or is tempo optional (only for items with tempo targets)?

5. **Teacher integration timing.** Currently horizon:later. Basic sharing
   (routines, item suggestions) could come earlier without AI. The new
   "teacher assignment capture" feature (#267) addresses the immediate
   capture problem without requiring teacher-facing features.

6. **Goals redesign.** The goals feature has been removed for a ground-up rethink.
   The five vision layers don't depend on goals in the short term — Capture, Plan,
   Space, and Show all work without them. Goals become important when reaching
   Layer 5 (gap analysis, pathways). Deferring is intentional, not an oversight.
