# Tasks: Web UI Testing & E2E Test Infrastructure

**Input**: Design documents from `/specs/013-web-testing/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add test dependencies, expose modules for integration tests, verify existing tests still pass

- [x] T001 Add `wasm-bindgen-test = "0.3"` to `[dev-dependencies]` in `crates/intrada-web/Cargo.toml`
- [x] T002 Create `crates/intrada-web/src/lib.rs` that re-exports modules needed by integration tests: `pub mod core_bridge; pub mod data; pub mod helpers; pub mod types; pub mod validation;`. Update `crates/intrada-web/src/main.rs` to use `intrada_web::` imports instead of `mod` declarations for the shared modules (keep `mod app; mod components; mod views;` private since they contain Leptos components that can't be tested). This is a structural change only — no behaviour changes (FR-015).
- [x] T003 Make `save_to_local_storage` and `save_sessions_to_local_storage` in `crates/intrada-web/src/core_bridge.rs` `pub` so WASM integration tests can exercise the write path directly. Also make `STORAGE_KEY` and `SESSIONS_KEY` constants `pub` for test assertions.
- [x] T004 Run `cargo test` and `cargo clippy -- -D warnings` to verify zero existing tests are broken and no new warnings introduced (SC-007)

---

## Phase 2: User Story 1 — Rust Unit Tests for Web Shell Logic (Priority: P1) MVP

**Goal**: Add at least 15 meaningful unit tests for helpers, validation, and core event-to-effect mapping that run with standard `cargo test` (FR-001, FR-002, FR-003, FR-004, SC-001)

**Independent Test**: Run `cargo test -p intrada-web` and verify all new tests pass

### Implementation for User Story 1

- [x] T005 [P] [US1] Add `#[cfg(test)] mod tests` with `parse_tags` tests in `crates/intrada-web/src/helpers.rs`: empty string returns empty vec, single tag, multiple tags, whitespace trimming, trailing commas, empty entries filtered
- [x] T006 [P] [US1] Add `parse_tempo` tests in `crates/intrada-web/src/helpers.rs`: both empty returns None, marking only, BPM only, both present, invalid BPM string
- [x] T007 [P] [US1] Add `parse_tempo_display` tests in `crates/intrada-web/src/helpers.rs`: None input, marking only, BPM only ("132 BPM"), full format ("Allegro (132 BPM)")
- [x] T008 [P] [US1] Add `#[cfg(test)] mod tests` with `validate_library_form` tests in `crates/intrada-web/src/validation.rs`: valid piece (no errors), valid exercise (no errors), missing title, title too long (>500 chars), missing composer for piece, composer optional for exercise, oversized composer, oversized category for exercise, oversized notes, invalid BPM (non-numeric), BPM out of range, oversized tempo marking, tag too long
- [x] T009 [P] [US1] Add `#[cfg(test)] mod tests` in `crates/intrada-web/src/core_bridge.rs` with core event-to-effect tests using `Core<Intrada>`: instantiate `Core::new()`, load data via `DataLoaded`/`SessionsLoaded` events, then test that (1) adding a piece via `Event::Piece(PieceEvent::Add(...))` produces `Effect::Storage(StorageEffect::SavePiece(...))`, (2) adding an exercise produces `SaveExercise`, (3) deleting an item produces `DeleteItem`, (4) logging a session produces `SaveSession`, (5) updating a session produces `UpdateSession`, (6) deleting a session produces `DeleteSession`. Assert `Core::view()` returns correct `ViewModel` state after each event (FR-001, FR-002).
- [x] T010 [US1] Run `cargo test -p intrada-web` and verify at least 15 tests pass (SC-001), all with clear failure messages (FR-018)

**Checkpoint**: `cargo test -p intrada-web` passes with 15+ meaningful tests covering helpers, validation, and core bridge effect mapping

---

## Phase 3: User Story 2 — WASM Integration Tests in Headless Browser (Priority: P2)

