# Piece-linked exercises

> Tier 3 lightweight spec. Status: **shipped** (#1015, merged 2026-07; design
> handoff 2026-06-29). **Scope: `intrada-core` + native iOS only.**
> Web/API (Turso) is out of scope for now (see Deferred).
> Design mock: [`design/linked-exercises-mock.dc.html`](../design/linked-exercises-mock.dc.html).

## Problem

Users create pieces and exercises as standalone library items, but nothing
relates them, and there's no way to see per-exercise progress in the context of a
piece. A classical piece has associated scales/arpeggios; a jazz standard has a
progression of practice activities (learn the tune, learn the shells, runs to the
3rd of each chord, improvise). The practitioner wants to see "the things I
practise for this piece" in one place **and see how each one is scoring**.

## Goal

A piece carries an ordered group of linked exercises, each showing its latest
0–10 practice score (as a ring), so the user sees at a glance how each drill for
the piece is progressing and can jump to any of them. From an exercise, they can
see which pieces it's linked from.

## Build sequence (layered — decided 2026-06-29)

Three dependent phases, each its own PR, in order:

1. **#1008 — scoring → 0–10 + overall session score.** The data foundation the
   ring renders.
2. **#1009 — score ring** replaces the `MasteryMeter` stepped bars everywhere a
   score appears (library rows, exercise detail).
3. **Linked exercises** (this spec) — builds on the ring and the 0–10 score.

This spec covers Phase 3; #1008/#1009 carry their own scope (finalised design
recorded as comments on those issues).

## Non-goals

- **No group practice run.** Tapping a linked exercise opens that exercise (and
  its normal scored practice). No "practise the whole group" flow; the session /
  Set / timer machinery is untouched.
- **No gated stages.** Ordered, but nothing is locked behind a prior exercise.
- **No separate manual status.** Tracking *is* the per-item 0–10 score — no
  Learning/Solid/Performance-ready axis (considered and dropped).
- **Exercises are not owned by a piece.** First-class, reusable; the same
  exercise can be linked to many pieces; its score is global to the exercise.
- **Online/web is out of scope** (see Deferred).

## Key decisions

1. **Shared & reusable links, not ownership.** Many-to-many; an exercise's edits
   and score are live everywhere it's linked.
2. **Ordered id list stored on the piece** — `Item.linked_exercise_ids:
   Vec<String>`, resolved to live exercises in the core view function. Chosen
   over a dedicated link entity (more machinery) and over reusing Sets/routines
   (not local-first yet; conflates "routine" with "a piece's exercises").
   Trade-off: whole-list LWW if the same piece's list is edited on two devices at
   once — fine at current single-user scale.
3. **Tracking reuses the per-item 0–10 score rollup** (`practice.latestScore`) —
   no new per-item state. Row shows **title + key/tempo + ring** only (no
   last-practised / best-tempo clutter — design decision).
4. **Reverse "Linked from" view is in scope** (was deferred; the design includes
   it). The exercise detail lists pieces that link the exercise — a core
   computation scanning pieces, no schema change.

## Data model (intrada-core)

- `Item` gains one additive field (`#[serde(default)]`):
  - `linked_exercise_ids: Vec<String>` (only populated for pieces).
- Events:
  - `LinkExercise { piece_id, exercise_id }`
  - `UnlinkExercise { piece_id, exercise_id }`
  - `ReorderLinkedExercises { piece_id, ordered_ids }`
  Each mutates the piece, bumps `updated_at`, persists via the persistence
  `Effect` (local-first path). See Offline-first re: the online branch.
- Validation (`validation.rs`): link target must be an existing `Exercise`; host
  must be a `Piece`; no duplicate ids; no self-link; missing id rejected.
- ViewModel:
  - For pieces, `linked_exercises: Vec<LinkedExerciseView>` — resolved from the
    ids → live title / key / tempo / kind **+ the `practice` rollup** (latest
    score), filtering out tombstoned/missing. A soft-deleted exercise drops out.
  - For exercises, `linked_from_pieces: Vec<PieceRefView>` — pieces whose
    `linked_exercise_ids` contains this exercise (computed; live).

