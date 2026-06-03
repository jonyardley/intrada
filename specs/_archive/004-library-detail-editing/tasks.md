# Tasks: Library Add, Detail View & Editing

**Input**: Design documents from `/specs/004-library-detail-editing/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/events.md

**Tests**: No test tasks included — feature spec does not request TDD. Core logic already has 82+ tests. Web UI verified via `trunk build` + manual testing per quickstart.md.

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- All changes are in `crates/intrada-web/src/main.rs` unless otherwise noted

---

## Phase 1: Setup (View State Infrastructure)

**Purpose**: Add the shell-side view state management and refactor the App component to support full-page view routing.

- [x] T001 Define `ViewState` enum (`List`, `Detail(String)`, `AddPiece`, `AddExercise`, `EditPiece(String)`, `EditExercise(String)`) in `crates/intrada-web/src/main.rs`
- [x] T002 Add `view_state: RwSignal<ViewState>` signal alongside existing `view_model` signal in the `App` component in `crates/intrada-web/src/main.rs`
- [x] T003 Refactor `App` component to match on `ViewState` signal and render the appropriate view (initially only `List` renders the existing library list markup) in `crates/intrada-web/src/main.rs`
- [x] T004 [P] Add required imports for new Crux event types (`CreateExercise`, `UpdatePiece`, `UpdateExercise`, `PieceEvent`, `ExerciseEvent`) in `crates/intrada-web/src/main.rs`
- [x] T005 Verify `cargo clippy -- -D warnings` and `trunk build` pass with refactored App component

**Checkpoint**: App renders the same library list as before. ViewState signal exists but only `List` variant is used. No visual changes.

---

## Phase 2: Foundational (Shared Validation & Form Helpers)

**Purpose**: Build shared validation logic and form helper functions that all user story phases depend on.

**CRITICAL**: No user story work can begin until this phase is complete.

- [x] T006 Implement `validate_piece_form()` function that validates title (required, 1-500), composer (required, 1-200), notes (max 5000), bpm (1-400), tempo marking (max 100), and tags (each 1-100) — returns `HashMap<String, String>` of field errors — in `crates/intrada-web/src/main.rs`
- [x] T007 Implement `validate_exercise_form()` function that validates title (required, 1-500), composer (optional, max 200), category (optional, max 100), notes (max 5000), bpm (1-400), tempo marking (max 100), and tags (each 1-100) — returns `HashMap<String, String>` of field errors — in `crates/intrada-web/src/main.rs`
- [x] T008 [P] Implement `parse_tags()` helper function that splits comma-separated string, trims whitespace, filters empty strings, returns `Vec<String>` in `crates/intrada-web/src/main.rs`
- [x] T009 [P] Implement `parse_tempo()` helper function that takes tempo_marking and bpm strings, returns `Option<Tempo>` (None if both empty, Some if either present, validates at-least-one rule) in `crates/intrada-web/src/main.rs`
- [x] T010 [P] Create `FormFieldError` component that conditionally renders an inline error message beneath a form field (takes field name and errors signal) in `crates/intrada-web/src/main.rs`
- [x] T011 Verify `cargo clippy -- -D warnings` and `trunk build` pass after adding validation helpers

**Checkpoint**: All shared validation and helper functions are available. No visual changes yet.

---

## Phase 3: User Story 1 — Add a New Piece or Exercise (Priority: P1) MVP

**Goal**: Users can add pieces and exercises to the library via forms accessible from the list view.

**Independent Test**: Load app → click "Add" → select "Piece" → fill title + composer → Save → verify item appears in list. Repeat for Exercise with just title.

### Implementation for User Story 1

- [x] T012 [US1] Add "Add" button with dropdown/menu to the library list header offering "Piece" and "Exercise" options that set `view_state` to `AddPiece` or `AddExercise` in `crates/intrada-web/src/main.rs`
- [x] T013 [US1] Implement `AddPieceForm` component with form fields: title (required text input), composer (required text input), key (optional text input), tempo marking (optional text input), BPM (optional number input), notes (optional textarea), tags (optional text input with comma-separated hint) — each bound to `RwSignal<String>` — in `crates/intrada-web/src/main.rs`
- [x] T014 [US1] Implement submit handler for `AddPieceForm`: run `validate_piece_form()`, display inline errors via `FormFieldError` if validation fails, otherwise construct `CreatePiece` and dispatch `Event::Piece(PieceEvent::Add(...))` via core, then navigate to `ViewState::List` in `crates/intrada-web/src/main.rs`
- [x] T015 [US1] Implement Cancel button on `AddPieceForm` that navigates back to `ViewState::List` without dispatching any event in `crates/intrada-web/src/main.rs`
- [x] T016 [US1] Implement `AddExerciseForm` component with form fields: title (required text input), composer (optional text input), category (optional text input), key (optional text input), tempo marking (optional text input), BPM (optional number input), notes (optional textarea), tags (optional text input) — each bound to `RwSignal<String>` — in `crates/intrada-web/src/main.rs`
- [x] T017 [US1] Implement submit handler for `AddExerciseForm`: run `validate_exercise_form()`, display inline errors, otherwise construct `CreateExercise` and dispatch `Event::Exercise(ExerciseEvent::Add(...))` via core, then navigate to `ViewState::List` in `crates/intrada-web/src/main.rs`
- [x] T018 [US1] Implement Cancel button on `AddExerciseForm` that navigates back to `ViewState::List` in `crates/intrada-web/src/main.rs`
- [x] T019 [US1] Wire `ViewState::AddPiece` and `ViewState::AddExercise` variants into the `App` component match to render the respective form components in `crates/intrada-web/src/main.rs`
- [x] T020 [US1] Style add forms with Tailwind CSS v4: consistent field layout, labels, input styling, error states (red border + text), and button styling (Save primary, Cancel secondary) matching MVP design system in `crates/intrada-web/src/main.rs`
- [x] T021 [US1] Add ARIA attributes to all add form fields: `aria-required`, `aria-invalid`, `aria-describedby` linking to error messages, `role="form"` in `crates/intrada-web/src/main.rs`
- [x] T022 [US1] Verify `cargo clippy -- -D warnings`, `cargo fmt --all -- --check`, and `trunk build` pass after User Story 1 implementation

**Checkpoint**: Users can add pieces and exercises. New items appear in the list. Validation errors display inline. Cancel returns to list. Existing stub items and "Add Sample Item" button still work.

---

## Phase 4: User Story 2 — View Item Details (Priority: P1)

**Goal**: Users can click on any item in the library list to see its full details in a dedicated view.

**Independent Test**: Load app → click on "Clair de Lune" → verify detail view shows all fields (title, composer, key, tempo, notes, tags, dates) → click Back → returns to list.

### Implementation for User Story 2

- [x] T023 [US2] Refactor `LibraryItemCard` component to accept a click handler and make the entire card clickable, navigating to `ViewState::Detail(item.id.clone())` when clicked in `crates/intrada-web/src/main.rs`
- [x] T024 [US2] Implement `DetailView` component that receives an item ID, looks up the item from `view_model.get().items` by ID, and displays all fields: title, item type badge, composer, category (exercises only), key, tempo (marking + BPM), notes, tags, created date, and updated date. If item ID is not found, navigate back to `ViewState::List` and display an error. In `crates/intrada-web/src/main.rs`
- [x] T025 [US2] In `DetailView`, omit optional fields that are `None`/empty — do not render labels or placeholders for unset optional fields (FR-008) in `crates/intrada-web/src/main.rs`
- [x] T026 [US2] Add "Back" button to `DetailView` that navigates to `ViewState::List` in `crates/intrada-web/src/main.rs`
- [x] T027 [US2] Add "Edit" button to `DetailView` that navigates to `ViewState::EditPiece(id)` or `ViewState::EditExercise(id)` based on `item_type` in `crates/intrada-web/src/main.rs`
- [x] T028 [US2] Add "Delete" button to `DetailView` (placeholder — will be functional in US4) in `crates/intrada-web/src/main.rs`
- [x] T029 [US2] Wire `ViewState::Detail(id)` variant into the `App` component match to render `DetailView` in `crates/intrada-web/src/main.rs`
- [x] T030 [US2] Style `DetailView` with Tailwind CSS v4: structured layout with labeled fields, consistent spacing, type badge, tag chips, formatted dates, and action buttons (Back, Edit, Delete) in `crates/intrada-web/src/main.rs`
- [x] T031 [US2] Add ARIA attributes to `DetailView`: `role="article"` on container, `aria-label` on Back/Edit/Delete buttons, proper heading hierarchy in `crates/intrada-web/src/main.rs`
- [x] T032 [US2] Verify `cargo clippy -- -D warnings`, `cargo fmt --all -- --check`, and `trunk build` pass after User Story 2 implementation

**Checkpoint**: Clicking any list item opens a detail view showing all fields. Optional empty fields are omitted. Back returns to list. Edit and Delete buttons are present (Edit wired in US3, Delete in US4).

---

## Phase 5: User Story 3 — Edit an Existing Item (Priority: P2)

**Goal**: Users can edit pieces and exercises from the detail view, with pre-populated form fields and inline validation.

**Independent Test**: Load app → click "Clair de Lune" → click Edit → change title to "Clair de Lune (Revised)" → Save → verify title updated in detail view → Back → verify title updated in list.

### Implementation for User Story 3

> **Implementation Note (U2)**: `LibraryItemView` has a `subtitle` field (combines composer/category) but no separate `composer` field. For pre-populating edit forms: use `title` directly; for pieces, `subtitle` IS the composer; for exercises, `subtitle` may combine category and composer. The `category` field is available separately as `Option<String>`. Tempo is a pre-formatted display string — edit forms should parse it or, more practically, use empty fields and let the user re-enter tempo values. Alternatively, consider looking up the original `Piece`/`Exercise` from the core model if accessible. Tags are available as `Vec<String>` and should be joined with ", " for the text input.

- [x] T033 [US3] Implement `EditPieceForm` component that receives a piece ID, looks up the item from `view_model`, and pre-populates form signals (title, composer via `subtitle`, key, tempo_marking, bpm, notes, tags as comma-separated string). If item ID is not found in `view_model.items`, navigate back to `ViewState::List` and display an error (handles edge case of editing a deleted item). In `crates/intrada-web/src/main.rs`
- [x] T034 [US3] Implement submit handler for `EditPieceForm`: run `validate_piece_form()`, display inline errors if failed, otherwise construct `UpdatePiece` using `Option<Option<T>>` pattern for optional fields, dispatch `Event::Piece(PieceEvent::Update { id, input })`, then navigate to `ViewState::Detail(id)` in `crates/intrada-web/src/main.rs`
- [x] T035 [US3] Implement Cancel button on `EditPieceForm` that navigates back to `ViewState::Detail(id)` without dispatching any event in `crates/intrada-web/src/main.rs`
- [x] T036 [US3] Implement `EditExerciseForm` component that receives an exercise ID, looks up the item, and pre-populates form signals (title, composer, category, key, tempo_marking, bpm, notes, tags). If item ID is not found in `view_model.items`, navigate back to `ViewState::List` and display an error (handles edge case of editing a deleted item). In `crates/intrada-web/src/main.rs`
- [x] T037 [US3] Implement submit handler for `EditExerciseForm`: run `validate_exercise_form()`, construct `UpdateExercise` with `Option<Option<T>>` pattern for composer/category/optional fields, dispatch `Event::Exercise(ExerciseEvent::Update { id, input })`, navigate to `ViewState::Detail(id)` in `crates/intrada-web/src/main.rs`
- [x] T038 [US3] Implement Cancel button on `EditExerciseForm` that navigates back to `ViewState::Detail(id)` in `crates/intrada-web/src/main.rs`
- [x] T039 [US3] Wire `ViewState::EditPiece(id)` and `ViewState::EditExercise(id)` variants into the `App` component match to render the respective edit form components in `crates/intrada-web/src/main.rs`
- [x] T040 [US3] Style edit forms with Tailwind CSS v4: same field layout as add forms, consistent styling, pre-populated values visible, page title "Edit Piece" / "Edit Exercise" in `crates/intrada-web/src/main.rs`
- [x] T041 [US3] Add ARIA attributes to edit form fields: `aria-required`, `aria-invalid`, `aria-describedby`, `role="form"` in `crates/intrada-web/src/main.rs`
- [x] T042 [US3] Verify `cargo clippy -- -D warnings`, `cargo fmt --all -- --check`, and `trunk build` pass after User Story 3 implementation

**Checkpoint**: Users can edit any item. Form is pre-populated. Changes are reflected in detail view and list. Cancel discards changes. Validation errors display inline.

---

## Phase 6: User Story 4 — Delete an Item (Priority: P3)

**Goal**: Users can delete items from the detail view with a confirmation step.

**Independent Test**: Load app → click "Clair de Lune" → click Delete → confirm → verify item removed from list and item count decreased.

### Implementation for User Story 4

- [x] T043 [US4] Add `show_delete_confirm: RwSignal<bool>` state to `DetailView` component (or pass as signal) for toggling the inline delete confirmation banner in `crates/intrada-web/src/main.rs`
- [x] T044 [US4] Implement inline delete confirmation banner in `DetailView`: "Are you sure you want to delete [title]?" with "Confirm" and "Cancel" buttons, shown when `show_delete_confirm` is true in `crates/intrada-web/src/main.rs`
- [x] T045 [US4] Wire the existing "Delete" button in `DetailView` to set `show_delete_confirm` to true in `crates/intrada-web/src/main.rs`
- [x] T046 [US4] Implement Confirm handler: dispatch `Event::Piece(PieceEvent::Delete { id })` or `Event::Exercise(ExerciseEvent::Delete { id })` based on `item_type`, then navigate to `ViewState::List` in `crates/intrada-web/src/main.rs`
- [x] T047 [US4] Implement Cancel handler on confirmation banner: set `show_delete_confirm` to false, item remains unchanged in `crates/intrada-web/src/main.rs`
- [x] T048 [US4] Style delete confirmation banner with Tailwind CSS v4: red/warning color scheme, clear confirm/cancel buttons, visually distinct from detail content in `crates/intrada-web/src/main.rs`
- [x] T049 [US4] Add ARIA attributes to confirmation: `role="alertdialog"`, `aria-label` on confirm/cancel buttons in `crates/intrada-web/src/main.rs`
- [x] T050 [US4] Verify `cargo clippy -- -D warnings`, `cargo fmt --all -- --check`, and `trunk build` pass after User Story 4 implementation

**Checkpoint**: Users can delete items with two-step confirmation. Item removed from list. Canceling confirmation keeps item. All CRUD operations (add, view, edit, delete) now functional.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final quality pass across all user stories.

- [x] T051 [P] Run full `cargo test --workspace` to verify all 82+ existing tests still pass (SC-006)
- [x] T052 [P] Run `cargo clippy -- -D warnings` across entire workspace — zero warnings
- [x] T053 [P] Run `cargo fmt --all -- --check` — formatting clean
- [x] T054 Verify `trunk build` succeeds for WASM build in `crates/intrada-web`
- [x] T055 Run quickstart.md manual verification checklist in browser (Chrome, Firefox, and Safari per SC-007): list view, add piece, add exercise, validation, detail view, edit, delete, cancel flows
- [x] T056 [P] Verify Unicode handling in all forms: test with accented characters (Dvořák), CJK text, and special characters
- [x] T057 [P] Review all components for consistent Tailwind styling: spacing, colors, typography, hover/focus states
- [x] T058 Verify the "Add Sample Item" button from MVP still works alongside new "Add" forms (spec assumption)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion — BLOCKS all user stories
- **US1 - Add (Phase 3)**: Depends on Phase 2 completion — uses validation helpers
- **US2 - Detail View (Phase 4)**: Depends on Phase 2 completion — independent of US1
- **US3 - Edit (Phase 5)**: Depends on Phase 2 AND Phase 4 (needs DetailView for navigation back)
- **US4 - Delete (Phase 6)**: Depends on Phase 2 AND Phase 4 (needs DetailView for confirmation UI)
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1 - Add)**: Can start after Phase 2. No dependencies on other stories.
- **US2 (P1 - Detail View)**: Can start after Phase 2. No dependencies on other stories. Can run in parallel with US1.
- **US3 (P2 - Edit)**: Requires US2 (Detail View) — edit is accessed from the detail view and navigates back to it.
- **US4 (P3 - Delete)**: Requires US2 (Detail View) — delete confirmation appears within the detail view.

### Within Each User Story

- Component structure and signals first
- Submit/action handlers next
- View state wiring into App match
- Styling and ARIA last
- Clippy/build verification at end

### Parallel Opportunities

- **Phase 2**: T008, T009, T010 can run in parallel (different helper functions)
- **Phase 3 + Phase 4**: US1 (Add forms) and US2 (Detail view) can run in parallel after Phase 2
- **Phase 5 + Phase 6**: US3 (Edit) and US4 (Delete) both depend on US2 but could run in parallel with each other
- **Phase 7**: T051, T052, T053, T056, T057 can all run in parallel

---

## Parallel Example: After Phase 2

```bash
# US1 and US2 can start simultaneously:
# Stream A: User Story 1 — Add forms
Task: "T012 [US1] Add 'Add' button with dropdown..."
Task: "T013 [US1] Implement AddPieceForm..."
# ... through T022

