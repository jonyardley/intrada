# Tasks: JSON File Persistence

**Input**: Design documents from `/specs/011-json-persistence/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

**Tests**: Included — existing CLI tests must be adapted and new persistence tests added.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Exact file paths included in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add shared `LibraryData` type and update workspace dependencies

- [x] T001 Add `LibraryData` struct with `#[serde(default)]` fields in `crates/intrada-core/src/domain/types.rs`
- [x] T002 Re-export `LibraryData` from `crates/intrada-core/src/lib.rs`
- [x] T003 Verify `cargo test -p intrada-core` passes (no core changes beyond the new struct)

---

## Phase 2: User Story 1 — CLI persists library data as JSON (Priority: P1)

**Goal**: Replace SQLite with JSON file persistence. All existing CLI commands work identically — only the storage backend changes.

**Independent Test**: `cargo test -p intrada-cli` passes. Run `intrada add piece`, exit, run `intrada list` — added piece appears. Inspect `~/.local/share/intrada/library.json` for valid JSON.

### Implementation for User Story 1

- [x] T004 [US1] Rewrite `crates/intrada-cli/src/storage.rs`: replace `SqliteStore` with `JsonStore` implementing `new()`, `new_with_path()` (for tests), `load_all()`, `save_piece()`, `save_exercise()`, `update_piece()`, `update_exercise()`, `delete_item()`, and private `write_library()` using atomic writes (temp file + rename) with `serde_json::to_string_pretty()`
- [x] T005 [US1] Update `crates/intrada-cli/src/shell.rs`: change `SqliteStore` → `JsonStore` in struct definition, constructor, and imports
- [x] T006 [US1] Update `crates/intrada-cli/src/main.rs`: change store construction from `SqliteStore::new()` to `JsonStore::new()`
- [x] T007 [US1] Remove `rusqlite` from `crates/intrada-cli/Cargo.toml`
- [x] T008 [US1] Remove `rusqlite` from workspace `Cargo.toml` `[workspace.dependencies]`
- [x] T009 [US1] Adapt all existing tests in `crates/intrada-cli/src/storage.rs`: replace `SqliteStore::new_in_memory()` with `JsonStore::new_with_path(tempdir)` using `tempfile` or `std::env::temp_dir()` for test isolation. Cover: save/load piece, save/load exercise, update piece, update exercise, delete item, empty store, piece with no optional fields
- [x] T010 [US1] Adapt all existing tests in `crates/intrada-cli/src/shell.rs`: update `test_shell()` helper to use `JsonStore::new_with_path(tempdir)`. Ensure load_data, add_piece_round_trip, add_piece_persists, unicode round-trips, boundary tests, and validation tests all pass
- [x] T011 [US1] Add new test in `crates/intrada-cli/src/storage.rs`: missing file returns empty library (no error)
- [x] T012 [US1] Add new test in `crates/intrada-cli/src/storage.rs`: malformed JSON file returns an error
- [x] T013 [US1] Add new test in `crates/intrada-cli/src/storage.rs`: JSON with unknown fields deserialises successfully (schema forward-compatibility)
- [x] T014 [US1] Run `cargo test -p intrada-cli` — all tests pass
- [x] T015 [US1] Run `cargo clippy -- -D warnings` — zero warnings across workspace

**Checkpoint**: CLI fully functional with JSON persistence. All existing tests pass. `rusqlite` gone from dependency tree.

---

## Phase 3: User Story 2 — Web shell persists library data to localStorage (Priority: P2)

**Goal**: Add localStorage persistence to the web shell. Data survives page reloads. Stub data used only on first run.

**Independent Test**: Open web app, add a piece, refresh — piece persists. Check `localStorage.getItem("intrada:library")` in browser console.

### Implementation for User Story 2

- [x] T016 [P] [US2] Add `web-sys` dependency with `Storage` and `Window` features, and `serde_json` (workspace) to `crates/intrada-web/Cargo.toml`
- [x] T017 [US2] Update `crates/intrada-web/src/core_bridge.rs`: maintain an in-memory `LibraryData` (loaded once on init). On `StorageEffect::LoadAll`, read `intrada:library` from localStorage, deserialise as `LibraryData`, fall back to stub data if key absent or JSON corrupt, store in the in-memory copy. On `SavePiece`/`SaveExercise`/`UpdatePiece`/`UpdateExercise`/`DeleteItem`, mutate the in-memory `LibraryData` (push/replace/remove) and write back to localStorage as compact JSON via `serde_json::to_string()`. Log warning to console if `setItem` fails (localStorage full).
- [x] T018 [US2] Update `crates/intrada-web/src/app.rs`: remove direct `create_stub_data()` call from `App()` init — instead, send a `LoadAll`-triggering event or rely on the existing `StorageEffect::LoadAll` flow in `core_bridge.rs` to handle first-load logic (stub data vs persisted data)
- [x] T019 [US2] Run `trunk build` in `crates/intrada-web/` — compiles to WASM without errors
- [x] T020 [US2] Run `cargo clippy -- -D warnings` — zero warnings across workspace

**Checkpoint**: Web shell persists to localStorage. Stub data appears on first load, persisted data on subsequent loads. All commands (add/edit/delete) survive page refresh.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Final verification across the full workspace

- [x] T021 Run full `cargo test` — all tests pass across all 3 crates
- [x] T022 Verify `rusqlite` absent from `Cargo.lock`: `cargo tree -p intrada-cli | grep -i sqlite` returns nothing
- [x] T023 Run quickstart.md manual verification steps for CLI (add piece, inspect JSON, list, add exercise, delete)
- [ ] T024 Run quickstart.md manual verification steps for web (trunk serve, add piece, refresh, check localStorage)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **US1 (Phase 2)**: Depends on Phase 1 (needs `LibraryData` type)
- **US2 (Phase 3)**: Depends on Phase 1 (needs `LibraryData` type). Independent of US1 (different crate).
- **Polish (Phase 4)**: Depends on Phases 2 and 3 both complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Phase 1 only. No dependency on US2.
- **User Story 2 (P2)**: Depends on Phase 1 only. No dependency on US1. Can be developed in parallel with US1.

### Within Each User Story

- Storage implementation before shell/main updates
- Shell updates before test adaptation
- All tests passing before checkpoint

### Parallel Opportunities

- **T016** (web Cargo.toml) can run in parallel with any US1 task (different crate)
- **US1 and US2** can be developed in parallel after Phase 1 completes (different crates, no shared files)
- **T011, T012, T013** (new storage tests) can run in parallel with each other

---

## Parallel Example: User Story 1 + User Story 2

```text
# After Phase 1 (Setup) completes:

# Stream A — US1 (CLI):
T004 → T005 + T006 + T007 + T008 (parallel, different files) → T009 + T010 (parallel) → T011 + T012 + T013 (parallel) → T014 → T015

# Stream B — US2 (Web):
T016 → T017 → T018 → T019 → T020
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: User Story 1 (T004–T015)
3. **STOP and VALIDATE**: `cargo test -p intrada-cli` passes, `rusqlite` gone
4. CLI is fully functional with JSON persistence

### Incremental Delivery

1. Phase 1 → Setup complete
2. Phase 2 → CLI JSON persistence working (MVP)
3. Phase 3 → Web localStorage persistence working
4. Phase 4 → Full verification, ready for PR

---

## Notes

- [P] tasks = different files, no dependencies
- [US1]/[US2] labels map tasks to specific user stories
- US1 and US2 are fully independent (different crates) — can be parallelised
- Commit after each phase checkpoint
- `data.rs` (stub data) is unchanged — still used for first-run seeding in web shell
