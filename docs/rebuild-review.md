# Rebuild vs pivot: the codebase against the practice-coach vision

Assessment date: 19 July 2026. Reviewed against `intrada-practice-coach-design.md`
(v2, 18 July 2026). Read-only analysis; nothing changed.

## Verdict up front

**Pivot in place (option a), with a hard edge.** The parts of this repo that are
expensive and tedious to rebuild (the Crux/FFI toolchain, the Swift shell's
Store and GRDB plumbing, the design system, CI, TestFlight lane, the test
culture) are exactly the parts that survive the new vision. The parts the new
vision replaces (the notebook-style session builder, self-rated scoring, most
of the core's domain content) are inert: they do not block Phase 1, which is
almost entirely additive greenfield. A fresh core would spend two to three
weeks re-proving plumbing that already works, before the first segmentation
lesson is learned. That is the wrong trade for an evenings-and-weekends
project whose stated risk is attempt segmentation, not architecture.

The hard edge: this is a pivot, not a merge. The new engine goes in as its own
quarantined module with its own model state; the web shell and Tauri host get
deleted (not paused); the builder-phase session machinery is scheduled for
deletion at Phase 2, not extension. Roughly 60% of the core's domain code will
eventually go. The recommendation is to let it die in place rather than pay
the plumbing rebuild up front.

One honest caveat on framing: the design doc's line "MIDI events are just
events; the event-sourced model extends naturally" flatters the current core.
It is not event-sourced. It is a command-pattern, mutate-in-place model that
stores sessions as aggregate rows, not event logs. That does not change the
verdict, because attempts and note streams arrive as new append-only data
anyway, but the doc's assumption about what it is extending is wrong.

---

## 1. Inventory

### Headline numbers

| Area | LOC | Tests | Judgement |
|---|---|---|---|
| `intrada-core` (Crux core) | 22,500 | 581 | Mixed: plumbing solid, domain ~60% legacy |
| `intrada-api` (Axum + Turso) | 12,400 (7.9k src / 4.5k tests) | ~190 | Solid code, wrong-era schema |
| `ios/` native Swift shell | 11,000 (+2,900 tests, +7,700 generated) | 184 fns, 71 snapshots | Solid |
| `intrada-web` (Leptos) | 19,600 | disabled in CI | Dead weight |
| `intrada-mobile` (Tauri) | 760 Rust + ~1,100 Swift | n/a | Dead, except ~570 LOC of Swift worth mining |

### The Crux core (`crates/intrada-core`)

What it actually is: a CRUD practice library plus a manual session logger.

- **`Model`**: one flat struct holding items, sessions, sets, practice
  summaries, and a long tail of administrative state (MCP tokens, MCP audit,
  OAuth flow, account deletion). One mega-`ViewModel` (~30 fields) carries
  pre-formatted display strings ("2m 5s", "12 min") for every screen at once.
- **Session lifecycle**: a real state machine, `Idle -> Building -> Active ->
  Summary`, with crash recovery. But its content is a hand-built setlist
  runner: the Building phase is drag-to-reorder block assembly (the exact
  thing "kill the choice" abolishes), the Active phase is a wall-clock timer
  plus a manual rep counter, and Summary collects self-rated 0-10 scores and
  free-text reflections. `domain/session.rs` is 5,530 lines; well over half is
  builder machinery (blocks, grouping, reordering, set-source detection).
- **`domain/chart.rs` (1,163 LOC, 30 tests)**: the sleeper asset. Pitch-class
  chord symbols, a quality taxonomy (Maj7/Dom7/Min7/Min7b5/alt/sus/aug...),
  a strict bar-and-pipe chart parser with per-bar errors, and scaffold
  derivation (shells, guide tones, arpeggios from changes). This is the
  foundation of interval-against-chord labelling and tune-parameterised
  drills.
- **`domain/variant.rs` + ladder plumbing**: exercise "step ladders" with
  per-step scores and a "solid at 8/10" threshold. Structurally this is a
  proto-version of the doc's per-parameter-level mastery; its data source
  (self-rated scores) is what changes.
- **`persistence.rs` (571 LOC)**: a clean typed effect for on-device storage
  (LoadItems/SaveItem/SaveItems/DeleteItem/LoadSessions/SaveSession) with real
  failure outputs (#816) and tombstoned deletes. The pattern, not the ops, is
  the asset.
- **`analytics.rs` (1,554 LOC)**: mastery dials and deltas computed from
  self-rated scores. Superseded by measured scoring.
- **Admin domains** (mcp_tokens, mcp_audit, oauth, account, ~640 LOC plus
  model/view fields): exist in the core so the web UI could manage tokens.
  The native app does not surface them. Server-side concerns living in the
  client core.
- **Test culture**: 581 tests, a large share being bincode FFI round-trip
  guards (the #846 class). That discipline is real value; the volume of guards
  needed is also a standing tax the FFI wire imposes on every crossing type.

### The Axum API (`crates/intrada-api`)

The healthiest code in the repo, built for the previous product:

- **Solid**: auth (PAT + Clerk JWT, battle-hardened), full OAuth 2.1 + PKCE +
  DCR, a hand-rolled MCP server with 13 tools, thin uniform routes, ~190
  well-aimed tests on a real local DB harness.
- **Wrong era**: the schema is the notebook's (items, sessions,
  setlist_entries, routines). No soft deletes, no `updated_at` on session
  tables, 83 migrations showing heavy churn (goals and lessons built then
  dropped wholesale), and every FK removed to work around Turso remote quirks.
  The server schema is *not* sync-ready; only the device store is.
- **Confirmed absent**: LLM proxy, planner, skill graph, scoring engine, MIDI.
  The only "intelligence" is arithmetic aggregation in one MCP tool.
- Meaningful fraction of `state.rs` + retry machinery exists solely to keep
  Turso-over-HTTP alive. Worth remembering when deciding how much the coach
  actually needs a server in Phases 1-2 (answer: nearly nothing).

### The native iOS shell (`ios/`)

Genuinely good, and genuinely a dumb pipe:

- **Store + bridge (solid)**: `@Observable @MainActor` pump, zero domain
  logic, `guarded`/`report` error discipline (no silent no-ops), typed HTTP
  results, degraded-mode fallback, and 10 real-bridge round-trip tests that
  drive the actual Rust core over bincode.
- **GRDB store (solid)**: `item`, `session`, `variant` tables, 9 additive
  migrations with upgrade-path tests, `updated_at` + `deleted_at` on
  everything, JSON-DTO codecs chosen deliberately over bincode for on-disk
  durability. Sync-agnostic by design. This is the offline-first invariant
  actually enforced.
- **Design system (solid)**: fully tokenised paper/score theme, bundled
  fonts, motion tokens, 71 snapshot references locking it.
- **Screens (usable)**: Library (deep: chord charts, ladders, scaffold flow),
  the full build-play-reflect-save session loop, Progress. Routines tab is
  the one stub. The catch: most screens render the notebook workflow the new
  vision replaces, so their fate is tied to the pivot, not to their quality.
- **Confirmed absent**: any audio. No CoreMIDI, no AVAudioEngine, no
  AVAudioSession, no metronome, no click. The only timing is a visual
  count-up ring. The entire listening layer is greenfield.

### Web, Tauri, CI

- **`intrada-web` (19,600 LOC)**: disabled in CI ("allowed to drift"), no
  back-coupling into the core (deleting it needs zero core surgery). Pure
  dead weight for this future. The only value is behavioural reference.
- **`intrada-mobile`**: Tauri host and auth plugin are dead. Two Swift
  implementations are worth mining before deletion:
  - `background-audio` plugin (327 LOC): `AVAudioSession(.playback,
    .mixWithOthers)`, silent-loop keep-alive, interruption re-arm,
    `MPNowPlayingInfoCenter`. This is exactly the audio-session groundwork a
    click track needs, and the native app has none of it.
  - `live-activity` plugin (+ widget extension): working ActivityKit Lock
    Screen timer.
- **CI**: mature and already pivoted; all web/Tauri jobs are `if: false`.
  Active path: Rust gates + native-ios (macos-26) + API deploy to Fly. The
  TestFlight lane works.

---

## 2. Gap analysis against the design doc

Classification: (a) directly reusable, (b) adaptable with modest work,
(c) irrelevant but harmless, (d) actively in the way.

| Requirement | Existing code | Class | Notes |
|---|---|---|---|
| Session state machine (gated blocks, done-criteria, stuck ladder) | `SessionStatus` lifecycle + crash recovery | **b** | The lifecycle skeleton (phase enum in model, event routing, recovery via `AppEffect`, the shell's `PlayerHost` pattern) transfers. The *content* of each phase is replaced: prescribed plan supplants Building, gate loop supplants timer+reps. |
| ...the Building-phase machinery specifically | ~3k LOC of block/reorder/set-source logic in `session.rs`, plus `SessionBuilderScreen` (765 LOC Swift) | **d** | This is the hand-curation workflow "kill the choice" abolishes. Extending it toward prescription would fight the model at every step. Quarantine now, delete at Phase 2. |
| MIDI event stream ingestion | Nothing, anywhere | none | Greenfield. Crux can carry it, but not naively; see §3 on batching. The Tauri background-audio Swift is the audio-session starting point. |
| Attempt segmentation | Nothing | none | Pure-Rust greenfield; the core is a fine home. Zero legacy interference. |
| Deterministic scoring | `score: Option<u8>` self-ratings; analytics built on them | **c/d** | Harmless as history (and useful as mastery priors), in the way if the new mastery model is bolted onto the same fields. New `Attempt`/score types must be separate from the self-report path. `analytics.rs` is superseded. |
| Skill graph (nodes, prereqs, mastery as estimate+confidence) | Nothing. `Item {Piece, Exercise}` is a flat library | **c** | Items stay as the library (tunes need somewhere to live, and `chord_chart` hangs off them). Do not stretch `Item` into `Skill`/`Drill`; new types. The variant ladder is the one shape worth copying (per-level state). |
| Phrase / Device / Tune / Drill / MethodPack model | Only `Tune`-adjacent: `Item(Piece)` + `ChordChart` | **a** (chart) / none (rest) | `chart.rs` is directly reusable and needed on day one of lick-transposition scoring (target notes against changes, per-key transposition). `KeyHelper.swift` + key/modality handling also reusable. |
| Gate criteria as data (JSON/TOML) | Nothing | none | Greenfield; `validation.rs` shows the core's existing data-checking idiom. Nothing in the way. |
| Session planner as pure function | `Set`/routines: static user-authored lists | **c** | Nothing to adapt. Note §5: the doc puts the planner behind Axum; the codebase's offline-first architecture argues it belongs in the core. |
| LLM proxy | Nothing, but the Axum app is a ready chassis | **b** | Auth, rate limiting, Sentry, deploy already exist. A `/api/coach` route is a small PR when Phase 3 arrives. The MCP server is a bonus surface for the same era. |
| Persistence for new entities | GRDB store + `PersistenceOperation` pattern | **a** (pattern) / **b** (ops) | Sync-ready schema discipline already enforced on-device. Attempts, phrases, graph state follow the existing migration + codec pattern. |
| Click / count-in / background audio | Tauri `background-audio` Swift (in the dead crate) | **b** | Port into `ios/`; the AVAudioSession + interruption logic is the hard-won part. Actual click scheduling (sample-accurate) still to build. |

The overall shape: the new vision's hard requirements are ~90% absent, ~10%
present as high-quality foundations (chord theory, persistence discipline,
lifecycle skeleton). Almost nothing must be *removed* before Phase 1 can
start; the (d)-class code only starts fighting you at Phase 2, which is when
it should be deleted.

---

## 3. The architecture question

**Can the current core absorb a MIDI stream and the new session machine?**
Mechanically yes; the Crux effect system is genuinely extensible and the
persistence effect proves the pattern (typed ops, real failure outputs, shell
as executor). But three structural facts need naming, because they shape how
the absorption must be done:

1. **The core is not event-sourced, and the update loop is view-rebuild-per-
   event.** Every `Event` triggers `update` and then a full `ViewModel`
   rebuild (the whole library with practice summaries included). Piping raw
   note-on/note-off through that path at playing speed would be architecture
   abuse. The right shape: the Swift shell owns the CoreMIDI ring buffer and
   timestamps against the click clock; note events cross the FFI bridge in
   batches (per beat, per few hundred ms, or per attempt window), and the
   core does segmentation and scoring on batches. Deterministic, testable,
   and the bridge crossing stays low-frequency. The doc's "MIDI events are
   just events" is true at the data-model level, false at the wire level.

2. **The bincode FFI wire is fragile by construction** (#846 class: positional
   format, serde-attr landmines, silent decode failures). The codebase has
   the antibodies (round-trip guards, `LiveBridge` tests), which is exactly
   why new bridge-crossing types for the engine must be few, small, and
   stable: one `NoteEvent`, one batch envelope, one gate-progress view. Do
   not let the engine grow a chatty FFI surface.

3. **Load-bearing vs legacy, named:**
   - *Load-bearing, keep*: `Effect`/`AppEffect` definitions, the
     `PersistenceOperation` pattern, `SessionStatus` lifecycle + recovery,
     `Item` + `ChordChart` + `Variant` and their codecs, `validation.rs`,
     the typegen/UniFFI build recipe, every round-trip test helper.
   - *Legacy, quarantine then delete*: Building-phase machinery and its
     event vocabulary (ReorderBlock, KeepOnlyPiece, UngroupAllBlocks...),
     `Set` domain, `analytics.rs`, the temp-id online mutate-response paths,
     MCP/OAuth/account state in the client core, the mega-`ViewModel`'s
     notebook fields.
   - *Standing tax to cancel*: invariant 6 (every domain handler preserved
     and tested in both `local_first` and online modes) exists to keep the
     paused web shell alive. Once web is deleted, new engine code should be
     local-only, and the dual-mode requirement should be retired rather than
     inherited.

**Is extension harder than restarting the core?** No, and the margin is not
close, for one reason: the expensive part of this core is not its domain code
(which is ordinary Rust, quick to write), it is the proven bridge toolchain
(facet typegen + UniFFI + the Xcode 26 MainActor workaround + gen-stamp
automation), the GRDB integration, and the test harnesses. A restarted core
re-pays all of that before learning anything about segmentation. The
notebook-shaped abstractions are a drag on Phase 2 ergonomics, not a wall,
and they are deletable incrementally because the shell is a true dumb pipe:
when a domain dies in the core, the Swift that renders it dies cleanly with
it.

---

## 4. Recommendation

**(a) Pivot in place**, structured as: new engine module(s) beside the old
domains, aggressive deletion of the dead shells now, scheduled deletion of
the notebook session machinery at Phase 2.

Justified from the inventory, not vibes:

- The **solid** column of the inventory (iOS Store/bridge/GRDB/theme, core
  plumbing and chart theory, API auth, CI/TestFlight) is precisely the
  option-b/c casualty list. Fresh-core (b) keeps the Swift files but breaks
  their contract: every generated binding changes, all 10 real-bridge tests
  and 71 snapshots re-anchor, and the toolchain must be re-proven. Fresh
  repo (c) additionally throws away working CI, the sim tooling, and the
  TestFlight lane for zero domain benefit.
- Phase 1 (segmentation, MIDI capture, lick-transposition scoring) touches
  the legacy domains almost nowhere, and its two genuine dependencies that
  exist anywhere in the repo (chord/key theory, background-audio session
  handling) are both in the keep column.
- The app currently works and is Jon's daily practice tool. A pivot keeps a
  usable app through the transition; both rewrite options go dark for weeks.

**Biggest risk of the recommended path: gravity.** The notebook abstractions
are *there*, so the temptation is to reuse `Item.variants` for drill levels,
hang attempt scores off `ItemPracticeSummary`, thread gate progress through
the mega-ViewModel, and extend `SessionBuilderScreen` instead of replacing
it. Six months of that and you have the worst of both worlds: the old model
wearing the new one as a costume. Mitigations, concrete: the engine gets its
own model sub-struct and its own ViewModel field; new mastery/score types
never touch the self-report fields; the Phase 2 PR that lands the planner
deletes the builder in the same series, not "later".

**Biggest risk of the rejected path (fresh core, b):** three-ish weeks of
plumbing rebuild (typegen, UniFFI quirks, GRDB re-integration, CI
re-anchoring) before the first attempt-segmentation lesson, on a project
whose own design doc names segmentation, not architecture, as the risk that
can kill it. For evenings-and-weekends capacity, that is the whole Phase 1
budget spent producing zero Phase 1 knowledge, and it deletes a working
practice tool during exactly the fortnight (Phase 0) Jon needs to be
practising daily with logs.

---

## 5. Where the doc is wrong and the code is right

The doc asked for this review to run both ways. Three findings:

1. **Planner placement.** The doc's table puts "Planning and prescription" in
   "Backend Rust, state in Turso". The codebase's own offline-first
   invariants (device store as source of truth, no network on the local
   path, reconciliation in core) are the better architecture, and the doc
   itself elsewhere wants "session planner as a pure function". A pure
   function of (graph state, pipeline states, minutes) belongs in
   `intrada-core`, running on device, with Turso as later sync/backup. The
   practice room is the worst place to depend on a server round-trip, and
   the API inventory shows how much code Turso-over-HTTP already costs in
   keep-alive workarounds. Recommendation: planner in core; Axum keeps only
   the LLM proxy (Phase 3) and eventual sync.

2. **"The event-sourced model extends naturally."** Covered in §3: the model
   is command-pattern with aggregate persistence, not event-sourced. The
   extension is still natural, but as new append-only *data* (attempts,
   note-event blobs per attempt), not because event machinery exists.

3. **Server schema assumptions.** Any Phase 2+ plan that says "state into
   Turso" inherits a server schema with no tombstones, no `updated_at` on
   session tables, and heavy migration churn. The device store is the
   sync-ready one. When sync arrives, derive the server schema from the
   device schema, not the reverse, and treat the existing Turso tables as
   legacy alongside the goals/lessons ruins.

One smaller note: the doc's data-model sketch omits practice history.
The existing session archive (real dates, durations, per-item self-scores)
is the only data available for seeding mastery priors and the "your trend"
horizons. Keep it readable from the new model even after the notebook UI
that produced it is gone.

---

## 6. First two weeks (Phase 1 opening)

Aligned to the doc's ordering: segmentation spike before any scoring UI, MIDI
capture feeding it, lick transposition as the first scored drill. Phase 0
(paper teacher) runs in parallel by design; it is practice time, not code
time. One resequencing within the spike: the segmentation spike needs real
playing data, so a thin capture harness lands first; it is a day or two of
low-risk work, whereas segmentation is the named risk and deserves real
fixtures from the start.

**PR 1: clear the decks** (days 1-2)
Delete `crates/intrada-web`, `crates/intrada-mobile`, and the disabled CI
jobs; first, mine `BackgroundAudioPlugin.swift` and the live-activity Swift
into `ios/` (as a reference folder or straight into the app target). Remove
`worker.js`/`wrangler.toml`. Retire the dual-mode (online/local-first)
test requirement for future engine code; leave existing handlers untouched.
Outcome: the repo states the truth, CI shrinks, and every subsequent core
change stops paying the both-modes tax. No behaviour change to the shipping
app.

**PR 2: MIDI capture harness + click** (days 3-7)
Swift only, debug-gated. A `MidiCapture` service on CoreMIDI (device connect,
timestamped note events), an `AVAudioSession` setup ported from the mined
background-audio plugin, a minimal click track with count-in, and a hidden
debug screen that records a take and writes JSONL (events timestamped
against click start) to Files. No core changes, no FFI changes. Outcome: Jon
can record real attempts at the piano; the recordings are the spike's test
fixtures. (This is also where the "which cable, which piano" MIDI-setup
reality gets its first contact.)

**PR 3: attempt segmentation spike** (days 8-14)
Pure Rust: a new `engine` module in `intrada-core` (no dependency on `Model`,
no FFI surface yet). Types: `NoteEvent`, `ClickGrid` (bpm, count-in, start
anchor), `Attempt`. Functions: count-in detection, attempt start/end
detection, restart detection, noodling rejection, collapse handling. Tested
against committed JSONL fixtures from PR 2 recordings (clean take, restarted
take, noodle-then-play, mid-attempt collapse). Deliverable includes a short
findings note: what segmentation can and cannot decide from timing alone,
which becomes the spec for the scoring gate. This PR is the doc's named
spike; if it fails, it fails cheaply and in Rust, not in UI.

**Week 3+ (named for orientation, not part of the fortnight):** lick
transposition scoring in the engine (`Phrase` with per-key targets, wrong
note / timing per bar against the `ClickGrid`, using `chart.rs` for chord
context), then the first bridge crossing (batched capture in, gate progress
out) and the one drill screen with the tick.

Suggested tiering: PR 1 is Tier 1 by content but ships via `ship` anyway
given its breadth; PR 2-3 are Tier 2 with the design doc standing in as the
Tier 3 spec for the phase.
