# Tasks: Unified Library Item Form

**Input**: Design documents from `/specs/009-unified-library-form/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Establish baseline and verify existing state before making changes

- [X] T001 Run `cargo test --workspace` and record the baseline test count (expected: 82+ pass). Run `cargo clippy --workspace -- -D warnings` and `cargo fmt --all -- --check` to confirm zero warnings and formatting compliance.
- [X] T002 Run `trunk build --release` from `crates/intrada-web/` and record the baseline WASM binary size for post-change comparison.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Create the shared types and components that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T003 Add the `ItemType` enum to `crates/intrada-web/src/types.rs`: Define `#[derive(Clone, Copy, PartialEq, Eq)] pub enum ItemType { Piece, Exercise }` alongside the existing `SharedCore` type. Export it from the module.
- [X] T004 Create the `TypeTabs` component in `crates/intrada-web/src/components/type_tabs.rs`: (a) Accept props: `active: Signal<ItemType>` (read-only signal for current tab), `on_change: Option<Callback<ItemType>>` (None = display-only mode). (b) Render a `<div role="tablist">` containing two `<button role="tab">` elements ("Piece" and "Exercise"). (c) Active tab: `aria-selected="true"`, `tabindex="0"`, styled with `bg-indigo-600 text-white` (matching project's Primary button). (d) Inactive tab (interactive, when `on_change` is Some): `aria-selected="false"`, `tabindex="-1"`, styled with `bg-white border border-slate-300 text-slate-700 hover:bg-slate-50`, click handler calls `on_change(ItemType)`. (e) Inactive tab (display-only, when `on_change` is None): `aria-selected="false"`, `tabindex="-1"`, styled with `bg-slate-100 text-slate-400 cursor-default`, no click handler, `aria-disabled="true"`. (f) Keyboard handling on the tablist: `on:keydown` for ArrowLeft/ArrowRight to move focus between tabs, Enter/Space to activate (only when interactive). (g) Add `aria-controls="tabpanel-piece"` / `aria-controls="tabpanel-exercise"` to each button. (h) Wrap the tab content area below with `<div id="tabpanel-piece" role="tabpanel">` (or exercise) as appropriate.
- [X] T005 Register `type_tabs` module in `crates/intrada-web/src/components/mod.rs`: Add `pub mod type_tabs;` and re-export `TypeTabs` in the `pub use` block.
- [X] T006 Add the unified validation function in `crates/intrada-web/src/validation.rs`: (a) Add `use crate::types::ItemType;` import. (b) Add `pub fn validate_library_form(item_type: ItemType, title: &str, composer: &str, category: &str, notes: &str, bpm_str: &str, tempo_marking: &str, tags_str: &str) -> HashMap<String, String>`. (c) Shared validations: title required 1-500 chars, notes max 5000, bpm optional 1-400, tempo_marking max 100, tags each max 100. (d) When `item_type == ItemType::Piece`: composer required 1-200 chars, category ignored. (e) When `item_type == ItemType::Exercise`: composer optional max 200 if present, category optional max 100 if present. (f) Keep `validate_piece_form()` and `validate_exercise_form()` temporarily — they will be removed in Polish phase after all callers are migrated.
- [X] T007 Run `cargo build --workspace` to verify foundational changes compile cleanly (all three validation functions should coexist without conflict).

**Checkpoint**: `ItemType` enum, `TypeTabs` component, and unified `validate_library_form()` are ready. User story implementation can now begin.

---

## Phase 3: User Story 1 — Add Library Item via Unified Form with Type Tabs (Priority: P1) 🎯 MVP

**Goal**: Replace the two separate "Add" forms with a single unified form featuring Piece/Exercise tabs. Users can switch tabs, form fields adapt dynamically, shared field values persist across switches, and the correct item type is created on submission.

**Independent Test**: Open the add form at `/library/new`, verify tabs appear, switch between tabs and confirm form fields change (Composer required/optional, Category appears/disappears), fill out and submit for each type, verify the correct item is created. Run quickstart.md Scenarios 1–4 and 8.

### Implementation for User Story 1

- [X] T008 [US1] Create the unified add form in `crates/intrada-web/src/views/add_form.rs`: (a) Define `#[component] pub fn AddLibraryItemForm(view_model: RwSignal<ViewModel>, core: SharedCore) -> impl IntoView`. (b) Create signals: `active_tab: RwSignal<ItemType>` initialized to `ItemType::Piece`, `title`, `composer`, `category`, `key_sig`, `tempo_marking`, `bpm`, `notes`, `tags_input` (all `RwSignal<String>::new(String::new())`), `errors: RwSignal<HashMap<String, String>>`. (c) Render `<BackLink label="Cancel" href="/".to_string() />` and `<PageHeading text="Add Library Item" />`. (d) Render `<TypeTabs active=Signal::derive(move || active_tab.get()) on_change=Some(Callback::new(move |tab: ItemType| { active_tab.set(tab); errors.set(HashMap::new()); })) />` — note: errors are cleared on tab switch (FR-007). (e) Inside a `<Card>`, render the `<form>` with `on:submit` handler. (f) Render shared fields: Title (`TextField`, required=true), Key, Tempo Marking + BPM (grid-cols-2), Notes (`TextArea`), Tags. (g) For the Composer field: use a conditional block — when `active_tab.get() == ItemType::Piece`, render `<TextField id="add-composer" label="Composer *" value=composer required=true field_name="composer" errors=errors />`. When `active_tab.get() == ItemType::Exercise`, render `<TextField id="add-composer" label="Composer" value=composer required=false field_name="composer" errors=errors />`. Both bind to the same `composer` signal (research.md decision: Option B — two TextFields sharing one signal). (h) For the Category field: conditionally render only when `active_tab.get() == ItemType::Exercise` — `<TextField id="add-category" label="Category" value=category field_name="category" errors=errors />`. (i) Render Save (Primary, submit) and Cancel (Secondary, navigates to "/") buttons. (j) In the submit handler: call `validate_library_form(active_tab.get(), &title.get(), &composer.get(), &category.get(), &notes.get(), &bpm.get(), &tempo_marking.get(), &tags_input.get())`. If errors, set errors signal and return. (k) If Piece: build `CreatePiece { title, composer, key, tempo, notes, tags }` → `Event::Piece(PieceEvent::Add(...))`. (l) If Exercise: build `CreateExercise { title, composer: Option, category: Option, key, tempo, notes, tags }` → `Event::Exercise(ExerciseEvent::Add(...))`. (m) Call `process_effects` then `navigate("/", NavigateOptions { replace: true, ..Default::default() })`. (n) Wrap the form body in a `<div id="tabpanel-piece" role="tabpanel">` or `<div id="tabpanel-exercise" role="tabpanel">` based on `active_tab` (matching the `aria-controls` in TypeTabs).
- [X] T009 [US1] Update `crates/intrada-web/src/views/mod.rs`: (a) Remove `pub mod add_piece;` and `pub mod add_exercise;`. (b) Add `pub mod add_form;`. (c) Update the `pub use` re-exports: remove `AddPieceForm` and `AddExerciseForm`, add `AddLibraryItemForm` from `add_form`.
- [X] T010 [US1] Update routes in `crates/intrada-web/src/app.rs`: (a) Replace the import of `AddPieceForm, AddExerciseForm` with `AddLibraryItemForm` from `crate::views`. (b) Remove the two route entries for `/pieces/new` and `/exercises/new`. (c) Add a single route: `<Route path=path!("/library/new") view=move || view! { <AddLibraryItemForm view_model=view_model core=core_cloneN.clone() /> } />`. (d) **IMPORTANT**: Place this route BEFORE the `/library/:id` route to avoid the `:id` parameter matching "new" as an ID. (e) Remove two of the now-unnecessary `core_clone` variables (net reduction: 6 clones → 4 clones, since we go from 6 routes to 4).
- [X] T011 [US1] Update the library list in `crates/intrada-web/src/views/library_list.rs`: (a) Remove the `show_add_menu: RwSignal<bool>` signal. (b) Remove the dropdown `<div>` with the two `<A href="/pieces/new">` and `<A href="/exercises/new">` links. (c) Replace with a single `<A href="/library/new" attr:class="inline-flex items-center justify-center rounded-lg bg-indigo-600 px-4 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 transition-colors">` containing the text "Add Item" (with a "+" icon if desired). (d) Preserve the existing header layout (flex justify-between with the library count).
- [X] T012 [US1] Delete the old form files: `crates/intrada-web/src/views/add_piece.rs` and `crates/intrada-web/src/views/add_exercise.rs`.
- [X] T013 [US1] Run `cargo build --workspace` to verify US1 changes compile cleanly.
- [X] T014 [US1] Run `cargo test --workspace` to verify all 82+ existing tests still pass (no core changes, but validates no accidental breakage).
- [X] T015 [US1] Run `cargo clippy --workspace -- -D warnings` and `cargo fmt --all -- --check` to confirm zero warnings and formatting compliance.
- [X] T016 [US1] Manual verification: run `trunk serve` from `crates/intrada-web/` and execute quickstart.md Scenarios 1 (add piece), 2 (add exercise), 3 (tab switching preserves fields), 4 (validation per tab), and 8 (submission uses correct type). Document pass/fail.

**Checkpoint**: The unified add form is fully functional. Users can add both pieces and exercises from a single tabbed form. This is the MVP — US1 delivers the core feature value.

---

## Phase 4: User Story 2 — Edit Form Adapts to Item Type (Priority: P2)

**Goal**: Replace the two separate edit forms with a single unified edit form. The form opens with display-only tabs pre-selected based on the item's type. Fields are pre-populated with existing data. Type cannot be changed during editing.

**Independent Test**: Open an existing piece for editing at `/library/{id}/edit`, verify Piece tab is selected and disabled, fields are pre-populated. Open an existing exercise, verify Exercise tab is selected. Run quickstart.md Scenario 5.

### Implementation for User Story 2

- [X] T017 [US2] Create the unified edit form in `crates/intrada-web/src/views/edit_form.rs`: (a) Define `#[component] pub fn EditLibraryItemForm(view_model: RwSignal<ViewModel>, core: SharedCore) -> impl IntoView`. (b) Extract item ID from route params using `use_params_map()`. (c) Find the item in `view_model.get_untracked().items` by ID. If not found, return "Item not found" view with back link to `/`. (d) Determine `item_type: ItemType` from `item.item_type` — if `"piece"` → `ItemType::Piece`, else → `ItemType::Exercise`. This is a plain value, not a signal (display-only). (e) Pre-populate signals: `title` from `item.title`, `key_sig` from `item.key.unwrap_or_default()`, `tempo_marking` and `bpm` from `parse_tempo_display(&item.tempo)`, `notes` from `item.notes.unwrap_or_default()`, `tags_input` from `item.tags.join(", ")`. (f) Pre-populate composer: if Piece → `item.subtitle.clone()`. If Exercise → use the existing recovery logic from edit_exercise.rs: if `item.category.is_some()` then `String::new()` (can't recover composer), else `item.subtitle.clone()`. (g) Pre-populate category: `item.category.clone().unwrap_or_default()`. (h) Render `<BackLink label="Cancel" href=format!("/library/{}", item_id) />` and `<PageHeading text="Edit Library Item" />`. (i) Render `<TypeTabs active=Signal::derive(move || item_type) on_change=None />` — display-only tabs (FR-015). (j) Render form fields identical to add_form.rs but with pre-populated signal values. Use the same conditional Composer (two TextFields) and conditional Category pattern. For Piece: show Composer as required. For Exercise: show Composer as optional, show Category. (k) Submit handler: validate with `validate_library_form(item_type, ...)`. (l) If Piece: build `UpdatePiece` with double-Option pattern (`Some(None)` for empty optional fields, `Some(Some(value))` for set values) → `Event::Piece(PieceEvent::Update { id, input })`. (m) If Exercise: build `UpdateExercise` with same double-Option pattern → `Event::Exercise(ExerciseEvent::Update { id, input })`. (n) After processing effects, navigate to `/library/{id}` with `NavigateOptions { replace: true }`.
- [X] T018 [US2] Update `crates/intrada-web/src/views/mod.rs`: (a) Remove `pub mod edit_piece;` and `pub mod edit_exercise;`. (b) Add `pub mod edit_form;`. (c) Update re-exports: remove `EditPieceForm` and `EditExerciseForm`, add `EditLibraryItemForm` from `edit_form`.
- [X] T019 [US2] Update routes in `crates/intrada-web/src/app.rs`: (a) Replace the import of `EditPieceForm, EditExerciseForm` with `EditLibraryItemForm` from `crate::views`. (b) Remove the two route entries for `/pieces/:id/edit` and `/exercises/:id/edit`. (c) Add a single route: `<Route path=path!("/library/:id/edit") view=move || view! { <EditLibraryItemForm view_model=view_model core=core_cloneN.clone() /> } />`. (d) Reduce the total `core_clone` count to match the final 4 routes.
- [X] T020 [US2] Update the detail view edit link in `crates/intrada-web/src/views/detail.rs`: (a) Remove the `type_for_edit` variable and the conditional `if type_for_edit == "piece" { format!("/pieces/{}/edit", id) } else { format!("/exercises/{}/edit", id) }`. (b) Replace with `let edit_href = format!("/library/{}/edit", id_for_edit);`. (c) Remove the now-unused `type_for_edit` clone.
- [X] T021 [US2] Delete the old edit form files: `crates/intrada-web/src/views/edit_piece.rs` and `crates/intrada-web/src/views/edit_exercise.rs`.
- [X] T022 [US2] Run `cargo build --workspace` to verify US2 changes compile cleanly.
- [X] T023 [US2] Run `cargo test --workspace` to verify all 82+ existing tests still pass.
- [X] T024 [US2] Run `cargo clippy --workspace -- -D warnings` and `cargo fmt --all -- --check` to confirm zero warnings and formatting compliance.
- [X] T025 [US2] Manual verification: run `trunk serve` and execute quickstart.md Scenario 5 (edit form display-only tabs). Test editing both a piece and an exercise. Verify tabs are non-interactive, fields are pre-populated, and save works correctly. Document pass/fail.

**Checkpoint**: Both add and edit forms are unified. The core feature is complete — users interact with a single form experience for both creating and editing library items.

---

## Phase 5: User Story 3 — Unified URL Structure for Add Form (Priority: P3)

**Goal**: Verify that the route consolidation is complete: old routes (`/pieces/new`, `/exercises/new`, `/pieces/:id/edit`, `/exercises/:id/edit`) are fully removed and return 404. The library list has a single "Add Item" button. URL does not change on tab switch.

**Independent Test**: Navigate to old URLs and verify 404. Verify single "Add Item" button. Switch tabs on the add form and verify URL stays `/library/new`. Run quickstart.md Scenario 6.

### Implementation for User Story 3

- [X] T026 [US3] Verify old route removal: Run `trunk serve` and navigate to `http://localhost:8080/pieces/new` — verify NotFoundView renders. Navigate to `http://localhost:8080/exercises/new` — verify NotFoundView renders. Navigate to `http://localhost:8080/pieces/{id}/edit` (using a known stub data ID) — verify NotFoundView renders. Navigate to `http://localhost:8080/exercises/{id}/edit` — verify NotFoundView renders. Document pass/fail. If any old route still works, check `app.rs` for leftover route entries and remove them.
- [X] T027 [US3] Verify URL stability on tab switch: Navigate to `/library/new`, switch between Piece and Exercise tabs, verify the URL remains `/library/new` throughout (tab state is local signal, not in URL). Document pass/fail.
- [X] T028 [US3] Verify single "Add Item" button: Navigate to `/` and inspect the library list header. Verify there is a single "Add Item" button (not a dropdown). Click it and verify it navigates to `/library/new`. Document pass/fail.

**Checkpoint**: All route and navigation changes are verified. Old routes are removed. The unified URL structure is in place.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, accessibility, performance, and cleanup

- [X] T029 Run full CI gate: `cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings && cargo test --workspace` — all must pass with zero warnings.
- [X] T030 Run `trunk build --release` from `crates/intrada-web/` and compare WASM binary size against the T002 baseline. The binary size should decrease or remain stable (removing 4 files, adding 2 smaller unified files + 1 small component). Record both sizes.
- [X] T031 Remove old validation functions from `crates/intrada-web/src/validation.rs`: Delete `validate_piece_form()` and `validate_exercise_form()` (all callers now use `validate_library_form()`). Remove any now-unused imports.
- [X] T032 Remove any dead code: search for unused imports of `AddPieceForm`, `AddExerciseForm`, `EditPieceForm`, `EditExerciseForm`, `validate_piece_form`, `validate_exercise_form`, or orphaned `show_add_menu` references across all files in `crates/intrada-web/src/`. Run `cargo clippy` to detect dead code warnings.
- [X] T033 Keyboard accessibility verification: Run quickstart.md Scenario 7 — navigate the add form tabs using keyboard only (Tab, ArrowLeft, ArrowRight, Enter, Space). Verify tabs are focusable and activatable. On the edit form, verify tabs receive focus but cannot be switched. Document pass/fail.
- [X] T034 Final smoke test: run through ALL 8 quickstart.md scenarios end-to-end. Document pass/fail for each scenario.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories (creates ItemType, TypeTabs, unified validation)
- **User Story 1 (Phase 3)**: Depends on Foundational — creates unified add form, updates routes and library list
- **User Story 2 (Phase 4)**: Depends on Foundational — creates unified edit form, updates routes and detail view. Can be done in parallel with US1 if needed (different files), but sequential execution is recommended since US2 shares the same app.rs route changes
- **User Story 3 (Phase 5)**: Depends on US1 + US2 completion — verification-only phase confirming all route changes are correct
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Phase 2 only. Delivers the MVP.
- **User Story 2 (P2)**: Depends on Phase 2 only. Can technically start in parallel with US1, but recommended sequential due to shared app.rs route modifications.
- **User Story 3 (P3)**: Depends on US1 + US2. This is a verification phase — no new code, just confirming the route consolidation from US1 and US2 works correctly.

### Within Each User Story

- Create the new unified component first
- Update mod.rs exports
- Update app.rs routes
- Update dependent views (library_list, detail)
- Delete old files
- Build → Test → Clippy → Fmt
- Manual verification via quickstart scenarios

### Parallel Opportunities

- T001 and T002 (Setup) can run in parallel
- T003, T004, T005, T006 (Foundational) must be sequential: T003 first (defines ItemType), then T004 (uses ItemType in TypeTabs), then T005 (registers module), then T006 (uses ItemType in validation)
- Within US1: T008 must complete before T009-T012 (T009 updates mod.rs to reference add_form, T010 updates routes, T011 updates list, T012 deletes old files)
- Within US2: T017 must complete before T018-T021 (same pattern)
- US3 tasks (T026-T028) are all independent manual verification tasks and can run in parallel

---

## Parallel Example: User Story 3

```bash
# All US3 tasks are manual verification — can run simultaneously:
Task: "T026 [US3] Verify old route removal"
Task: "T027 [US3] Verify URL stability on tab switch"
Task: "T028 [US3] Verify single Add Item button"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (baseline metrics)
2. Complete Phase 2: Foundational (ItemType, TypeTabs, unified validation)
3. Complete Phase 3: User Story 1 (unified add form)
4. **STOP and VALIDATE**: Test US1 independently — can users add both pieces and exercises from the tabbed form?
5. At this point the app works: unified add form at `/library/new`, old add routes removed, single "Add Item" button. Edit forms still use old separate routes.

### Incremental Delivery

1. Setup + Foundational → Shared infrastructure ready
2. Add User Story 1 → Unified add form → Test independently → **MVP Delivered!**
3. Add User Story 2 → Unified edit form → Test independently → **Full feature!**
4. Add User Story 3 → Route verification → **Complete!**
5. Polish → Final validation and cleanup

---

## Notes

- [P] tasks = different files, no dependencies on each other
- [Story] label maps task to specific user story for traceability
- US3 is primarily a verification phase — the route changes happen in US1 (add routes) and US2 (edit routes)
- All manual testing requires `trunk serve` running from `crates/intrada-web/`
- Stub data regenerates on each app load — item ULIDs will differ between sessions
- No core domain changes (intrada-core) — all changes are in intrada-web
- Old validation functions (`validate_piece_form`, `validate_exercise_form`) are kept during Phases 2–5 and removed in Polish (T031) to avoid compile errors during incremental migration
- Commit after each phase completion (not each individual task) to keep the codebase in a compilable state
