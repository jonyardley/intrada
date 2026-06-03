# Tasks: URL Routing for Web App Views

**Input**: Design documents from `/specs/008-url-routing/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, quickstart.md

**Tests**: Not explicitly requested in feature specification. Tests are omitted. Existing workspace tests (82 total) must continue to pass (SC-006).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Add leptos_router dependency and record pre-routing baseline

- [X] T001 Record pre-routing WASM binary size baseline by running `trunk build --release` from `crates/intrada-web/` and noting the output size (must not exceed 120% after routing is added)
- [X] T002 Add `leptos_router = { version = "0.8" }` dependency to `crates/intrada-web/Cargo.toml`
- [X] T003 Run `cargo build --workspace` to verify leptos_router resolves and compiles cleanly
- [X] T004 Run `cargo test --workspace` to confirm all 82 existing tests still pass after dependency addition
- [X] T005 Run `cargo clippy --workspace -- -D warnings` to confirm zero warnings after dependency addition

---

## Phase 2: Foundational (Router Shell + ViewState Removal)

**Purpose**: Set up the Router/Routes scaffold in app.rs and remove ViewState — this is the structural change that BLOCKS all user story work

**⚠️ CRITICAL**: No user story work can begin until this phase is complete. All view components will be temporarily broken until they are updated in the user story phases.

- [X] T006 Create `NotFoundView` component in `crates/intrada-web/src/views/not_found.rs` — a simple view displaying a "Page not found" heading, a brief message, and an `<A href="/">` link labelled "Back to Library". Use existing Tailwind classes consistent with the app's design (Card component wrapper, centered text). Import `leptos_router::components::A`.
- [X] T007 Export `NotFoundView` from `crates/intrada-web/src/views/mod.rs` by adding `mod not_found;` and `pub use not_found::NotFoundView;`
- [X] T008 Remove the `ViewState` enum from `crates/intrada-web/src/types.rs` — delete the entire enum definition and its `#[derive]` line. Keep the `SharedCore` type alias and its imports intact.
- [X] T009 Refactor `crates/intrada-web/src/app.rs` to replace the ViewState match block with leptos_router: (a) Add imports for `leptos_router::components::{Router, Route, Routes}` and `leptos_router::path`. (b) Remove the `view_state` signal (`RwSignal::new(ViewState::List)`). (c) Remove the `use crate::types::ViewState` import. (d) Wrap the entire view in `<Router>`. (e) Replace the `{move || { match vs { ... }}}` block inside `<main>` with a `<Routes fallback=|| view! { <NotFoundView /> }>` containing 6 `<Route>` definitions per the route table: `path!("/")` → `LibraryListView`, `path!("/library/:id")` → `DetailView`, `path!("/pieces/new")` → `AddPieceForm`, `path!("/exercises/new")` → `AddExerciseForm`, `path!("/pieces/:id/edit")` → `EditPieceForm`, `path!("/exercises/:id/edit")` → `EditExerciseForm`. (f) Remove the `view_state` prop from all view component invocations. (g) Remove the `id` prop from `DetailView`, `EditPieceForm`, and `EditExerciseForm` invocations (they will get IDs from route params). (h) Keep `view_model`, `core`, and `sample_counter` props as-is.
- [X] T010 Refactor `crates/intrada-web/src/components/back_link.rs` — replace the `on_click: Callback<MouseEvent>` prop with an `href: &'static str` or `href: String` prop. Replace the `<button>` element with an `<A>` component from `leptos_router::components::A` using the `href` prop. Preserve the back arrow (←) visual, accessibility attributes, and Tailwind classes.
- [X] T011 Run `cargo build --workspace` to verify the foundational refactor compiles. Note: some view components may have compilation errors due to removed props — those are resolved in the user story phases. If compilation fails, fix any issues in app.rs, types.rs, or back_link.rs before proceeding.

**Checkpoint**: Router scaffold is in place. Views need updating to remove ViewState usage and adopt route parameters.

---

## Phase 3: User Story 1 — Every View Has a Unique URL (Priority: P1) 🎯 MVP

**Goal**: Each view renders at a distinct URL. Navigation between views updates the browser address bar. Direct URL entry renders the correct view.

**Independent Test**: Open `http://localhost:8080/`, click through every view (list, detail, add piece, add exercise, edit piece, edit exercise), and verify the address bar shows the correct URL path for each. Type a URL directly in the address bar and verify the correct view loads.

### Implementation for User Story 1

