# Exercise steps (variants) — one ladder mechanism

> Tier 3. Issue #1083 (epic #1087, workstream C). This spec rides with the C1
> implementation branch. Last reviewed: 2026-07-17.

## Problem

Real practice progresses along ladders, not single ratings. "Land on the 3rd,
then add a note each round", "all twelve keys", "shells in each inversion" — all
the same shape: an exercise owns an ordered list of things you tackle in turn,
each scored on its own, and progress means *advancing the rung*, not just
polishing one number.

Today an exercise (`Item` with `kind == Exercise`) has one derived score per
piece (`latest_score`, #1087 B). There is no rung. Multi-key practice (#46) was
about to be built as bespoke key sub-items; this replaces that with one generic
mechanism whose first preset *is* the twelve keys (C3, closes #46).

Users see **"Steps"** (or **"Keys"** for the key preset). The core's name is
**"variants"**, never on screen. The per-step state a user reads is
**"Solid"**, never "consolidated".

## Approach

An exercise owns an ordered `Vec<Variant>`. Each variant is scored independently
by tagging the session entry that practised it with a `variant_id`. A variant's
score history is therefore **derived** from sessions, not stored on the variant.
The **current step** is derived too: the first step that isn't yet solid.

```
Item (kind: Exercise)
  └── variants: [Variant { id, label, position, updated_at, deleted_at }]   (ordered)

PracticeSession
  └── entries: [SetlistEntry { …, variant_id: Option<String> }]             (which rung this rep hit)

derived:  step.latest_score  = latest SetlistEntry.score where variant_id == step.id
          step.solid         = latest_score >= SOLID_THRESHOLD
          current_step        = first step by position that is not solid  (None if all solid)
```

### Storage split — child table at GRDB, `Vec<Variant>` at the core

This is the one deliberate deviation from house style, so it is called out.

- **Persistence (GRDB, `LibraryStore`):** a real normalized child table
  `variant(id PK, item_id FK, label, position, updated_at, deleted_at)`. This is
  the **first** normalized child table in the app — every prior one-to-many
  (`tags`, `linked_exercise_ids`, session `entries`, `chord_chart`) is a JSON
  column on the parent row.
- **Why deviate:** invariant 2 requires **per-row** `updated_at`/`deleted_at` so
  the future sync engine can LWW-merge and tombstone *individual* variants (device
  A renames step 2 while device B archives step 5). A JSON blob on the parent is
  a single LWW unit — one device's variant edit would clobber the other's. The
  documented reason the app avoids child tables (bincode positional decode breaks
  old rows) doesn't apply: SQL columns aren't positional.
- **Core (FFI):** the core still sees `Item.variants: Vec<Variant>`, each variant
  carrying its own `updated_at`. `LibraryStore.item(from:)` does a second query to
  hydrate variants; `upsert` decomposes them back to child rows. So reconciliation
  stays in the core (invariant 4) — the shell only executes storage ops — and the
  child table is purely a persistence detail the core never sees.

### Core types (new)

```rust
// domain/item.rs
pub struct Variant {
    pub id: String,          // client-minted ulid (invariant 3)
    pub label: String,
    pub position: usize,
    pub updated_at: DateTime<Utc>,          // core-set; shell persists verbatim
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,  // schema-level tombstone; archive UX is C4
}

// Item gains, appended last (bincode positional; #[serde(default)] for old rows):
    #[serde(default)]
    pub variants: Vec<Variant>,
```

```rust
// domain/session.rs — SetlistEntry gains, appended LAST after group_id:
    #[serde(default)]
    pub variant_id: Option<String>,
```

### Events (new, C1)

- `ItemEvent::AddVariant { item_id, label }` — mints a ulid, `position` =
  max+1, `updated_at = now`, pushes to `Item.variants`, `SaveItem`.
- `SessionEvent::SetEntryVariant { entry_id, variant_id: Option<String> }` —
  sets/clears the rung on a setlist entry. Mirrors `SetEntryIntention`.

Rename / reorder / archive → **C4**. The 12-key preset (bulk `AddVariant` with
generated labels) → **C3**. `create_entry` and the `LoadSetIntoSetlist` literal
default `variant_id: None`.

### Derivation (new, C1)

Computed in the core (like `ItemPracticeSummary`), exposed minimally on the view
so C2 can render rings without more core work: each step gets `latest_score` +
`solid`, and the item gets `current_variant_id`. No stored solid flag.

## Key decisions

- **(2026-07-15, from #1083)** Local-first-only until sync. The online web app
  gets no variant support; the server silently drops `variant_id` (exactly like
  `group_id` today — `intrada-api` `row_to_entry` hardcodes it away). C1 ships a
  test that an online-mode session flow with `variant_id` set still works, so
  invariant 6 is *consciously scoped*, not silently broken. Server `variant`
  tables arrive with the sync engine, not now.
- **(2026-07-15)** Step scope capped: label + position + derived score history.
  No per-step tempo, notes or targets in v1 — every extra field is a migration on
  the tier where migrations are dangerous (the device is the only copy).
- **(2026-07-17)** Storage split (above): child table at GRDB, `Vec<Variant>` at
  the core.
- **(2026-07-17, OPEN — see below)** "Solid" rule.

## Open question — the "solid" rule

`current_step` = first non-solid step, so "solid" defines the whole ladder's
progress. Scores are 1–10 (0 = unrated). No existing band. Proposed v1:

- **A step is solid when its latest score ≥ 8** (top ~fifth of the scale), where
  "latest" = the score on the most recent session entry tagged with that
  `variant_id`. A step with no scores yet is not solid.
- **Ladder complete** (every step solid) → `current_variant_id = None`.

Simplest rule that ships; refine later if it feels too eager/too strict. The
threshold is a named const (`SOLID_THRESHOLD`) so it's a one-line change.

## C1 checklist (this branch)

- [ ] `Variant` type + `Item.variants` (serde-defaulted, appended last)
- [ ] `SetlistEntry.variant_id` (serde-defaulted, appended **last**) + defaulted
      in `create_entry` and `LoadSetIntoSetlist`
- [ ] `ItemEvent::AddVariant`, `SessionEvent::SetEntryVariant` handlers
- [ ] Derivation: per-step `latest_score`/`solid` + `current_variant_id` on view
- [ ] GRDB `v9_variant` child-table migration (`updated_at` + `deleted_at`) +
      `item(from:)`/`upsert` hydrate/decompose + `StoredStep`-style codec
- [ ] Upgrade-path test: DB populated at v8 migrates to v9 with items intact and
      an empty variant set
- [ ] `assert_round_trips` for `Variant`, `Item` (with variants),
      `ItemEvent::AddVariant`, `SetlistEntry` (with `variant_id`),
      `SessionEvent::SetEntryVariant`
- [ ] Real-bridge (`LiveBridge`) round-trips: add variant → view; tag entry → view
- [ ] Online-mode graceful-degradation test (invariant 6 scoped)

## Later phases (context only)

- **C2** exercise-detail Steps section (per-step rings) + reflection step picker
  (defaults to current step; never slows the everyday save). Reconcile with B's
  per-piece detail surfaces before build.
- **C3** 12-keys preset + key grid; closes #46.
- **C4** step management polish (rename, reorder, archive — archive uses the
  `deleted_at` column this phase lays down).
