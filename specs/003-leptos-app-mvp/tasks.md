# Tasks: Leptos Web App MVP

**Input**: Design documents from `/specs/003-leptos-app-mvp/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

**Tests**: No explicit test tasks — the spec does not request TDD. Existing 82 core/CLI tests must continue passing (SC-003). Manual browser verification per quickstart.md scenarios.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Workspace crate**: `crates/intrada-web/` (new crate alongside `intrada-core` and `intrada-cli`)
- **CI**: `.github/workflows/ci.yml`
- Existing core: `crates/intrada-core/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the `intrada-web` crate skeleton with all configuration files so that `trunk build` compiles a minimal WASM binary

- [X] T001 Create crate directory `crates/intrada-web/src/` and verify `Cargo.toml` workspace `members = ["crates/*"]` auto-discovers it
- [X] T002 Create `crates/intrada-web/Cargo.toml` with dependencies: leptos 0.8 (csr feature), intrada-core (path = "../intrada-core"), console_error_panic_hook, wasm-bindgen
- [X] T003 [P] Create `crates/intrada-web/Trunk.toml` with `[serve]` port 8080 and `[watch]` configuration
- [X] T004 [P] Create `crates/intrada-web/index.html` entry point with `<link data-trunk rel="rust" data-wasm-opt="z" href="Cargo.toml" />`, `<link data-trunk rel="tailwind-css" href="input.css" />`, `<noscript>` element (FR-010), and Intrada page title
- [X] T005 [P] Create `crates/intrada-web/input.css` with Tailwind v4 directives: `@import 'tailwindcss'; @source "./src/**/*.rs";`
- [X] T006 Create minimal `crates/intrada-web/src/main.rs` with `console_error_panic_hook::set_once()` and `leptos::mount::mount_to_body(App)` mounting a placeholder App component that renders "Hello Intrada"
- [X] T007 Verify `trunk build` compiles without errors in `crates/intrada-web/` and produces `crates/intrada-web/dist/` with WASM + HTML + CSS files

**Checkpoint**: `trunk build` succeeds, producing a WASM binary. `cargo test` from workspace root still passes all 82 existing tests (SC-003).

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish the Crux web shell architecture that all user story features depend on

**Warning**: No user story work can begin until this phase is complete

- [X] T008 Implement Crux web shell scaffolding in `crates/intrada-web/src/main.rs`: create `Core<Intrada>` instance, create `RwSignal<ViewModel>` for reactive state, implement `process_effects()` helper that iterates effects from `core.process_event()` and matches on `Render`/`Storage` variants
- [X] T009 Implement stub effect handlers in `crates/intrada-web/src/main.rs`: for `Storage(req)` match on `req.operation` — `LoadAll` returns hardcoded stub data (Piece: "Clair de Lune" by Debussy + Exercise: "Hanon No. 1" by Hanon per data-model.md), Save/Update/Delete are no-ops. Do NOT call `core.resolve()` on notify_shell effects (RequestHandle::Never)
- [X] T010 Wire app initialization in `crates/intrada-web/src/main.rs`: on component mount, send `Event::DataLoaded { pieces, exercises }` with stub data to core, process resulting effects, update `RwSignal<ViewModel>` from `core.view()`
- [X] T011 Verify `trunk build` still compiles and `cargo test` passes all 82 tests from workspace root

**Checkpoint**: Crux web shell architecture wired end-to-end. `Core<Intrada>` processes events and produces a ViewModel accessible via `RwSignal`. Foundation ready for user story implementation.

---

## Phase 3: User Story 1 — Web Application Shell with Styled Landing Page (Priority: P1)

**Goal**: Serve a styled landing page at the root URL with "Intrada" branding and visible Tailwind CSS styling

**Independent Test**: Run `trunk serve` from `crates/intrada-web/`, open `http://127.0.0.1:8080`, verify page shows "Intrada" title with styled content and Tailwind CSS classes are applied (quickstart.md Scenario 1)

### Implementation for User Story 1

