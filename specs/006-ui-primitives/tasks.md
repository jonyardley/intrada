# Tasks: UI Primitive Components

**Input**: Design documents from `/specs/006-ui-primitives/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, quickstart.md

**Tests**: Not requested — this is a pure UI refactoring with no new logic. Existing 82 tests verify correctness (SC-003).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- All file paths relative to `crates/intrada-web/src/`

---

## Phase 1: Setup

**Purpose**: Create new component files and register them in the module system

- [X] T001 [P] Create ButtonVariant enum and Button component in crates/intrada-web/src/components/button.rs — render `<button>` with Tailwind classes per variant (Primary: `inline-flex items-center gap-1.5 rounded-lg bg-indigo-600 px-3.5 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 transition-colors`, Secondary: `inline-flex items-center gap-1.5 rounded-lg bg-white px-3.5 py-2 text-sm font-medium text-slate-700 border border-slate-300 hover:bg-slate-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-400 transition-colors`, Danger: `inline-flex items-center gap-1.5 rounded-lg bg-red-600 px-3.5 py-2 text-sm font-medium text-white hover:bg-red-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-600 transition-colors`, DangerOutline: `inline-flex items-center gap-1.5 rounded-lg bg-white px-3.5 py-2 text-sm font-medium text-red-600 border border-red-300 hover:bg-red-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-400 transition-colors`); props: variant (ButtonVariant), on_click (Callback<ev::MouseEvent>), button_type (optional &'static str, default "button"), children (Children)
- [X] T002 [P] Create TextField component in crates/intrada-web/src/components/text_field.rs — wrap `<label>`, `<input>`, and `<FormFieldError>` with standard styling; props: id (&'static str), label (&'static str), value (RwSignal<String>), required (bool), placeholder (Option<&'static str>), field_name (&'static str), errors (RwSignal<HashMap<String, String>>), input_type (Option<&'static str>, default "text")
- [X] T003 [P] Create TextArea component in crates/intrada-web/src/components/text_area.rs — wrap `<label>`, `<textarea>`, and `<FormFieldError>` with same label/error styling as TextField; props: id (&'static str), label (&'static str), value (RwSignal<String>), rows (Option<u32>, default 3), field_name (&'static str), errors (RwSignal<HashMap<String, String>>)
- [X] T004 [P] Create Card component in crates/intrada-web/src/components/card.rs — render `<div class="bg-white rounded-xl shadow-sm border border-slate-200 p-6">` with Children slot
- [X] T005 [P] Create TypeBadge component in crates/intrada-web/src/components/type_badge.rs — render `<span>` with violet-100 classes for "piece", emerald-100 classes for "exercise", slate-100 classes for unknown types; props: item_type (String)
- [X] T006 [P] Create PageHeading component in crates/intrada-web/src/components/page_heading.rs — render `<h2 class="text-2xl font-bold text-slate-900 mb-6">` with text prop (&'static str)
- [X] T007 [P] Create BackLink component in crates/intrada-web/src/components/back_link.rs — render `<button class="mb-6 inline-flex items-center gap-1 text-sm text-slate-500 hover:text-slate-700 transition-colors">` with "\u{2190}" prefix; props: label (&'static str), on_click (Callback<ev::MouseEvent>)
- [X] T008 [P] Create FieldLabel component in crates/intrada-web/src/components/field_label.rs — render `<dt class="text-xs font-medium text-slate-400 uppercase tracking-wider">` with text prop (&'static str)
- [X] T009 [P] Create AppHeader component in crates/intrada-web/src/components/app_header.rs — extract the full `<header>` block from app.rs including app name "Intrada", tagline, and version badge with all existing classes and ARIA attributes
- [X] T010 [P] Create AppFooter component in crates/intrada-web/src/components/app_footer.rs — extract the full `<footer>` block from app.rs with all existing classes and ARIA attributes

---

## Phase 2: Foundational (Module Registration)

**Purpose**: Register all new components in the module system — MUST complete before any view updates

**⚠️ CRITICAL**: No view file updates can begin until this phase is complete

- [X] T011 Update crates/intrada-web/src/components/mod.rs — add `pub mod` declarations for all 10 new component files (button, text_field, text_area, card, type_badge, page_heading, back_link, field_label, app_header, app_footer) and corresponding `pub use` re-exports for Button, ButtonVariant (includes Primary, Secondary, Danger, DangerOutline), TextField, TextArea, Card, TypeBadge, PageHeading, BackLink, FieldLabel, AppHeader, AppFooter

**Checkpoint**: All components are importable via `crate::components::*` — run `cargo build -p intrada-web` to verify

---

## Phase 3: User Story 1 — Replace Inline Button Styles (Priority: P1) 🎯 MVP

**Goal**: Replace all inline button CSS class strings with the shared Button component across all views

**Independent Test**: `cargo build -p intrada-web && cargo clippy -p intrada-web -- -D warnings` — verify all buttons render via Button component and no inline button class strings remain in view files

### Implementation for User Story 1

- [X] T012 [US1] Update crates/intrada-web/src/views/add_piece.rs — replace the Save submit button (line 188-193) and Cancel button (line 194-200) with `<Button>` using ButtonVariant::Primary (button_type="submit") and ButtonVariant::Secondary respectively; replace the back-navigation button (line 33-38) with `<BackLink label="Cancel" on_click=... />`; add `use crate::components::{Button, ButtonVariant, BackLink};` to imports
- [X] T013 [P] [US1] Update crates/intrada-web/src/views/add_exercise.rs — same button replacements as T012: Save (submit, Primary), Cancel (Secondary), back-navigation button with `<BackLink label="Cancel" on_click=... />`
- [X] T014 [P] [US1] Update crates/intrada-web/src/views/edit_piece.rs — same button replacements as T012: Save (submit, Primary), Cancel (Secondary), back-navigation button with `<BackLink label="Cancel" on_click=... />`
- [X] T015 [P] [US1] Update crates/intrada-web/src/views/edit_exercise.rs — same button replacements as T012: Save (submit, Primary), Cancel (Secondary), back-navigation button with `<BackLink label="Cancel" on_click=... />`
- [X] T016 [US1] Update crates/intrada-web/src/views/detail.rs — replace Edit button (line 198-208) with ButtonVariant::Primary, Delete button (line 210-215) with ButtonVariant::DangerOutline, Confirm Delete button (line 74-89) with ButtonVariant::Danger, Cancel in delete banner (line 90-95) with ButtonVariant::Secondary; replace back-navigation button (lines 55-60) with `<BackLink label="Back to Library" on_click=... />`; add imports
- [X] T017 [US1] Update crates/intrada-web/src/views/library_list.rs — replace the "+ Add" dropdown trigger button (line 74-82) with ButtonVariant::Primary; note: the dropdown menu item buttons and "Add Sample" button have unique styling so they remain inline

**Checkpoint**: All standard buttons use the Button component. Run `cargo build -p intrada-web` to verify compilation.

---

## Phase 4: User Story 2 — Replace Inline Form Field Markup (Priority: P1)

**Goal**: Replace all inline label-input-error patterns in form views with TextField and TextArea components

**Independent Test**: `cargo build -p intrada-web && cargo clippy -p intrada-web -- -D warnings` — verify all form fields use shared components and validation still works

### Implementation for User Story 2

- [X] T018 [US2] Update crates/intrada-web/src/views/add_piece.rs — replace all 7 form field blocks with TextField/TextArea components: Title (required, id="piece-title"), Composer (required, id="piece-composer"), Key (optional, placeholder), Tempo Marking (optional, placeholder), BPM (optional, input_type="number", placeholder), Notes (TextArea, rows=3), Tags (optional, placeholder); remove `use crate::components::FormFieldError;` since TextField/TextArea handle it internally
- [X] T019 [P] [US2] Update crates/intrada-web/src/views/add_exercise.rs — replace all 8 form field blocks with TextField/TextArea: Title (required), Composer (optional), Category (optional, placeholder), Key (optional, placeholder), Tempo Marking (optional, placeholder), BPM (input_type="number"), Notes (TextArea), Tags (placeholder); remove FormFieldError import
- [X] T020 [P] [US2] Update crates/intrada-web/src/views/edit_piece.rs — replace all 7 form field blocks with TextField/TextArea matching the same IDs and config as add_piece but with "edit-piece-" prefixed IDs; remove FormFieldError import
- [X] T021 [P] [US2] Update crates/intrada-web/src/views/edit_exercise.rs — replace all 8 form field blocks with TextField/TextArea matching add_exercise config but with "edit-exercise-" prefixed IDs; remove FormFieldError import

**Checkpoint**: All form fields use shared components. Run `cargo build -p intrada-web` and verify form validation still works.

---

## Phase 5: User Story 3 — Extract Layout Shell Components (Priority: P2)

**Goal**: Replace inline header and footer markup in app.rs with AppHeader and AppFooter components

**Independent Test**: `cargo build -p intrada-web && cargo clippy -p intrada-web -- -D warnings` — verify app.rs uses shared layout components

### Implementation for User Story 3

- [X] T022 [US3] Update crates/intrada-web/src/app.rs — replace the inline `<header>` block (lines 37-50) with `<AppHeader />` and the inline `<footer>` block (lines 121-125) with `<AppFooter />`; add `use crate::components::{AppHeader, AppFooter};` to imports

**Checkpoint**: App component focuses solely on routing. Run `cargo build -p intrada-web`.

---

## Phase 6: User Story 4 — Extract Card, Badge, and Typography Components (Priority: P2)

**Goal**: Replace inline Card, TypeBadge, PageHeading, BackLink, and FieldLabel markup in views with shared components

**Independent Test**: `cargo build -p intrada-web && cargo clippy -p intrada-web -- -D warnings` — verify all typography and container patterns use shared components

### Implementation for User Story 4

- [X] T023 [US4] Update crates/intrada-web/src/views/add_piece.rs — replace `<h2>` heading (line 40) with `<PageHeading text="Add Piece" />`, wrap `<form>` content with `<Card>` (replace the form's card class `bg-white rounded-xl shadow-sm border border-slate-200 p-6` with Card component, keep `space-y-5` on an inner form element); BackLink already handled in US1 (T012)
- [X] T024 [P] [US4] Update crates/intrada-web/src/views/add_exercise.rs — same as T023: PageHeading "Add Exercise", Card wrapper; BackLink already handled in US1 (T013)
- [X] T025 [P] [US4] Update crates/intrada-web/src/views/edit_piece.rs — same as T023: PageHeading "Edit Piece", Card wrapper; BackLink already handled in US1 (T014)
- [X] T026 [P] [US4] Update crates/intrada-web/src/views/edit_exercise.rs — same as T023: PageHeading "Edit Exercise", Card wrapper; BackLink already handled in US1 (T015)
- [X] T027 [US4] Update crates/intrada-web/src/views/detail.rs — keep the `<h2>` heading inline (it's inside a flex container with TypeBadge — PageHeading's `mb-6` would break this layout), replace TypeBadge inline span (lines 118-124) with `<TypeBadge item_type=item_type.clone() />`, replace Card container `<div class="bg-white...">` (line 105) with `<Card>`, replace all `<dt>` labels (lines 132, 140, 148, 159, 169) with `<FieldLabel text="..." />`; BackLink already handled in US1 (T016)
- [X] T028 [US4] Update crates/intrada-web/src/components/library_item_card.rs — replace inline badge_classes logic (lines 22-26) with `<TypeBadge item_type=item_type />` component; keep the card-specific `<li>` styling since LibraryItemCard has unique hover/cursor/shadow behaviour distinct from the Card component

**Checkpoint**: All card, badge, heading, back-link, and label patterns use shared components. Run `cargo build -p intrada-web`.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final verification and cleanup across all files

- [X] T029 Run `cargo build --workspace` to verify full workspace compilation
- [X] T030 Run `cargo clippy --workspace -- -D warnings` to verify zero clippy warnings (SC-008)
- [X] T031 Run `cargo test --workspace` to verify all 82 existing tests pass (SC-003)
- [X] T032 Run `cd crates/intrada-web && trunk build` to verify WASM build succeeds (SC-004)
- [X] T033 Verify no individual file exceeds 300 lines using `wc -l` on all .rs files in intrada-web/src/ (SC-005)
- [X] T034 Verify inline button class strings (`rounded-lg bg-indigo-600`, `rounded-lg bg-white.*border-slate-300`) do not appear in view files — only in components/button.rs (SC-001)
- [X] T035 Verify inline form label class strings (`block text-sm font-medium text-slate-700 mb-1`) and input class strings (`w-full rounded-lg border border-slate-300`) do not appear in view files — only in components/text_field.rs and components/text_area.rs (SC-002)
- [X] T036 Run quickstart.md verification scenarios V1-V4 (automated) and document results
- [ ] T037 Run quickstart.md verification scenarios V5-V6 (manual visual smoke test) — verify all pages look identical to pre-refactoring (SC-006)
- [X] T038 Measure CSS duplication reduction — count inline CSS class strings in view files before and after, verify at least 50% reduction (SC-007)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — all 10 component files can be created in parallel
- **Foundational (Phase 2)**: Depends on Phase 1 completion — registers all components in mod.rs
- **User Stories (Phases 3-6)**: All depend on Phase 2 completion (components must be importable)
  - US1 (Phase 3) and US2 (Phase 4) are P1 priority — execute sequentially or in parallel
  - US3 (Phase 5) and US4 (Phase 6) are P2 priority — execute after P1 stories
  - US3 is independent of US1/US2 (only touches app.rs)
  - US4 depends on US1 being complete (BackLink used in US1 tasks; US4 may need to adjust)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (Buttons)**: Can start after Phase 2 — no dependencies on other stories
- **US2 (Form Fields)**: Can start after Phase 2 — independent of US1 (different markup patterns)
- **US3 (Layout Shell)**: Can start after Phase 2 — only touches app.rs
- **US4 (Card/Badge/Typography)**: Can start after Phase 2 — some tasks touch same files as US1/US2, so should execute after them to avoid merge conflicts

### Within Each User Story

- Tasks touching different files marked [P] can run in parallel
- Tasks touching the same file must be sequential
- Each story is independently verifiable with `cargo build`

### Parallel Opportunities

Phase 1 — all 10 component files are independent:
```text
T001 (button.rs) | T002 (text_field.rs) | T003 (text_area.rs) | T004 (card.rs) | T005 (type_badge.rs) | T006 (page_heading.rs) | T007 (back_link.rs) | T008 (field_label.rs) | T009 (app_header.rs) | T010 (app_footer.rs)
```

Phase 3 (US1) — form views are independent:
```text
T012 (add_piece) → then T013 (add_exercise) | T014 (edit_piece) | T015 (edit_exercise) in parallel
→ then T016 (detail) → T017 (library_list)
```

Phase 4 (US2) — form views are independent:
```text
T018 (add_piece) → then T019 (add_exercise) | T020 (edit_piece) | T021 (edit_exercise) in parallel
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Create all 10 component files [P]
2. Complete Phase 2: Register in mod.rs
3. Complete Phase 3: Replace all button markup (US1)
4. Complete Phase 4: Replace all form field markup (US2)
5. **STOP and VALIDATE**: Run `cargo build && cargo clippy && cargo test`
6. This covers the highest-impact duplication (buttons + form fields = ~75% of total)