**Goal**: Add at least 3 WASM integration tests verifying localStorage round-trips for library and session data (FR-006, FR-008, SC-002)

**Independent Test**: Run `wasm-pack test --headless --chrome -- --test wasm` from `crates/intrada-web/` and verify all tests pass

### Implementation for User Story 2

- [x] T011 [US2] Create `crates/intrada-web/tests/wasm.rs` with `wasm_bindgen_test_configure!(run_in_browser)` and a `clear_local_storage()` helper function that clears all localStorage keys via `web_sys::window().unwrap().local_storage().unwrap().unwrap().clear().unwrap()` (FR-009)
- [x] T012 [P] [US2] Add WASM test: library data round-trip — clear localStorage, construct a `LibraryData` with one `Piece` and one `Exercise`, write to localStorage via `intrada_web::core_bridge::save_to_local_storage`, read back via `intrada_web::core_bridge::load_library_data`, assert returned pieces/exercises match (FR-008)
- [x] T013 [P] [US2] Add WASM test: session data round-trip — clear localStorage, construct a `SessionsData` with one `Session`, write via `intrada_web::core_bridge::save_sessions_to_local_storage`, read back via `intrada_web::core_bridge::load_sessions_data`, assert returned sessions match (FR-008)
- [x] T014 [P] [US2] Add WASM test: empty localStorage seeds stub data — clear localStorage, call `intrada_web::core_bridge::load_library_data`, verify returned pieces and exercises are non-empty (stub data seeded), verify localStorage key `intrada:library` now contains data
- [x] T015 [US2] Verify WASM tests pass locally: run `wasm-pack test --headless --chrome -- --test wasm` from `crates/intrada-web/` (FR-007)
- [x] T016 [US2] Add `wasm-test` job to `.github/workflows/ci.yml`: install `wasm-pack`, run `wasm-pack test --headless --chrome -- --test wasm` with `working-directory: crates/intrada-web` (FR-010, FR-016)

**Checkpoint**: `wasm-pack test --headless --chrome -- --test wasm` passes with 3+ tests; CI job added

---

## Phase 4: User Story 3 — E2E Testing Recommendation and Proof of Concept (Priority: P3)

**Goal**: Produce E2E tool recommendation document (already in research.md) and implement a Playwright proof-of-concept smoke test with CI/CD integration (FR-011 through FR-015, SC-003, SC-004)

**Independent Test**: Build app with `trunk build`, run `npx playwright test` from `e2e/`, and verify the smoke test passes both locally and in CI

### Implementation for User Story 3

- [x] T017 [US3] Create `e2e/package.json` with `@playwright/test` dependency and `test` script
- [x] T018 [US3] Create `e2e/playwright.config.ts` configuring: base URL to local static server, Chromium-only project, web server command that serves the built dist directory, retries and timeout settings. Use an environment variable (`DIST_DIR`) with a default of `../crates/intrada-web/dist` so CI can override the path when the artifact is downloaded to a different location.
- [x] T019 [US3] Create `e2e/tests/smoke.spec.ts` with proof-of-concept test: navigate to `/`, verify page title or heading is visible, verify library list renders with at least one item (FR-013, FR-015)
- [x] T020 [US3] Verify E2E test passes locally: run `trunk build` in `crates/intrada-web/`, then `npx playwright test` in `e2e/` (SC-004)
- [x] T021 [US3] Add artifact upload step to existing `wasm-build` job in `.github/workflows/ci.yml`: upload `crates/intrada-web/dist/` as `web-dist` artifact
- [x] T022 [US3] Add `e2e` job to `.github/workflows/ci.yml`: depends on `wasm-build`, downloads `web-dist` artifact, installs Node.js 20, runs `npm ci`, installs Playwright Chromium, sets `DIST_DIR` env var to the downloaded artifact path, runs `npx playwright test` (FR-014, FR-016)
- [x] T023 [US3] Add `.gitignore` entries for `e2e/node_modules/`, `e2e/test-results/`, `e2e/playwright-report/`