- [X] T012 [US1] Build the landing page layout in `crates/intrada-web/src/main.rs`: App component renders a header with "Intrada" title (FR-002), a brief description paragraph, and a main content container — all using Tailwind CSS utility classes (FR-003)
- [X] T013 [US1] Add semantic HTML structure in `crates/intrada-web/src/main.rs`: use `<main>`, `<header>`, `<h1>`, `<section>` elements with appropriate ARIA attributes for Lighthouse accessibility score 90+ (SC-004)
- [X] T014 [US1] Style the landing page in `crates/intrada-web/src/main.rs` and `crates/intrada-web/input.css`: apply Tailwind v4 classes for typography, spacing, colors, and layout that demonstrate visible utility-first CSS styling
- [X] T015 [US1] Verify landing page renders correctly: run `trunk serve`, open browser to `http://127.0.0.1:8080`, confirm "Intrada" title visible with styled content (Scenario 1)

**Checkpoint**: Styled landing page renders at localhost:8080 with Intrada branding. Tailwind CSS classes are visibly applied. `cargo test` still passes.

---

## Phase 4: User Story 2 — Fast Developer Feedback Loop (Priority: P1)

**Goal**: Source code changes automatically rebuild and reload in the browser within 5 seconds

**Independent Test**: With `trunk serve` running, change a visible text string in `crates/intrada-web/src/main.rs`, save, and verify the browser updates automatically within 5 seconds (quickstart.md Scenario 2)

### Implementation for User Story 2

- [X] T016 [US2] Verify trunk auto-rebuild works: with `trunk serve` running, modify a visible text string in `crates/intrada-web/src/main.rs`, save, confirm browser reloads with updated text within 5 seconds (SC-002)
- [X] T017 [US2] Verify CSS changes auto-rebuild: with `trunk serve` running, modify a Tailwind class in `crates/intrada-web/src/main.rs`, save, confirm updated styling appears in browser
- [X] T018 [US2] Verify error reporting: introduce a deliberate compile error in `crates/intrada-web/src/main.rs`, confirm trunk reports the error clearly in the terminal (FR-006), fix the error, confirm recovery

**Checkpoint**: Developer feedback loop confirmed working. Source changes rebuild and appear in browser within 5 seconds. Compile errors display clearly in terminal.

---

## Phase 5: User Story 3 — Client-Side Interactivity (Priority: P2)

**Goal**: Render a mini library view from the Crux ViewModel showing stub data items with an interactive update button

**Independent Test**: Load the landing page, verify mini library view displays "2 item(s)" with item titles ("Clair de Lune", "Hanon No. 1") and types (piece, exercise). Click "Add Sample Item" button, verify count increases without page reload. Reload page, verify reset to 2 items (quickstart.md Scenarios 3 and 4)

### Implementation for User Story 3

- [X] T019 [US3] Create mini library view component in `crates/intrada-web/src/main.rs`: read from `RwSignal<ViewModel>` to render item count (e.g., "2 item(s)") and a list of library items with title, subtitle (composer/category), and item type badge (FR-005)
- [X] T020 [US3] Style the mini library view in `crates/intrada-web/src/main.rs`: apply Tailwind classes for card/list layout, item type badges (piece vs exercise), and responsive spacing
- [X] T021 [US3] Implement interactive update button in `crates/intrada-web/src/main.rs`: add an "Add Sample Item" button that sends a Crux event to add a new stub item, processes effects, updates `RwSignal<ViewModel>`, and re-renders the view without page reload (FR-005 acceptance scenario 2)
- [X] T022 [US3] Display ViewModel error and status fields in `crates/intrada-web/src/main.rs`: if `view_model.error` is Some, show an error banner; if `view_model.status` is Some, show a status message
- [X] T023 [US3] Verify interactive round-trip: load page, confirm 2 stub items displayed with correct titles and types. Click "Add Sample Item", confirm count increases and new item appears. Reload page, confirm reset to 2 items (Scenarios 3 and 4)

**Checkpoint**: Mini library view renders ViewModel data. Interactive button triggers Crux event cycle and updates view dynamically. Page reload resets to stub data.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: CI update, accessibility validation, and final verification across all user stories

