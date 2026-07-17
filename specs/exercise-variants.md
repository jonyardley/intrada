# Exercise steps (variants)

> Tier 3 lightweight spec, riding with the C1 implementation branch
> ([#1083], epic [#1087]). Status: **C1 (core + data) implemented on this
> branch**; C2 (Steps UI + reflection picker), C3 (12-keys preset, closes #46)
> and C4 (step management polish) follow as their own PRs. **Scope:
> `intrada-core` + native iOS only** — web/API (Turso) out of scope until sync
> (see Offline-first below). Users see **"Steps"** (or "Keys"); "variant" is
> the core's name and never appears on screen.

[#1083]: https://github.com/jonyardley/intrada/issues/1083
[#1087]: https://github.com/jonyardley/intrada/issues/1087

## Problem

"Land on the 3rd, then add a note each round" is a ladder. So is "all twelve
keys", and so is "shells in each inversion". Today an exercise has one flat
score, so progress along any of these reads as noise: score 8 in C says
nothing about F♯. Multi-key practice (#46) and the chart-to-scaffold twelve-key
ladder ([#1107], a blocked consumer) both need the same thing: an exercise
that owns an **ordered list of variants, each scored independently**, where
progress means advancing the step — not polishing one rating.

## Approach

One generic mechanism, no per-preset code. An exercise owns
`Vec<Variant>`; a variant is `{ id, label, position, updated_at, deleted_at }`.
Session entries gain an optional `variant_id` recording *which step* a score
belongs to. Everything else is derived: per-step score history, whether a step
is **solid**, and the **current step** (the first that isn't yet solid).
Presets (twelve keys, inversions) are just label lists — C3 ships the first.

```text
Item (exercise)
  └─ variants: Vec<Variant { id, label, position, updated_at, deleted_at }>
PracticeSession
  └─ entries[]: SetlistEntry { …, variant_id: Option<String> }   // serde(default)
ViewModel
  └─ LibraryItemView.variants: Vec<VariantView { …, latest_score,
       score_history, is_solid, is_current }>
  └─ SetlistEntryView.variant_id
```

## Key decisions

1. **Variants embed in `Item` on the wire, but persist to a `variant` child
   table.** `Item.variants: Vec<Variant>` (`#[serde(default)]`, trailing)
   keeps the model/bridge shape simple and makes every save atomic with the
   item row; the GRDB store fans the list out to per-row storage. The child
   table (the store's first) exists for **per-row LWW**: each step carries its
   own `updated_at` + `deleted_at`, so the future sync engine can merge a
   rename against a score without whole-item conflicts (invariant 2).
2. **No hard deletes; tombstones stay in the model.** Removing a step sets
   `deleted_at`; the row stays in the table *and* in `Item.variants` (views
   filter it). Kept in-model so (a) the core owns reconciliation — the shell
   never diffs child rows (invariant 4), and (b) history keeps resolving: a
   session entry pointing at a removed step still finds its label.
3. **`SetVariants { id, labels }` reconciles by label.** One event defines the
   whole ladder: labels matching an existing variant (case-insensitive) keep
   its id — and its score history; removed labels tombstone; a re-added label
   **resurrects** the tombstoned variant, history intact. Rename is
   deliberately *not* expressible here (indistinguishable from remove+add);
   C4 adds an id-based rename event.
4. **"Solid" is `latest step score >= 8` (of 10)** — `SOLID_SCORE_MIN` in
   `domain/variant.rs`. A named constant, not a per-user setting, until real
   use argues otherwise. "Current step" = first live step by position that
   isn't solid; a fully-solid ladder has no current step (it's done).
   Derivation lives in `build_view`, computed per render like
   `exercise_contexts` — never stored.
5. **`variant_id` joins the session-entry codec, not the schema.** Entries are
   a JSON blob in the session row, so old rows decode the missing key to
   `None` — no migration. The core writes `variant_id` only via
   `UpdateEntryVariant` (summary/active reflection, mirroring
   `UpdateEntryScore`); scores stay on the entry, the variant says which rung
   they belong to. No snapshot label in the entry: labels resolve live via
   decision 2 (revisit only if exercise deletion in history proves to matter).
6. **Local-first only until sync** (epic decision, invariant 6 consciously
   scoped). Online mode: `SetVariants` / `UpdateEntryVariant` surface a clear
   error, mutate nothing, emit no HTTP — pinned by tests, so the scope-out is
   explicit rather than a silent no-op (#846 class). The API keeps compiling
   (`variants: vec![]` at its construction sites, the `modality` precedent);
   server tables arrive with the sync engine.
7. **Step scope capped: label + position + score history.** No per-step tempo,
   notes or targets in v1 — every extra field is a migration on the tier where
   the device is the only copy of the data.

## Validation

- Host item must exist and be an **Exercise** (pieces don't get ladders).
- Labels: trimmed; each 1–`MAX_VARIANT_LABEL` (100) chars; at most
  `MAX_VARIANTS` (24) per ladder; case-insensitive duplicates rejected.
  Empty list = clear the ladder (tombstones every step).
- `UpdateEntryVariant`: entry must exist (Active/Summary) and be Completed;
  `variant_id` must be a **live** variant of that entry's item; `None` clears.

## Persistence (GRDB, migration `v9_variant`)

```sql
CREATE TABLE variant (
  id TEXT PRIMARY KEY NOT NULL,
  exercise_id TEXT NOT NULL,
  label TEXT NOT NULL,
  position INTEGER NOT NULL,
  updated_at TEXT NOT NULL,   -- RFC3339, written by the core (same as item)
  deleted_at TEXT             -- soft-delete tombstone
);
CREATE INDEX idx_variant_exercise_id ON variant(exercise_id);
```

- `save(item)` / `save(items)` upsert every `item.variants` row inside the
  existing transaction (ON CONFLICT(id) DO UPDATE). No delete-missing: the
  core always carries the tombstones it loaded, so absent-row diffing never
  happens in Swift.
- `loadItems()` attaches **all** variant rows (tombstones included) per
  decision 2, ordered by position.
- Deleting an exercise tombstones the item row only; its variant rows stay,
  unreferenced — harmless, and sync-safe.

## Testing

- **Core (TDD):** ladder create / reorder-preserves-ids / tombstone /
  resurrect; validation rejections; empty-list clear; per-step derivation
  (latest, history, solid, current); entry variant set/clear + rejections;
  saved sessions carry `variant_id`; online-mode graceful degradation for
  both events (no mutation, no HTTP, error surfaced).
- **Bridge (#846):** `assert_round_trips` for `Variant`-in-`Item`-in-
  `SaveItem`/`SaveItems`, `SetVariants`, `UpdateEntryVariant`,
  `SetlistEntry.variant_id`-in-`SaveSession`, `VariantView`; plus a real
  `LiveBridge` Swift test driving set-ladder → view → entry-variant set/clear
  through actual bincode.
- **iOS:** `v9` upgrade-path test from a populated v8 DB (rows intact, empty
  ladders); variant save/load round-trip incl. tombstone + ordering; old
  entries blob (no `variantId` key) decodes to `nil`; batch save atomicity.

## Open questions (deferred, tracked on #1083)

- **C2:** reflection picker defaulting to the current step; Steps section
  reconciled with the per-piece detail surfaces before build.
- **C3:** 12-keys preset ordering (fifths vs chromatic) and key-grid UI;
  closes #46. #1107's twelve-key scaffold ladder consumes the same preset.
- **C4:** rename (id-based), drag reorder, archive UI on top of the
  already-shipped tombstones.
- Whether `SOLID_SCORE_MIN` should ever be user-tunable, and whether "solid"
  should decay with time (research-foundation.md's consolidation model) —
  out of scope until the scheduler epic.