## Offline-first / invariants

- Reads/writes go through the persistence `Effect`, not HTTP (invariant 1).
- The piece is the synced entity; the link list rides its `updated_at` +
  `deleted_at` — no separate entity (invariant 2).
- Client-owned ids; links reference existing ids (invariant 3).
- Reconciliation stays in the core (LWW on the piece) (invariant 4).
- **Invariant 6 (both modes) consciously deferred for the new handlers**:
  implement and test `local_first` now; the online branch + API column land with
  the Deferred web/API work. New events must still compile cleanly with existing
  online plumbing.

## Persistence (iOS only)

- **iOS GRDB:** append-only migration `vN_item_linked_exercise_ids` adds
  `linked_exercise_ids TEXT NOT NULL DEFAULT '[]'` (JSON array) to `item`. The
  row↔`Item` codec updated. Upgrade-path test: populate a DB at the prior
  version, migrate, assert data intact + column defaulted.
- Core type + GRDB schema + codec evolve together.

## iOS UI (SwiftUI) — per the handoff mock

- **"Linked exercises" section** on the piece's `LibraryDetailScreen` (Pieces
  only) — a `.cardSurface()` block after notes/tags, before Delete. Header:
  "Linked exercises" + a **count badge** + an **Edit / Done** toggle. Rows
  divided by `HairlineDivider`. A "Link an exercise" footer button.
  - **Row:** gold type bar + title (serif) + key/tempo meta + trailing **44pt
    score ring** (from #1009; unrated → empty ring + en-dash). Tap row → exercise
    detail. Swipe to unlink (removes the link, never the exercise).
  - **Edit mode:** ring **hidden**; row shows a leading red remove (minus-circle)
    + trailing drag grip; `onMove` reorder.
  - **Empty state:** link icon + serif "No exercises linked yet" + body + an
    outlined "Link an exercise" button.
- **Add exercises sheet — Picker A** (model presentation on `TagFilterSheet`):
  search field, a "Create new exercise" row (→ `LibraryAddScreen`), then "Your
  exercises · already-linked hidden" with **checkmark multi-select** rows, and a
  **sticky "Add N exercises"** button (disabled with "Select exercises to add"
  when none).
- **Library rows** (`LibraryItemCard`) show the ring for pieces *and* exercises
  (the #1009 swap).
- **Exercise detail:** a **"Linked from"** `.cardSurface()` card listing the
  piece(s) that link this exercise, each tappable → the piece. (Hero ring +
  "Practise this" are #1009 / existing.)
- Tokens only; no literals. Ships with snapshot tests (empty, populated, edit,
  picker), VoiceOver labels, Dynamic Type, iPad `SplitView`.

## Testing

- Core: link / unlink / reorder handlers (`local_first`); validation edges
  (non-exercise target, non-piece host, duplicate, self-link, missing id); view
  resolution filters tombstoned/missing and attaches the score rollup;
  `linked_from_pieces` computation; real bincode-bridge round-trip
  (`assert_round_trips`) for the extended `Item`.
- iOS: GRDB migration upgrade-path test; snapshot tests for the section + picker.

## Resolved questions

- Add picker = **Picker A** (checkmark + sticky Add N).
- Row density = **title + meta + ring** (no micro-stats).
- Reverse **"Linked from"** view = **in scope**.
- Library row shows the ring for pieces too.

## Deferred / out of scope (tracked)

- **Web/API (Turso) parity** — API migration for `linked_exercise_ids`, routes,
  the online write branch + "test both modes" (invariant 6). When web un-pauses.

## Phasing

Phase 3 of the layered sequence. Single PR: core field (`linked_exercise_ids`)
+ events + validation + view (link list + `linked_from_pieces`), the GRDB
migration + codec, the iOS section + Picker A + edit mode + exercise "Linked
from" card, tests. Depends on #1008 (Phase 1) and #1009 (Phase 2) having landed.
