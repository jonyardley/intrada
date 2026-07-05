# Related Exercises — design reconciliation (native iOS)

> Tier 3 spec. Rides with the Phase 1 branch (`feat/related-exercises-phase-1-foundation`).
> Design is canonical in `design/Linked Exercises.dc.html` + `design/intrada-design-system.dc.html`.

## Problem

The "Related exercises" feature is fully designed but the native iOS app only
partially matches it. A piece links to the exercises practised alongside it; the
link should surface as a managed card on the piece detail, a provenance
breadcrumb on the exercise detail, a movable **block** in the session builder,
and a 0–10 **score ring** + per-session history everywhere scoring happens.

## What's already shipped (do not rebuild)

On `main` (HEAD `5b1c6a6`, PRs #1015 / #1020 / #1022–#1024):

- **Core is complete for this feature.** 1–10 score (`validate_score`),
  bidirectional piece↔exercise links (`LinkExercise` / `UnlinkExercise` /
  `ReorderLinkedExercises`), favourites (`priority`), reflection events
  (`UpdateEntryScore` / `UpdateEntryNotes`), and session-block grouping
  (`group_id`, `build_blocks`, drag-as-a-unit) — all with tests. Session
  duration projections (`total_duration_display` / `total_duration_summary`)
  exist on `BuildingSetlistView`.
- **iOS**: `ScoreRing` / `ScoreSelector` components, ring tokens (`masteryFill`,
  `masteryTrack`, `successTeal`, `addDashOutline`), `dumbbell.fill` badge,
  `LibraryDetailScreen` related-exercises card, `SessionSummaryScreen` on
  `ScoreSelector`.

**Consequence: this whole feature is now Tier-2 SwiftUI/token work — no core
changes, no GRDB migrations, no FFI-bridge widening** (see Decisions). The single
bridge-crossing write anywhere in the plan is wiring `updateEntryNotes` from
Swift (Phase 6), which needs a real-bridge round-trip test (#846 class).

## Decisions (locked 2026-07-01)

All chose the no-migration path, keeping the offline tier's only-copy-of-the-data
safe:

1. **Genre chip → reuse `tags`.** No dedicated `genre` field (would be a
   migration + upgrade-path test).
2. **RecentSessions → drop the per-session note line.** Show score + date +
   trend only. `ScoreHistoryEntry` gains no note field.
3. **Session-builder per-exercise "include today" toggle → dropped.** No core
   backing; swipe / Edit removal covers "not today".
4. **Reflection hand-off sheet → ScoreSelector + note only, no live ring.** The
   ring stays on the detail screens.

## Phases (each a PR-sized branch)

| Phase | Scope | Depends on |
|---|---|---|
| **1 · Foundation** | Shared primitives (this branch). | — |
| **2 · Library header + rows** | Already shipped in prior work — "All ▾" `LibraryFilterMenu` dropdown, favourite-star pill, header divider + opaque reveal, rows with ring + inline star + tag chips. This phase only reconciles the remaining fidelity delta: the ScoreRing numeral serif→Inter (`IntradaFont.scoreNumeral`). Dead `LibraryFilterTabs` removal spun off separately. | 1 |
| **3 · Piece/exercise detail** | Wire `RecentSessions` into both details (map `practice.scoreHistory`; RFC3339→"Tue · Jun 24"); exercise-detail 132px hero ScoreRing + gold "Related to [piece]" breadcrumb (primary + "· +N more", replacing the old related-pieces list). Deferred: "Practise this" CTA (#1034, no single-item practice entry point) + full multi-piece breadcrumb nav (#1035). | 1 |
| **3b · iPad SplitView** | List↔detail split, built with the screens. | 2, 3 |
| **4 · Picker** | Reframed `LinkedExercisePickerSheet` from add-only "Add N" to an add/remove **manager** (lists all exercises, pre-selects related, Done applies the link/unlink diff) with the gold circular +/check control, "Your exercises" eyebrow + gold type bars. Deferred (#1037): shared `BottomSheet` extraction (→ Phase 5's second consumer), filter-bar chrome + selected tray, and pruning the card's now-redundant Edit-mode remove. | 1, 3 |
| **5 · Session builder** | **Re-layout** `SessionBuilderScreen` from browse-first (library always visible + queue tray) to the design's dedicated **"Build session"** serif list (Frames 10/13/14): a dashed "Add piece or exercise" row opens an **"Add to session" BottomSheet** (extract the shared `BottomSheet` here, #1037 — its second consumer); sticky full-width **"Start session · MM min"** gradient bar (`totalDurationDisplay`); "Editing" subtitle + **swipe-to-remove** (row→`removeFromSetlist`, group→`removeBlock`) alongside an **Edit/Done** toggle replacing the permanent `.editMode(.active)`; group-pill + warm block header + gold nested bars; "Standalone exercise · N min" styling. Preserve #1024's stable-identity `ForEach` keying (first-entry ulid). **Done** as Tier-2: new `AddToSessionSheet` (browse+add) via the dashed row; sticky start bar; Edit/Done + swipe-remove; grouped-block restyle. **Corrected:** `BuildingSetlistView` has no total-duration field (it's on `PracticeSessionView`), so the header/summary + start bar use **item/block counts**, not "· MM min" — duration deferred to #1040 (core field). BottomSheet extraction still #1037; include-toggle dropped; in-group add deferred. | 1 |
| **6 · Focus reflection** | **Done.** New `ReflectionSheet` (Frame 9: "Item complete · [elapsed]" / "How did [item] go?" / `ScoreSelector` + optional note / "Save & continue" · "Skip rating"; selector-only per decision). The Focus player's advance button now presents it; on Save the order is **notes → NextItem → score** (score needs the entry `Completed`; notes have no status guard so they surface a validation error before advancing; dismiss only on success, #846). `updateEntryNotes` was never sent from Swift — now round-tripped through `LiveBridge` in `StoreEffectLoopTests`. Summary already on `ScoreSelector` (Phase 1). | 1 |

## Phase 1 — Foundation (this branch)

Shared primitives every later screen consumes, so screens stay declarative.

- **`ScoreSelector`** reshaped from 18pt circles to the 1–10 **pill row**
  (32pt tall, `badge` radius, cumulative accent fill, unfilled = `slotOutline`
  border + `inkSecondary`, selection haptic, VoiceOver adjustable). Re-records
  the two `SessionSummaryScreen` snapshots (its only live consumers).
- **`RecentSessions`** (new): eyebrow + green trend chip (`successTeal`, "5 → 7")
  over rows of a 38pt `ScoreRing` + date. Pure presentation; the caller (Phase 3)
  maps `scoreHistory` (RFC3339 dates) to `RecentSession` rows. Note line dropped
  (Decision 2).
- **`ScoreRing`** additive hero variant (`showsScale`) — "OF 10" caption under
  the numeral for the exercise-detail hero. Existing rings unchanged.
- **`AddRowButton`** gains `.outlined` (empty-state CTA) and `.plain`
  (borderless footer) styles alongside the default `.dashed`.
- **Core**: characterization test that a mid-session per-entry score survives
  into the `Summary` projection (the reconciliation Phase 6 relies on). No code
  change — proves existing behaviour.

**Deferred out of Phase 1 (tracked):**

- **Ring numeral font** — the design draws the ring numeral in Inter; Phase 1
  shipped it in Source Serif (`pageTitle`). **Done in Phase 2**
  (`IntradaFont.scoreNumeral`, Inter semibold — no bold instance is registered,
  so the design's 700 maps to semibold).
- **`BottomSheet` / shared filter-bar chrome** — extract it in **Phase 4**, its
  first real consumer, per "consolidate before you template" (second use, not
  pre-emptively).

## Testing

- Snapshot tests: `testScoreSelectorPills`, `testScoreRingHero`,
  `testRecentSessions`, `testAddRowButtonVariants`; re-record
  `testSessionSummaryCompleted` / `testSessionSummaryEndedEarly`. Run
  `just ios-snapshots-optimize` before committing; delete orphans.
- Core: `test_mid_session_entry_score_survives_into_summary` in
  `domain/session.rs`.
- Coverage: full for the core test; UI primitives covered by snapshots
  (WASM/iOS shell is Codecov-ignored).

## Offline-first check (per CLAUDE.md PR checklist)

No persistence, sync, or new domain entity touched by this feature after the
decisions above — reads/writes already go through shipped core events, entities
already carry client-owned ulids + `updated_at`/`deleted_at`, reconciliation is
already core-side. Web/API keep compiling (no shared-type change).

## Open questions (later phases, not blocking Phase 1)

- Favourites filter: shell-side vs a persisted core query (Phase 2). Prefer
  shell-side.
- Inline favourite star vs a top "PRIORITIES" section — coexist or replace (Phase 2).
- Exercise "Related to [piece]" when several pieces link one exercise —
  `linked_from_pieces` is a `Vec`; show first/primary or a list (Phase 3).
- Focus-player chrome (Frame 8: rep tracker, timer ring) — separate, de-scopable
  phase (6b); confirm core exposure before assuming any projection exists.