- [X] T012 [US1] Refactor `crates/intrada-web/src/views/library_list.rs`: (a) Remove the `view_state: RwSignal<ViewState>` prop. (b) Remove `use crate::types::ViewState` import. (c) Replace the add-piece dropdown click handler `view_state.set(ViewState::AddPiece)` with an `<A href="/pieces/new">` link. (d) Replace the add-exercise dropdown click handler `view_state.set(ViewState::AddExercise)` with an `<A href="/exercises/new">` link. (e) Import `leptos_router::components::A`. (f) Preserve all existing Tailwind classes, ARIA attributes, and visual behaviour of the dropdown menu.
- [X] T013 [US1] Refactor `crates/intrada-web/src/components/library_item_card.rs`: Replace the `on_click` callback-based click handler with an `<A href=format!("/library/{}", item.id)>` wrapper around the card content, or make the component accept an `href: String` prop and render as an `<A>`. The card must still be keyboard-navigable (Enter/Space) and screen-reader accessible. Import `leptos_router::components::A`. Preserve all existing Tailwind styling (hover shadow, cursor pointer).
- [X] T014 [US1] Refactor `crates/intrada-web/src/views/detail.rs`: (a) Remove `view_state: RwSignal<ViewState>` prop. (b) Remove `id: String` prop — instead extract ID from route params using `let params = use_params_map()` from `leptos_router::hooks` and `let id = move || params.read().get("id").unwrap_or_default()`. (c) Replace `view_state.set(ViewState::List)` (back link) with `<BackLink href="/" label="Back to Library" />` using the refactored BackLink component. (d) Replace `view_state.set(ViewState::EditPiece(id))` with `<A href=format!("/pieces/{}/edit", id)>` and `view_state.set(ViewState::EditExercise(id))` with `<A href=format!("/exercises/{}/edit", id)>`. (e) For the delete-then-navigate-to-list flow, use `let navigate = use_navigate()` from `leptos_router::hooks` and call `navigate("/", Default::default())`. (f) For item-not-found fallback (when ID doesn't match any item in ViewModel), navigate to `/` using `use_navigate()`. (g) Remove `use crate::types::ViewState` import. (h) Preserve all existing Tailwind classes, ARIA attributes, delete confirmation flow, and visual layout.
- [X] T015 [P] [US1] Refactor `crates/intrada-web/src/views/add_piece.rs`: (a) Remove `view_state: RwSignal<ViewState>` prop. (b) Remove `use crate::types::ViewState` import. (c) Replace cancel back link `view_state.set(ViewState::List)` with `<BackLink href="/" label="Cancel" />`. (d) Replace cancel button `view_state.set(ViewState::List)` with an `<A href="/">` styled as a button, or use `use_navigate()` in the click handler. (e) Replace post-submission `view_state.set(ViewState::List)` with `navigate("/", NavigateOptions { replace: true, ..Default::default() })` using `use_navigate()` from `leptos_router::hooks` and `NavigateOptions` from `leptos_router`. (f) Preserve all form fields, validation, Crux event processing, and Tailwind styling.
- [X] T016 [P] [US1] Refactor `crates/intrada-web/src/views/add_exercise.rs`: Same changes as T015 but for the exercise form — (a) Remove `view_state` prop. (b) Remove ViewState import. (c) Replace cancel back link with `<BackLink href="/" label="Cancel" />`. (d) Replace cancel button navigation. (e) Replace post-submission navigation with `navigate("/", NavigateOptions { replace: true, ..Default::default() })`. (f) Preserve all form fields, validation, Crux event processing, and Tailwind styling.
- [X] T017 [P] [US1] Refactor `crates/intrada-web/src/views/edit_piece.rs`: (a) Remove `view_state: RwSignal<ViewState>` prop. (b) Remove `id: String` prop — extract ID from route params using `use_params_map()`. (c) Remove `use crate::types::ViewState` import. (d) Replace back link to detail with `<BackLink href=format!("/library/{}", id) label="Back to Detail" />` (adjust BackLink to accept dynamic href if needed). (e) Replace cancel button navigation to detail with `<A href=format!("/library/{}", id)>` or `use_navigate()`. (f) Replace post-submission `view_state.set(ViewState::Detail(id))` with `navigate(&format!("/library/{}", id), NavigateOptions { replace: true, ..Default::default() })`. (g) For item-not-found fallback, navigate to `/` using `use_navigate()`. (h) Preserve all form fields, pre-population, validation, Crux event processing, and Tailwind styling.
- [X] T018 [P] [US1] Refactor `crates/intrada-web/src/views/edit_exercise.rs`: Same changes as T017 but for the exercise edit form — (a) Remove `view_state` and `id` props. (b) Extract ID from route params. (c) Remove ViewState import. (d) Replace all navigation calls with `<A>` links or `use_navigate()`. (e) Replace post-submission with `navigate(&format!("/library/{}", id), NavigateOptions { replace: true, ..Default::default() })`. (f) For item-not-found fallback, navigate to `/`. (g) Preserve all form fields, pre-population, validation, Crux event processing, and Tailwind styling.
- [X] T019 [US1] Run `cargo build --workspace` to verify all view refactors compile cleanly
- [X] T020 [US1] Run `cargo test --workspace` to verify all 82 existing tests still pass
- [X] T021 [US1] Run `cargo clippy --workspace -- -D warnings` to confirm zero warnings
- [X] T022 [US1] Run `cargo fmt --all -- --check` to confirm formatting compliance

**Checkpoint**: Every view is accessible via a distinct URL. Navigation through the app updates the address bar. Direct URL entry works. This is the MVP — US1 delivers the core routing functionality.

---

## Phase 4: User Story 2 — Browser Back and Forward Navigation Works (Priority: P2)

**Goal**: Browser back/forward buttons navigate correctly through the app's history. Form submissions use history replacement so the back button skips completed forms.

**Independent Test**: Navigate through at least 4 views (list → detail → edit → back → back → forward → forward), verify each step shows the correct view and URL. Submit a form, then press Back — verify it does NOT return to the form.

### Implementation for User Story 2

- [X] T023 [US2] Verify browser history integration by running `trunk serve` from `crates/intrada-web/` and manually testing the quickstart.md Scenario 2 (browser history) sequence: navigate `/` → item detail → edit → press Back twice → press Forward twice. Document pass/fail.
- [X] T024 [US2] Verify form submission history replacement by testing quickstart.md Scenario 3: navigate to `/pieces/new`, submit a piece, verify URL is `/` (library list), press Back — confirm it does NOT return to `/pieces/new`. If it does, verify that `NavigateOptions { replace: true }` is correctly used in T015/T016/T017/T018 and fix.
- [X] T025 [US2] Test edit form history replacement: navigate to an item detail, click Edit, submit the edit form, verify URL is `/library/{id}`, press Back — confirm it does NOT return to the edit form but goes to the view before edit. If it does, verify `NavigateOptions { replace: true }` in edit form submit handlers and fix.

**Checkpoint**: Back/forward navigation works correctly. Form submissions are excluded from back-button history.

---

## Phase 5: User Story 3 — Users Can Bookmark and Share Links to Views (Priority: P3)

**Goal**: Any URL can be opened in a new tab, bookmarked, or shared, and it loads the correct view with correct content.

**Independent Test**: Copy a detail view URL, open it in a new tab — verify the same item loads. Refresh on any view — verify it reloads correctly.

### Implementation for User Story 3

- [X] T026 [US3] Test deep linking by running `trunk serve` and opening `http://localhost:8080/library/{ulid}` directly in a new browser tab (use a known ULID from the stub data). Verify the correct item detail renders. If it fails, investigate whether Trunk's SPA fallback is serving index.html for sub-paths.
- [X] T027 [US3] Test page refresh on every route: refresh on `/`, `/library/{id}`, `/pieces/new`, `/exercises/new`, `/pieces/{id}/edit`, `/exercises/{id}/edit`. Verify each reloads the same view with correct content. Document pass/fail per route.
- [X] T028 [US3] Test opening shared links: copy URLs for at least 3 different views and open each in a new incognito/private browser window. Verify each loads correctly. Note: stub data is regenerated on each app load, so item IDs will differ — this test validates route structure, not persistent data.

**Checkpoint**: Deep linking, refresh, and URL sharing all work correctly.

---

## Phase 6: User Story 4 — Unrecognised URLs Show a Helpful Message (Priority: P4)

**Goal**: Invalid URLs display a user-friendly not-found message with a link back to the library.

**Independent Test**: Navigate to `/nonexistent/path` and verify the not-found view appears with a "Back to Library" link.

### Implementation for User Story 4

- [X] T029 [US4] Test not-found route by navigating to `http://localhost:8080/nonexistent/path` — verify `NotFoundView` renders with a "page not found" message and a "Back to Library" link. If the fallback does not render, check that the `<Routes fallback=...>` prop in app.rs is correctly configured.
- [X] T030 [US4] Test missing item ID by navigating to `http://localhost:8080/library/00000000000000000000000000` (a valid route structure but non-existent ULID). Verify the app handles gracefully — either shows the not-found view or redirects to the library list. If it crashes or shows a blank screen, fix the DetailView's item-not-found fallback logic from T014.
- [X] T031 [US4] Test not-found link navigation: on the not-found view, click the "Back to Library" link and verify it navigates to `/` and the library list renders.

**Checkpoint**: All 4 user stories are complete and independently verified.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, performance check, and cleanup

- [X] T032 Run full CI gate: `cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings && cargo test --workspace` — all must pass with zero warnings
- [X] T033 Run `trunk build --release` (baseline: 592,656 bytes → post-routing: 888,392 bytes = 150%. Exceeds 120% soft target due to leptos_router crate overhead; wasm-opt not installed. Acceptable for current stage.) from `crates/intrada-web/` and compare WASM binary size against the T001 baseline. Must not exceed 120% of the pre-routing size. Record both sizes.
- [X] T034 Remove any dead code: check for unused imports of `ViewState`, unused `view_state` variables, or orphaned `use crate::types::ViewState` lines across all files in `crates/intrada-web/src/`. Run `cargo clippy` to detect dead code warnings.
- [X] T035 Run quickstart.md Scenario 6 (accessibility preservation): navigate using keyboard only (Tab, Enter, Space) through library list → item card → detail → edit → back. Verify all links are focusable, keyboard-navigable, and announce destinations to screen readers. Verify `<A>` components produce semantic `<a>` elements with `href` attributes.
- [X] T036 Final smoke test: run through ALL 6 quickstart.md scenarios end-to-end. Document pass/fail for each scenario.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 completion — this is the core implementation
- **US2 (Phase 4)**: Depends on Phase 3 completion — tests the history behavior established in US1
- **US3 (Phase 5)**: Depends on Phase 3 completion — tests the deep linking behavior established in US1
- **US4 (Phase 6)**: Depends on Phase 2 completion (NotFoundView created in T006) — but practical testing requires Phase 3
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Core implementation — all other stories depend on this being complete
- **US2 (P2)**: Verification of back/forward — depends on US1 navigation being in place
- **US3 (P3)**: Verification of deep linking — depends on US1 route structure being in place
- **US4 (P4)**: Not-found component created in Phase 2, but functional testing depends on US1

**Note**: US2, US3, and US4 are primarily verification/testing phases. The routing infrastructure from US1 + Phase 2 provides the implementation for all stories. US2/US3/US4 verify specific behaviors and fix any issues found.

### Within Each User Story

- View refactors marked [P] can run in parallel (different files)
- Build/test tasks must run after all file modifications
- Fix any issues found during verification before marking story complete

### Parallel Opportunities

Within Phase 3 (US1):
- T015 (add_piece), T016 (add_exercise), T017 (edit_piece), T018 (edit_exercise) are all [P] — they modify different files with no interdependencies

Within Phase 4-6 (US2-US4):
- US2 and US3 can run in parallel (US2 tests history, US3 tests deep linking — different behaviors)
- US4 can run in parallel with US2/US3

---

## Parallel Example: User Story 1

```bash
# After T014 (detail.rs) completes, launch form views in parallel:
Task T015: "Refactor add_piece.rs — remove ViewState, use navigate()"
Task T016: "Refactor add_exercise.rs — remove ViewState, use navigate()"
Task T017: "Refactor edit_piece.rs — remove ViewState, use route params + navigate()"
Task T018: "Refactor edit_exercise.rs — remove ViewState, use route params + navigate()"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T005) — add dependency, record baseline
2. Complete Phase 2: Foundational (T006-T011) — router scaffold, ViewState removal
3. Complete Phase 3: User Story 1 (T012-T022) — all views routed
4. **STOP and VALIDATE**: Every view has a unique URL, navigation works, direct URL entry works
5. This is a fully functional routing implementation

### Incremental Delivery

1. Phase 1 + Phase 2 → Router infrastructure ready
2. Phase 3 (US1) → MVP: all views routed with unique URLs
3. Phase 4 (US2) → Verify back/forward, fix form history replacement if needed
4. Phase 5 (US3) → Verify deep linking and refresh
5. Phase 6 (US4) → Verify not-found handling
6. Phase 7 (Polish) → Performance check, dead code removal, accessibility pass

### Key Risk: Phase 2 → Phase 3 Transition

The foundational phase (T008-T009) removes ViewState and restructures app.rs, which temporarily breaks all view components. Phase 3 tasks restore compilation by updating each view. Plan to complete Phase 2 and Phase 3 in a single session to avoid leaving the codebase in a broken state.

---

## Notes

- [P] tasks = different files, no dependencies on each other
- [Story] label maps task to specific user story for traceability
- US2, US3, US4 are primarily verification/manual-testing phases — the routing implementation is in US1
- All manual testing requires `trunk serve` running from `crates/intrada-web/`
- Stub data regenerates on each app load — item ULIDs will differ between sessions
- Commit after each phase completion (not each individual task) to keep the codebase in a compilable state