- [X] T024 [P] Add `wasm-build` job to `.github/workflows/ci.yml`: use `dtolnay/rust-toolchain@stable` with `targets: wasm32-unknown-unknown`, `Swatinem/rust-cache@v2` with `shared-key: "wasm"`, `jetli/trunk-action@v0.5.1` for trunk install, download Tailwind CSS standalone CLI, run `trunk build` from `crates/intrada-web/` (FR-011)
- [X] T025 [P] Verify `cargo test` passes all 82 existing tests from workspace root (SC-003)
- [X] T026 [P] Verify `cargo clippy -- -D warnings` produces no new warnings across workspace (quickstart.md Scenario 6)
- [X] T027 [P] Verify `cargo fmt --all -- --check` passes formatting check
- [X] T028 [P] Verify `trunk build` produces `crates/intrada-web/dist/` directory with WASM + HTML + CSS files (quickstart.md Scenario 7)
- [X] T029 Verify `<noscript>` element renders message when JavaScript is disabled (quickstart.md Scenario 5, FR-010)
- [X] T030 Run Lighthouse accessibility audit on landing page, confirm score 90+ (SC-004, quickstart.md Scenario 8)
- [X] T031 Run full quickstart.md validation — all 8 scenarios pass

**Checkpoint**: CI pipeline updated with WASM build check. All existing tests pass. Accessibility score meets target. All quickstart scenarios validated.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational (Phase 2) — landing page structure
- **US2 (Phase 4)**: Depends on US1 (Phase 3) — needs visible content to verify auto-rebuild
- **US3 (Phase 5)**: Depends on Foundational (Phase 2) — Crux shell must be wired. Can run in parallel with US1/US2 if ViewModel rendering is prioritized
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — No dependencies on other stories
- **User Story 2 (P1)**: Depends on US1 — needs a rendered page to verify auto-rebuild behavior
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) — No dependencies on US1/US2 (uses ViewModel directly)

### Within Each User Story

- Layout/structure before styling
- Styling before verification
- Core rendering before interactive features
- Story complete before moving to next priority

### Parallel Opportunities

- **Phase 1**: T003, T004, T005 can run in parallel (different files)
- **Phase 3 + Phase 5**: US1 and US3 could technically run in parallel after Phase 2 (different concerns: layout vs ViewModel rendering)
- **Phase 6**: T024, T025, T026, T027 can all run in parallel (different validation domains)

---

## Parallel Example: Phase 1 Setup

```bash
# Launch all config files in parallel (different files, no dependencies):
Task: "Create crates/intrada-web/Trunk.toml" (T003)
Task: "Create crates/intrada-web/index.html" (T004)
Task: "Create crates/intrada-web/input.css" (T005)
```

## Parallel Example: Phase 6 Polish

```bash
# Launch all verification tasks in parallel:
Task: "Add wasm-build job to CI" (T024)
Task: "Verify cargo test passes" (T025)
Task: "Verify cargo clippy clean" (T026)
Task: "Verify cargo fmt passes" (T027)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup — crate skeleton compiles to WASM
2. Complete Phase 2: Foundational — Crux web shell wired with stub data
3. Complete Phase 3: User Story 1 — styled landing page renders
4. **STOP and VALIDATE**: Visit `http://127.0.0.1:8080`, confirm styled Intrada page
5. Demo ready with static styled landing page

### Incremental Delivery

1. Setup + Foundational → WASM binary compiles, Crux shell wired
2. Add User Story 1 → Styled landing page renders → Demo (MVP!)
3. Add User Story 2 → Auto-rebuild verified → Dev workflow complete
4. Add User Story 3 → Mini library view with interactivity → Full MVP
5. Polish → CI updated, accessibility validated, all scenarios pass

### Single Developer Strategy

Execute phases sequentially (P1 → P2 → P3 → P4 → P5 → P6). Each phase builds on the previous. Stop at any checkpoint to validate independently.

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- `console_error_panic_hook::set_once()` must be called in `main()` for browser error debugging
- Do NOT call `core.resolve()` on `notify_shell` effects — they use `RequestHandle::Never`
- Stub data resets on page reload (no persistence by design)
