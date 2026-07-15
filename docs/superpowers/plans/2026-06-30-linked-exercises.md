# Linked exercises (Phase 3) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** Let a piece carry an ordered group of linked exercises (each shown with its 0–10 `ScoreRing`), with a multi-select picker to add them and a "Linked from" reverse card on the exercise. Full design: `specs/piece-linked-exercises.md`; mock: `design/linked-exercises-mock.dc.html`.

**Architecture:** `linked_exercise_ids: Vec<String>` on `Item` (structurally identical to the existing `tags: Vec<String>` — mirror its lifecycle end-to-end). Dedicated `LinkExercise`/`UnlinkExercise`/`ReorderLinkedExercises` events (NOT via `CreateItem`/`UpdateItem`). The core resolves ids → live exercise views (reusing the `practice` rollup) and computes the reverse `linked_from_pieces`. GRDB stores a JSON column. Builds on merged Phase 1 (0–10 scoring) + Phase 2 (`ScoreRing`).

**Tech Stack:** Rust (crux_core, serde/bincode FFI), SwiftUI, GRDB.

## Global Constraints

- **Offline-first:** writes go through `save_or_put` (the `local_first` GRDB branch — invariant 1); the piece is the synced entity, its `updated_at` bumped on each link change (invariant 2); client-owned ids (3); reconciliation in core (4); a failed local write surfaces, never a silent success (5).
- **Bincode FFI:** new `Item` field uses `#[serde(default)]`; **append** new `ItemEvent` variants (never reorder); bridge-crossing types (the new events) get an `assert_round_trips` test (#846).
- **`linked_exercise_ids` is managed ONLY by the link/unlink/reorder events** — it is NOT added to `CreateItem`/`UpdateItem`.
- **Generated bindings** regenerate via `just ios-gen` (gitignored — never committed/hand-edited).
- iOS: tokens only (`IntradaColor`/`IntradaFont`/`IntradaSpacing`/`IntradaRadius`), no literals; VoiceOver + Dynamic Type; reuse `ScoreRing`, `.cardSurface()`, `HairlineDivider`, `NavigationLink(value:)` (the existing `navigationDestination(for: String.self)` already routes any id → `LibraryDetailScreen`).
- **api compile-compat:** the new `Item` field breaks `intrada-api`'s `Item {…}` constructors — add `linked_exercise_ids: vec![]` there (api parity is deferred; CI builds api). `intrada-web` is CI-excluded/paused — leave it.
- Per-task green gate: core → `cargo test -p intrada-core` + clippy + fmt; iOS → `just ios-test` on the worktree-scoped sim (shared-sim rule). Commit messages end `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`; scoped `git add` only.

## File Structure
- Core: `crates/intrada-core/src/domain/item.rs` (field + events + handlers), `src/validation.rs` (validate fn), `src/model.rs` (`LinkedExerciseView`, `PieceRefView`, `LibraryItemView` fields), `src/app.rs` (view resolution), `src/domain/types.rs` (round-trip test).
- api compat: `crates/intrada-api/src/db/items.rs`.
- Persistence: `ios/Intrada/Core/LibraryStore.swift` (v6 migration + codec).
- iOS UI: `ios/Intrada/Views/Screens/LibraryDetailScreen.swift` (+ section + linked-from card), new `ios/Intrada/Views/Components/LinkedExercisePickerSheet.swift`, `ios/IntradaTests/ScreenSnapshotTests.swift`.

---

### Task 1: Core — `linked_exercise_ids` field + link events + handlers + validation

**Files:** `crates/intrada-core/src/domain/item.rs`, `src/validation.rs`; compile-compat any forced `Item {…}` constructor across the workspace (core + `intrada-api`).

**Interfaces produced:** `Item.linked_exercise_ids: Vec<String>`; `ItemEvent::LinkExercise { piece_id, exercise_id }`, `ItemEvent::UnlinkExercise { piece_id, exercise_id }`, `ItemEvent::ReorderLinkedExercises { piece_id, ordered_ids }`.

- [ ] **Step 1: Failing tests** (in `item.rs` tests) — drive each event and assert the model + validation. Cover: link adds the id to the piece; link rejects a non-existent exercise / a non-Exercise target / a non-Piece host / a duplicate / a self-link (leaving the list unchanged + `last_error` set); unlink removes it; reorder sets the order; and that each bumps `updated_at`. Use the test fixtures already in `item.rs`.

- [ ] **Step 2: Add the field** to `Item` (after `tags`): `#[serde(default)] pub linked_exercise_ids: Vec<String>,`. Fix every `Item {…}` literal the compiler flags (core fixtures + `intrada-api/src/db/items.rs`) with `linked_exercise_ids: vec![]`. Do NOT add it to `CreateItem`/`UpdateItem`.

- [ ] **Step 3: Append the three event variants** to `ItemEvent` (after `RemoveTags`).

- [ ] **Step 4: Add `validate_link_exercise(piece_id, exercise_id, model)`** in `validation.rs`: piece exists + is `Piece`; exercise exists + is `Exercise`; not already linked; `piece_id != exercise_id`. Return `LibraryError::Validation`/`NotFound`.

- [ ] **Step 5: Handlers** in the `ItemEvent` match — mirror `AddTags`: validate (set `last_error` + `render()` on failure, leaving the list unchanged), else mutate (`LinkExercise` push if absent; `UnlinkExercise` retain≠id; `ReorderLinkedExercises` set to `ordered_ids` filtered to the currently-linked set so a stale id can't inject), bump `updated_at`, `model.last_error = None`, return `save_or_put(model, piece.clone())`.

- [ ] **Step 6: Green** — `cargo test -p intrada-core` (new tests pass), `cargo clippy -p intrada-core -- -D warnings`, and `cargo test -p intrada-api` (compat compiles). Commit: `feat(core): link exercises to a piece (field + events + validation)`.

---

### Task 2: Core — resolved views (`linked_exercises` + `linked_from_pieces`)

**Files:** `crates/intrada-core/src/model.rs` (new view structs + `LibraryItemView` fields), `src/app.rs` (the `view()` builder ~345–373).

**Interfaces produced:** `LinkedExerciseView { id, title, key, tempo, practice: Option<ItemPracticeSummary> }`; `PieceRefView { id, title }`; `LibraryItemView.linked_exercises: Vec<LinkedExerciseView>`; `LibraryItemView.linked_from_pieces: Vec<PieceRefView>` (consumed by Swift in Tasks 6/8).

- [ ] **Step 1: Failing test** — build a model with a piece linking two exercises (one tombstoned/missing) and assert the piece's `LibraryItemView.linked_exercises` resolves the live one in order, drops the missing one; and the exercise's `linked_from_pieces` lists the piece. Use the crate's `view(&model)` / `app.view()` pattern.

- [ ] **Step 2: Add the view structs** to `model.rs` (mirror the field/derive style of neighbouring views) and the two `Vec` fields on `LibraryItemView`.

- [ ] **Step 3: Resolve in `app.rs view()`** — for a `Piece`, map `item.linked_exercise_ids` → find live `Exercise` (skip missing/non-exercise), attaching `model.practice_summaries.get(&ex.id)`; preserve order. For an `Exercise`, scan `model.items` for pieces whose `linked_exercise_ids.contains(&item.id)` → `PieceRefView`. Non-matching kinds get `vec![]`.

- [ ] **Step 4: Green** — `cargo test -p intrada-core` + clippy. Commit: `feat(core): resolve linked_exercises + linked_from_pieces on the view`.

---

### Task 3: Core — FFI round-trip coverage

**Files:** `crates/intrada-core/src/domain/types.rs`.

- [ ] **Step 1: Add `item_link_events_round_trip_on_ffi_bincode_wire`** (mirror `item_tag_events_round_trip…`) asserting `assert_round_trips` for all three new `ItemEvent` variants (with populated ids/ordered_ids). If an existing `Item` round-trip test exists, ensure it covers a populated `linked_exercise_ids`.
- [ ] **Step 2: Green** — `cargo test -p intrada-core`. Commit: `test(core): round-trip link events on the FFI bincode wire`.

---

### Task 4: Regenerate Swift bindings

- [ ] **Step 1:** `just ios-gen`; confirm `SharedTypes.swift` has `case linkExercise(pieceId:exerciseId:)` etc., `linkedExerciseIds` on `Item`, and `linkedExercises`/`linkedFromPieces` on `LibraryItemView` + the `LinkedExerciseView`/`PieceRefView` types. `ios/generated` is gitignored — nothing to commit. (No commit; this is a build artifact.)

---

### Task 5: iOS GRDB — persist `linked_exercise_ids`

**Files:** `ios/Intrada/Core/LibraryStore.swift`; test `ios/IntradaTests/LibraryStoreMigrationTests.swift`.

- [ ] **Step 1: Failing upgrade-path test** — populate an `item`-table DB migrated to `v5_rescale_entry_scores`, insert an item row (no `linked_exercise_ids` column yet), run the full migrator, assert the column exists defaulting to `'[]'` and a save/load round-trips a populated `linkedExerciseIds` (e.g. `["e1","e2"]`).
- [ ] **Step 2: Append migration** `v6_item_linked_exercises`: `ALTER TABLE item ADD COLUMN linked_exercise_ids TEXT NOT NULL DEFAULT '[]'`.
- [ ] **Step 3: Codec** — add `encodeLinkedExerciseIds`/`decodeLinkedExerciseIds` (mirror `encodeTags`/`decodeTags`); decode in `item(from:)`; add the column + arg to `save()`'s INSERT + `ON CONFLICT` set (mirror `tags`).
- [ ] **Step 4: Green** — `just ios-test`. Commit: `feat(ios): persist linked_exercise_ids (GRDB v6)`.

---

### Task 6: iOS — "Linked exercises" section on the piece detail

**Files:** `ios/Intrada/Views/Screens/LibraryDetailScreen.swift`; `ios/IntradaTests/ScreenSnapshotTests.swift`.

- [ ] **Step 1: Add the section** (Pieces only, after tags, before delete): a `.cardSurface()` block with a header ("Linked exercises" + a count + an **Edit/Done** toggle), the linked-exercise rows divided by `HairlineDivider`, and a "Link an exercise" footer button (opens the picker — Task 7 wires it; for this task a `@State showingPicker` + a placeholder `.sheet` stub is fine, finalized in Task 7).
- [ ] **Step 2: Tracked-exercise row** — `NavigationLink(value: exercise.id)` wrapping: title (`cardTitle`) + key/tempo meta + trailing `ScoreRing(score: exercise.practice?.latestScore.map(Int.init), size: 44)`. (Routing is already handled by the parent `navigationDestination(for: String.self)`.)
- [ ] **Step 3: Edit mode** — when editing, rows show a leading remove (red minus) that dispatches `.item(.unlinkExercise(pieceId: item.id, exerciseId: ex.id))` and a drag affordance dispatching `.item(.reorderLinkedExercises(pieceId:orderedIds:))` on `onMove`; the `ScoreRing` is hidden while editing (per the mock). Re-read `viewModel.error` after each `send` — surface failures, never a silent success.
- [ ] **Step 4: Empty state** — an in-card prompt + the "Link an exercise" button when the piece has no links.
- [ ] **Step 5: Snapshots** — add `testPieceDetailLinkedEmpty`, `testPieceDetailLinkedPopulated`, `testPieceDetailLinkedEditing` using preview fixtures (a piece with 0 / N linked exercises). Record + `just ios-snapshots-optimize`/`-check`.
- [ ] **Step 6: Green** — `just ios-test`. Commit: `feat(ios): Linked exercises section on the piece detail`.

---

### Task 7: iOS — multi-select "Add exercises" picker (Picker A)

**Files:** create `ios/Intrada/Views/Components/LinkedExercisePickerSheet.swift`; wire it in `LibraryDetailScreen.swift`.

- [ ] **Step 1: Build `LinkedExercisePickerSheet`** (model on `TagFilterSheet`): a searchable list of exercises (`viewModel.items` filtered to `kind == .exercise` AND not already linked to this piece), checkmark multi-select, a "Create new exercise" row → `NavigationLink` → `LibraryAddScreen(defaultKind: .exercise)`, and a sticky "Add N" confirm (disabled when none) that returns the selected ids.
- [ ] **Step 2: Wire it** — the section's "Link an exercise" button presents the sheet; on confirm, dispatch one `.item(.linkExercise(pieceId: item.id, exerciseId: id))` per selected id; re-read `viewModel.error`.
- [ ] **Step 3: Snapshot** — `testLinkedExercisePicker` (a few selectable rows, some selected). Record + optimize/check.
- [ ] **Step 4: Green** — `just ios-test`. Commit: `feat(ios): multi-select Add-exercises picker (links to the piece)`.

---

### Task 8: iOS — "Linked from" card on the exercise detail

**Files:** `ios/Intrada/Views/Screens/LibraryDetailScreen.swift`; `ScreenSnapshotTests.swift`.

- [ ] **Step 1: Add a "Linked from" `.cardSurface()` card** (Exercises only, when `linked_from_pieces` non-empty): header "Linked from" + a `NavigationLink(value: piece.id)` row per piece (title + chevron) routing to the piece's detail.
- [ ] **Step 2: Snapshot** — `testExerciseDetailLinkedFrom`. Record + optimize/check.
- [ ] **Step 3: Green** — `just ios-test`. Commit: `feat(ios): Linked-from card on the exercise detail`.

---

### Task 9: Final — gates, review, PR

- [ ] **Step 1: Full gates** — `cargo fmt --check`, `cargo clippy -- -D warnings` (or `-p intrada-core -p intrada-api`), `cargo test -p intrada-core` + `cargo test -p intrada-api` (FULL output, no `head` truncation — confirm 0 failed everywhere), and `just ios-test` green.
- [ ] **Step 2: Whole-branch review** (capable model) over the full diff — focus on offline-first invariants, the reverse-view computation, silent-success in the edit-mode dispatches, and end-to-end coherence. Fix Critical/Important.
- [ ] **Step 3: PR** — push the branch, open the PR (per CLAUDE.md, via the gates funnel). Body: scope (core + iOS; api compat; web paused), the offline-first checklist, Coverage line, and that it implements `specs/piece-linked-exercises.md`. Post the review as a PR comment ending with `Deferred items tracked: …`.

## Self-Review (controller)
- Each `Vec<String>` link op mirrors `tags` end-to-end; the piece is the synced entity; no silent-success in the edit dispatches.
- Reverse `linked_from_pieces` is a live scan (no schema); tombstoned/missing exercises drop from `linked_exercises`.
- api compiles (compat `vec![]`); web left (CI-excluded).
