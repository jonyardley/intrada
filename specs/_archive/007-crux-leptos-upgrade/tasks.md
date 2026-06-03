# Tasks: Crux & Leptos Upgrade

**Input**: Design documents from `/specs/007-crux-leptos-upgrade/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, quickstart.md

**Tests**: No new test tasks — this feature verifies that all 82 existing tests continue to pass. Test modifications are permitted if API changes require them (per clarification).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Pre-Upgrade Baseline)

**Purpose**: Capture the pre-upgrade baseline metrics needed for verification later

- [X] T001 Record pre-upgrade WASM binary size by running `trunk build --release` and noting file size of `dist/*.wasm` (for NFR-002 comparison)
- [X] T002 Verify pre-upgrade clean state: `cargo test --workspace` (82 tests), `cargo clippy --workspace -- -D warnings` (0 warnings), `cargo fmt --all -- --check` (pass)

**Checkpoint**: Baseline metrics recorded — upgrade can begin

---

## Phase 2: User Story 1 — All Existing Functionality Continues Working (Priority: P1) 🎯 MVP

**Goal**: Bump Leptos from 0.7 to 0.8 and confirm the entire codebase compiles, passes all tests, and produces a working WASM build with zero warnings

**Independent Test**: `cargo build --workspace && cargo test --workspace && cargo clippy --workspace -- -D warnings && trunk build` all succeed

### Implementation for User Story 1

- [X] T003 [US1] Update leptos version from `"0.7"` to `"0.8"` in `crates/intrada-web/Cargo.toml`
- [X] T004 [US1] Run `cargo update -p leptos` to resolve the new dependency tree and update `Cargo.lock`
- [X] T005 [US1] Build the full workspace with `cargo build --workspace` and fix any compilation errors
- [X] T006 [US1] Run `cargo test --workspace` and fix any test failures (tests may be updated to accommodate API changes but must verify the same behaviour)
- [X] T007 [US1] Run `cargo clippy --workspace -- -D warnings` and fix any warnings including deprecations
- [X] T008 [US1] Run `cargo fmt --all` to ensure formatting passes
- [X] T009 [US1] Run `trunk build` to verify the WASM build succeeds

**Checkpoint**: Leptos 0.8 compiles, all tests pass, zero warnings, WASM builds — US1 complete

---

## Phase 3: User Story 2 — Codebase Adopts Current Idiomatic Patterns (Priority: P2)

**Goal**: Replace the 0.7-era `prop:value` + `on:input` + `event_target_value` pattern with the 0.8-idiomatic `bind:value` directive in form components, while preserving all accessibility attributes

**Independent Test**: `cargo build --workspace && cargo clippy --workspace -- -D warnings` succeed; form inputs still display values, accept user input, and show validation errors

### Implementation for User Story 2

- [X] T010 [P] [US2] Migrate `<input>` in `crates/intrada-web/src/components/text_field.rs`: replace `prop:value=move || value.get()` and `on:input=move |ev| { value.set(event_target_value(&ev)); }` with `bind:value=value`; preserve `id`, `type`, `class`, `placeholder`, `required`, `aria-describedby`, `aria-invalid` attributes; remove unused `event_target_value` import if no longer needed
- [X] T011 [P] [US2] Migrate `<textarea>` in `crates/intrada-web/src/components/text_area.rs`: replace `prop:value=move || value.get()` and `on:input=move |ev| { value.set(event_target_value(&ev)); }` with `bind:value=value`; preserve `id`, `rows`, `class`, `aria-describedby`, `aria-invalid` attributes; remove unused `event_target_value` import if no longer needed
- [X] T012 [US2] Remove any now-unused `event_target_value` import from `crates/intrada-web/src/helpers.rs` or component files (verify with clippy)
- [X] T013 [US2] Run `cargo build --workspace && cargo clippy --workspace -- -D warnings` to confirm zero warnings after pattern migration
- [X] T014 [US2] Run `cargo fmt --all` to ensure formatting passes after changes

**Checkpoint**: All form components use idiomatic `bind:value`; no deprecated patterns remain — US2 complete

---

## Phase 4: User Story 3 — Dependency Versions Are Current and Compatible (Priority: P3)

**Goal**: Verify the dependency tree is clean — no conflicting versions, no duplicates, no yanked crates — and confirm crux_core 0.17.0-rc2 remains the latest published version

**Independent Test**: `cargo tree` shows no duplicate major versions of leptos or crux_core; `cargo build --workspace` succeeds from clean state

### Implementation for User Story 3

- [X] T015 [US3] Verify crux_core 0.17.0-rc2 is still the latest published version (check crates.io); document finding — no version change expected
- [X] T016 [US3] Run `cargo tree -d` to check for duplicate crate versions in the dependency tree; resolve any conflicts
- [X] T017 [US3] Verify `Cargo.lock` is clean: delete `Cargo.lock`, run `cargo build --workspace`, confirm it resolves without errors

**Checkpoint**: Dependency tree is clean and all versions are current — US3 complete

---

## Phase 5: Polish & Cross-Cutting Verification

**Purpose**: Final verification across all user stories — CI gates, binary size, and manual smoke test

- [X] T018 Run full CI gate: `cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings && cargo test --workspace && trunk build`
- [X] T019 Measure post-upgrade WASM binary size with `trunk build --release` and compare against T001 baseline (must be <120% of pre-upgrade size per NFR-002)
- [ ] T020 Manual smoke test: run `trunk serve`, navigate through all views (library list, add piece, add exercise, edit piece, edit exercise, detail view, delete confirmation, search), verify all render correctly and interactions work identically to pre-upgrade

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **US1 (Phase 2)**: Depends on Setup (T001-T002) — the version bump and compilation fix
- **US2 (Phase 3)**: Depends on US1 completion (code must compile on 0.8 before adopting new patterns)
- **US3 (Phase 4)**: Can run after US1 (dependency tree check); independent of US2
- **Polish (Phase 5)**: Depends on US1 + US2 completion

### User Story Dependencies

- **User Story 1 (P1)**: Foundation — must complete first; all other stories depend on it
- **User Story 2 (P2)**: Depends on US1 (must compile on 0.8 before migrating patterns)
- **User Story 3 (P3)**: Depends on US1 (version bump must be in place); independent of US2

### Within Each User Story

- T003 → T004 → T005 → T006/T007 (can parallel) → T008 → T009
- T010 ∥ T011 (parallel — different files) → T012 → T013 → T014
- T015 ∥ T016 (parallel — independent checks) → T017

### Parallel Opportunities

- **T010 ∥ T011**: text_field.rs and text_area.rs are independent files; can migrate simultaneously
- **T015 ∥ T016**: Version check and dependency tree check are independent
- **US2 ∥ US3**: After US1 completes, US2 and US3 can proceed in parallel (different concerns, no shared files)

---

## Parallel Example: User Story 2

```bash
# Launch both bind:value migrations together (different files):
Task: "Migrate <input> in text_field.rs: bind:value"
Task: "Migrate <textarea> in text_area.rs: bind:value"
# Then sequential: remove unused imports → clippy → fmt
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Record baseline
2. Complete Phase 2: Version bump + compile + verify (US1)
3. **STOP and VALIDATE**: All tests pass, zero warnings, WASM builds
4. This alone delivers a complete, working Leptos 0.8 upgrade

### Incremental Delivery

1. Record baseline → Foundation ready
2. Leptos 0.8 version bump → Test + verify → US1 complete (MVP!)
3. bind:value pattern migration → Verify → US2 complete
4. Dependency tree verification → US3 complete
5. Final polish + binary size check + smoke test → Feature complete

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- US1 is the critical path — if it fails, the upgrade is blocked
- US2 (pattern migration) is the only phase that modifies source files beyond Cargo.toml
- US3 is a verification-only phase — no file changes expected
- Commit after each phase checkpoint
- Total: 20 tasks across 5 phases
