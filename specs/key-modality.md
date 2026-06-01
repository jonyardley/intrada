# Structured key + modality

> Tier 3 spec (rides with its implementation PR). Issue: #829.

## Problem

Library items stored musical key as one freeform string (`Item.key:
Option<String>`, e.g. `"F# major"`). That's fine for display but opaque for
data work — you can't cleanly ask "all my F♯ pieces (either mode)" or "what
proportion of my repertoire is minor?" without fragile substring matching.

## Approach

Split the value into two fields:

- `key: Option<String>` — now the **tonic** (`"F#"`, `"Db"`, `"A"`).
- `modality: Option<Modality>` — a new `Major | Minor` enum (serialised
  `"major"`/`"minor"`).

The circle-of-fifths **selection logic stays in the Swift `KeyPicker`** (UI
concern; #819 closed won't-do). Only the *stored shape* is a core concern. The
composed display (`"F♯ major"`) is assembled in the shell (`KeyHelper.display` /
`LibraryItemView.keyDisplay`), not the core.

## Key decisions

- **iOS local-first only.** The app persists to the on-device GRDB store; the
  online/HTTP path isn't wired. So the local store gains a `modality` column;
  the **API/Turso migration is deferred** until sync lands — the API compiles
  and returns `modality: None` (documented in `db/items.rs`). No API behaviour
  change, existing API tests stay green.
- **No backfill.** `KeyHelper` still parses legacy `"F# major"` strings, so old
  rows display correctly and **self-heal** to `key + modality` the next time
  they're saved through the picker.
- **`modality` is additive + nullable**, `#[serde(default)]` — satisfies the
  offline-first invariants (additive schema, sync-ready columns unchanged).
- **Modes (Dorian etc.) are out of scope** — tracked as #830.

## Touch points

- `intrada-core`: `Modality` enum (`domain/item.rs`); `modality` on `Item` /
  `CreateItem` / `UpdateItem` (three-state) / `LibraryItemView`; threaded
  through the item handler + HTTP DTOs. New tests: add-carries-modality,
  three-state update.
- `intrada-api`: `Item` constructions set `modality: None` (deferred); no Turso
  column yet.
- iOS: GRDB `v2_add_modality` migration + codec (`LibraryStore`); `KeyHelper`
  uses the core `Modality` and exposes `selection(key:modality:)` /
  `nextOnTap` / `display`; `KeyPicker` binds `key` + `modality`; Add/Edit
  screens split the field; Detail/Card render `keyDisplay`.

## Verification

- `cargo test --workspace` (core modality tests + all existing), `cargo clippy
  -- -D warnings`, `cargo fmt --check`.
- iOS `xcodebuild test` on Xcode 26.5 / iOS 26.5: `KeyHelperTests`,
  `LibraryStoreTests` (modality round-trip + `modality` column), all snapshots
  (unchanged — the split is display-transparent).
- Manual: add an item, pick a key (stores tonic + modality); open a legacy
  `"F# major"` item — displays correctly and normalises to `F#` + Major on save.

## Out of scope (tracked)

- Turso/API persistence of modality (until sync) · modality filter UI (#820) ·
  church modes (#830).