**Checkpoint**: Playwright smoke test passes locally; CI workflow has `e2e` job configured

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all test layers

- [x] T024 Run full test suite: `cargo test` (unit + web unit), `wasm-pack test --headless --chrome` (WASM), `npx playwright test` (E2E) — verify all pass
- [x] T025 Run `cargo clippy -- -D warnings` and `cargo fmt --all --check` to verify no lint or formatting regressions
- [x] T026 Verify total new CI time stays within 5-minute budget (FR-017, SC-006)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **User Story 1 (Phase 2)**: Depends on Setup (T001-T004) — pure Rust tests, no browser needed
- **User Story 2 (Phase 3)**: Depends on Setup (T001-T003) — needs `wasm-bindgen-test` dependency + `lib.rs` + pub save functions
- **User Story 3 (Phase 4)**: Independent of US1 and US2 — different toolchain (Node.js/Playwright)
- **Polish (Phase 5)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after T004. No dependencies on other stories.
- **User Story 2 (P2)**: Can start after T003. No dependencies on other stories.
- **User Story 3 (P3)**: Can start immediately (separate toolchain). No dependencies on US1 or US2.

### Within Each User Story

- US1: T005-T009 are all parallelizable (different files/functions), T010 is the final verification
- US2: T011 must come first (test harness setup), then T012-T014 in parallel, then T015-T016 sequentially
- US3: T017-T019 are sequential (each builds on previous), T020 verifies locally, T021-T023 are CI integration

### Parallel Opportunities

- US1 and US3 can be worked on simultaneously (completely independent toolchains)
- US2 can start in parallel with US1 after T003 (different test files)
- Within US1: T005, T006, T007, T008, T009 touch different files/test modules and can be done in parallel
- Within US2: T012, T013, T014 are independent test cases in the same file, can be written in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all unit test tasks in parallel (different functions/files):
Task T005: "parse_tags tests in crates/intrada-web/src/helpers.rs"
Task T006: "parse_tempo tests in crates/intrada-web/src/helpers.rs"
Task T007: "parse_tempo_display tests in crates/intrada-web/src/helpers.rs"
Task T008: "validate_library_form tests in crates/intrada-web/src/validation.rs"
Task T009: "core event-to-effect tests in crates/intrada-web/src/core_bridge.rs"

# Then verify:
Task T010: "Run cargo test -p intrada-web"
```

## Parallel Example: User Story 2

```bash
# After T011 (harness setup), launch WASM test cases in parallel:
Task T012: "Library data round-trip WASM test"
Task T013: "Session data round-trip WASM test"
Task T014: "Empty localStorage stub data WASM test"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: User Story 1 (T005-T010)
3. **STOP and VALIDATE**: `cargo test -p intrada-web` passes with 15+ tests
4. Merge if ready — immediate value with zero new infrastructure

### Incremental Delivery

1. Setup → US1 (unit tests) → Validate → 15+ tests, no browser needed
2. Add US2 (WASM tests) → Validate → 3+ localStorage round-trip tests in headless Chrome
3. Add US3 (E2E PoC) → Validate → Playwright smoke test runs locally + CI
4. Polish → Full CI validation across all layers

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- This feature's "implementation" IS the tests — the only application source changes are structural (adding `lib.rs`, making save functions `pub`) with no behaviour changes (FR-015)
- The research document (FR-011, FR-012, SC-003) is already complete in `specs/013-web-testing/research.md`
- Core bridge effect tests (T009) use `Core<Intrada>` from `intrada-core` directly — this works in standard `cargo test` because the Crux core is platform-independent
- WASM integration tests (T012-T014) use `intrada_web::core_bridge::*` via the new `lib.rs` exports
- Commit after each phase or logical group