### Full Delivery

7. Complete Phase 5: Extract header/footer (US3)
8. Complete Phase 6: Extract card/badge/typography (US4)
9. Complete Phase 7: Full verification suite
10. Visual smoke test per quickstart.md V5-V6

### Incremental Delivery

Each phase is independently verifiable:
- After Phase 3: All buttons use shared components ✓
- After Phase 4: All form fields use shared components ✓
- After Phase 5: Layout shell extracted ✓
- After Phase 6: All primitives extracted ✓
- After Phase 7: Full verification complete ✓

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- This is a pure refactoring — no new user-facing functionality
- All existing 82 tests must continue to pass without modification
- The form `<form>` element's `on:submit` handler and `class="... space-y-5"` should remain on the form itself, not be absorbed into the Card component — Card wraps visual container styling only
- The delete confirmation banner in detail.rs uses Danger (solid red confirm) and Secondary (cancel) Button variants
- The detail.rs "Delete" trigger button uses DangerOutline variant (white bg, red text, red border)
- All button variants include `inline-flex items-center gap-1.5`, `px-3.5`, and `focus-visible:outline-*` for accessibility — this matches the library_list.rs "+ Add" button's existing classes
- The "Add Sample" button in library_list.rs uses unique slate-200 styling — this is not a standard variant and should remain inline
- The detail.rs `<h2>` heading remains inline because it's inside a flex container with TypeBadge — PageHeading's `mb-6` would break this layout
- BackLink replacements are all handled in US1 (Phase 3), not US4, since back-navigation buttons are button-styled elements