# Stream B: User Story 2 — Detail view
Task: "T023 [US2] Refactor LibraryItemCard to be clickable..."
Task: "T024 [US2] Implement DetailView component..."
# ... through T032
```

---

## Implementation Strategy

### MVP First (US1 + US2 Only)

1. Complete Phase 1: Setup (T001-T005)
2. Complete Phase 2: Foundational (T006-T011)
3. Complete Phase 3: US1 - Add (T012-T022)
4. Complete Phase 4: US2 - Detail View (T023-T032)
5. **STOP and VALIDATE**: Add items, view details, verify all flows
6. This delivers the core "add and view" capability

### Incremental Delivery

1. Setup + Foundational → view routing infrastructure ready
2. US1 (Add) → users can create items → validate independently
3. US2 (Detail View) → users can see full details → validate independently
4. US3 (Edit) → users can modify items → validate independently
5. US4 (Delete) → users can remove items → validate independently
6. Polish → cross-cutting quality, accessibility, and build verification

### Single Developer Strategy (Recommended)

1. Phase 1 → Phase 2 → Phase 3 (US1) → Phase 4 (US2) → Phase 5 (US3) → Phase 6 (US4) → Phase 7
2. Commit after each phase checkpoint
3. Each phase builds on the previous, all in `crates/intrada-web/src/main.rs`

---

## Notes

- All 58 tasks target `crates/intrada-web/src/main.rs` — the Crux core is unchanged
- [P] tasks = different functions/components, no dependencies
- [Story] label maps task to specific user story for traceability
- The `UpdatePiece`/`UpdateExercise` types use `Option<Option<T>>` — `Some(None)` to clear, `Some(Some(v))` to set
- Tags are comma-separated text input, parsed on submit via `parse_tags()`
- Delete uses inline confirmation banner within the detail view, not `window.confirm()`
- All effects are handled by existing `process_effects()` — save/update/delete are no-ops
- Commit after each phase checkpoint for clean git history
