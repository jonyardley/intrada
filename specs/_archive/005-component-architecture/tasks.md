# Tasks: Web App Component Architecture

**Input**: Design documents from `/specs/005-component-architecture/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing. This is a pure refactoring — all tasks involve moving existing code from `crates/intrada-web/src/main.rs` into new files with appropriate `pub` visibility and `use` imports. No new functionality is added.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- All source changes are within `crates/intrada-web/src/`
- Current state: single file `main.rs` (1,906 lines)
- Target state: 17 files across `src/`, `src/components/`, `src/views/`

---

## Phase 1: Setup (Module Structure)

**Purpose**: Create the directory structure and module skeleton so that subsequent phases can populate individual files.

- [X] T001 Create directory `crates/intrada-web/src/components/`
- [X] T002 Create directory `crates/intrada-web/src/views/`
- [X] T003 Create `crates/intrada-web/src/types.rs` with `ViewState` enum (lines 26-34) and `SharedCore` type alias (line 20) moved from `main.rs`. Add necessary imports (`send_wrapper::SendWrapper`, `std::cell::RefCell`, `std::rc::Rc`, `crux_core::Core`, `intrada_core::Intrada`). Make both `pub`: `pub type SharedCore` and `pub enum ViewState`
- [X] T004 [P] Create `crates/intrada-web/src/components/mod.rs` with placeholder `pub mod form_field_error;` and `pub mod library_item_card;` declarations
- [X] T005 [P] Create `crates/intrada-web/src/views/mod.rs` with placeholder `pub mod` declarations for all six view files: `library_list`, `detail`, `add_piece`, `add_exercise`, `edit_piece`, `edit_exercise`
- [X] T006 Add module declarations to `crates/intrada-web/src/main.rs`: `mod types;`, `mod helpers;`, `mod validation;`, `mod data;`, `mod core_bridge;`, `mod components;`, `mod views;`, `mod app;` — and add `use app::App;` import. Remove the moved `ViewState` and `SharedCore` definitions from `main.rs`, replacing with `use types::{ViewState, SharedCore};` where needed
- [X] T007 Verify `cargo check -p intrada-web` compiles (expect errors for missing module contents — confirms structure is correct)

**Checkpoint**: Directory structure and module skeleton in place. Compilation will have errors because module files are empty/stubs — that is expected. The key validation is that the module graph is correct.

---

## Phase 2: Foundational (Shared Non-Visual Modules)

**Purpose**: Extract all shared non-visual logic into dedicated modules. These modules are used by components and views in later phases.

**CRITICAL**: Phases 3-6 depend on these modules being complete.

- [X] T008 [P] Create `crates/intrada-web/src/helpers.rs` — move `parse_tags()` (lines 42-48), `parse_tempo()` (lines 52-77), and `parse_tempo_display()` (lines 1880-1906) from `main.rs`. Add `use intrada_core::domain::types::Tempo;` import. Make all three functions `pub`
- [X] T009 [P] Create `crates/intrada-web/src/validation.rs` — move `validate_piece_form()` (lines 79-175) and `validate_exercise_form()` (lines 177-273) from `main.rs`. Add `use std::collections::HashMap;` import. Make both functions `pub`
- [X] T010 [P] Create `crates/intrada-web/src/data.rs` — move `create_stub_data()` (lines 280-336) and `SAMPLE_PIECES` constant (lines 337-348) from `main.rs`. Add required imports for `Piece`, `Exercise`, `Tempo`, `chrono::Utc`, `ulid::Ulid`. Make `create_stub_data` `pub` and `SAMPLE_PIECES` `pub(crate)`
- [X] T011 [P] Create `crates/intrada-web/src/core_bridge.rs` — move `process_effects()` (lines 308-335) from `main.rs`. Add imports for `crux_core::Core`, `intrada_core::{Intrada, Event, ViewModel}`, `intrada_core::app::{Effect, StorageEffect}`, `leptos::prelude::RwSignal`, and `crate::data::create_stub_data`. Make `process_effects` `pub`
- [X] T012 Remove the moved functions, constants, and type definitions from `crates/intrada-web/src/main.rs` — replace with `use` imports from the new modules. The `main()` function (lines 274-277) stays in `main.rs`
- [X] T013 Verify `cargo check -p intrada-web` compiles successfully with all foundational modules extracted
- [X] T014 Verify `cargo clippy -p intrada-web -- -D warnings` passes with zero warnings

**Checkpoint**: All non-visual shared logic is extracted. `main.rs` now contains only the `main()` entry point, the `App` component, all view components, and the two shared components. `cargo check` and `cargo clippy` pass.

---

## Phase 3: User Story 1 — Extract Shared UI Building Blocks (Priority: P1) MVP

**Goal**: Move `FormFieldError` and `LibraryItemCard` shared components into `components/` directory.

**Independent Test**: `cargo check -p intrada-web` compiles. `cargo test` passes (82+ tests). Components are importable via `use crate::components::{FormFieldError, LibraryItemCard};`.

### Implementation for User Story 1

- [X] T015 [P] [US1] Create `crates/intrada-web/src/components/form_field_error.rs` — move the `FormFieldError` component (lines 468-483) from `main.rs`. Add `use leptos::prelude::*;` and `use std::collections::HashMap;` imports. Make the component function `pub`
- [X] T016 [P] [US1] Create `crates/intrada-web/src/components/library_item_card.rs` — move the `LibraryItemCard` component (lines 651-744) from `main.rs`. Add imports: `use leptos::prelude::*;`, `use leptos::ev;`, `use intrada_core::LibraryItemView;`, and `use wasm_bindgen::JsCast;`. Make the component function `pub`
- [X] T017 [US1] Update `crates/intrada-web/src/components/mod.rs` — ensure `pub mod form_field_error;` and `pub mod library_item_card;` are declared, and add re-exports: `pub use form_field_error::FormFieldError;` and `pub use library_item_card::LibraryItemCard;`
- [X] T018 [US1] Remove the moved `FormFieldError` and `LibraryItemCard` component functions from `crates/intrada-web/src/main.rs`. Add `use crate::components::{FormFieldError, LibraryItemCard};` to the remaining view components that reference them
- [X] T019 [US1] Verify `cargo check -p intrada-web` compiles and `cargo clippy -p intrada-web -- -D warnings` passes with zero warnings

**Checkpoint**: Shared building blocks are in `components/` directory. All remaining components in `main.rs` reference them via imports. Application compiles cleanly.

---

## Phase 4: User Story 2 — Organise Views into Logical Groups (Priority: P1)

**Goal**: Move all six view components into the `views/` directory, each in its own file.

**Independent Test**: `cargo check -p intrada-web` compiles. `cargo test` passes (82+ tests). Each view is in its own file under `views/`. No file exceeds 300 lines.

### Implementation for User Story 2

- [X] T020 [P] [US2] Create `crates/intrada-web/src/views/library_list.rs` — move `LibraryListView` component (lines 480-649) from `main.rs`. Add imports: `use leptos::prelude::*;`, `use leptos::ev;`, `use intrada_core::{Event, Intrada, ViewModel, LibraryItemView};`, `use intrada_core::domain::piece::PieceEvent;`, `use intrada_core::domain::types::CreatePiece;`, `use crate::types::{ViewState, SharedCore};`, `use crate::components::LibraryItemCard;`, `use crate::data::SAMPLE_PIECES;`, `use crate::core_bridge::process_effects;`. Make the component function `pub`. Include any closure/helper logic local to the list view
- [X] T021 [P] [US2] Create `crates/intrada-web/src/views/detail.rs` — move `DetailView` component (lines 740-946) from `main.rs`. Add imports: `use leptos::prelude::*;`, `use crux_core::Core;`, `use intrada_core::{Event, Intrada, ViewModel};`, `use intrada_core::domain::piece::PieceEvent;`, `use intrada_core::domain::exercise::ExerciseEvent;`, `use crate::types::{ViewState, SharedCore};`, `use crate::core_bridge::process_effects;`, `use crate::helpers::parse_tempo_display;`. Make the component function `pub`
- [X] T022 [P] [US2] Create `crates/intrada-web/src/views/add_piece.rs` — move `AddPieceForm` component (lines 951-1141) from `main.rs`. Add imports: `use leptos::prelude::*;`, `use leptos::ev;`, `use std::collections::HashMap;`, `use chrono::Utc;`, `use ulid::Ulid;`, `use intrada_core::{Event, Intrada, ViewModel};`, `use intrada_core::domain::piece::PieceEvent;`, `use intrada_core::domain::types::{CreatePiece, Tempo};`, `use crate::types::{ViewState, SharedCore};`, `use crate::components::FormFieldError;`, `use crate::helpers::{parse_tags, parse_tempo};`, `use crate::validation::validate_piece_form;`, `use crate::core_bridge::process_effects;`. Make the component function `pub`
- [X] T023 [P] [US2] Create `crates/intrada-web/src/views/add_exercise.rs` — move `AddExerciseForm` component (lines 1146-1360) from `main.rs`. Add imports: same pattern as `add_piece.rs` but with `ExerciseEvent`, `CreateExercise`, and `validate_exercise_form`. Make the component function `pub`
- [X] T024 [P] [US2] Create `crates/intrada-web/src/views/edit_piece.rs` — move `EditPieceForm` component (lines 1364-1590) from `main.rs`. Add imports: same pattern as `add_piece.rs` plus `UpdatePiece` and `parse_tempo_display`. Make the component function `pub`
- [X] T025 [P] [US2] Create `crates/intrada-web/src/views/edit_exercise.rs` — move `EditExerciseForm` component (lines 1594-1879) from `main.rs`. Add imports: same pattern as `add_exercise.rs` plus `UpdateExercise` and `parse_tempo_display`. Make the component function `pub`
- [X] T026 [US2] Update `crates/intrada-web/src/views/mod.rs` — ensure all six `pub mod` declarations are present, and add re-exports: `pub use library_list::LibraryListView;`, `pub use detail::DetailView;`, `pub use add_piece::AddPieceForm;`, `pub use add_exercise::AddExerciseForm;`, `pub use edit_piece::EditPieceForm;`, `pub use edit_exercise::EditExerciseForm;`
- [X] T027 [US2] Remove all moved view components from `crates/intrada-web/src/main.rs`. The file should now contain only `main()` and the `App` component
- [X] T028 [US2] Verify `cargo check -p intrada-web` compiles and `cargo clippy -p intrada-web -- -D warnings` passes with zero warnings
- [X] T029 [US2] Verify no file in `crates/intrada-web/src/` exceeds 300 lines using `find crates/intrada-web/src -name '*.rs' -exec wc -l {} + | sort -rn`

**Checkpoint**: All six views are in their own files under `views/`. `main.rs` contains only the entry point and `App` component. Every `.rs` file is under 300 lines.

---

## Phase 5: User Story 3 — Isolate App Component (Priority: P2)

**Goal**: Extract the `App` root component into `app.rs`, leaving `main.rs` as a minimal entry point.

**Independent Test**: `cargo check -p intrada-web` compiles. `cargo test` passes. `main.rs` is under 15 lines. `app.rs` contains only the root `App` component.

### Implementation for User Story 3

- [X] T030 [US3] Create `crates/intrada-web/src/app.rs` — move the `App` component (lines 344-466) from `main.rs`. Add imports: `use leptos::prelude::*;`, `use std::rc::Rc;`, `use std::cell::RefCell;`, `use crux_core::Core;`, `use send_wrapper::SendWrapper;`, `use intrada_core::{Event, Intrada, ViewModel};`, `use crate::types::{ViewState, SharedCore};`, `use crate::views::*;`, `use crate::data::create_stub_data;`, `use crate::core_bridge::process_effects;`. Make the component function `pub`
- [X] T031 [US3] Reduce `crates/intrada-web/src/main.rs` to minimal entry point: only `mod` declarations, `use app::App;`, and the `main()` function (console_error_panic_hook + mount_to_body). Remove all remaining component code and unused imports
- [X] T032 [US3] Verify `cargo check -p intrada-web` compiles and `cargo clippy -p intrada-web -- -D warnings` passes with zero warnings
- [X] T033 [US3] Verify `crates/intrada-web/src/main.rs` is under 20 lines (module declarations + entry point only)

**Checkpoint**: `main.rs` is a minimal entry point. `app.rs` contains the root component. All non-visual logic is in dedicated modules. The full module tree is: `main.rs` → `app.rs` → `views/*` → `components/*` with shared modules (`types`, `helpers`, `validation`, `data`, `core_bridge`).

---

## Phase 6: User Story 4 — Verify Conventions & Structure (Priority: P2)

**Goal**: Validate that the final structure follows discoverable conventions and all success criteria are met.

**Independent Test**: File listing matches plan.md target structure. Module naming follows consistent `snake_case` convention. All files have clear, purpose-indicating names.

### Implementation for User Story 4

- [X] T034 [US4] Verify file structure matches plan.md target by running `find crates/intrada-web/src -name '*.rs' | sort` and comparing against the 17-file target layout
- [X] T035 [US4] Verify module re-exports are explicit (not wildcard `pub use *`) in `crates/intrada-web/src/components/mod.rs` and `crates/intrada-web/src/views/mod.rs`
- [X] T036 [US4] Verify dependency direction is unidirectional: no view imports another view; no component imports a view; views import from components but not vice versa. Check with `grep -r "use crate::views" crates/intrada-web/src/components/` (should return nothing) and `grep -rn "use crate::views::" crates/intrada-web/src/views/` (should only show mod.rs re-exports)
- [X] T037 [US4] Remove all leftover phase comments (e.g., `// Phase 1 —`, `// Phase 2 —`) from all files in `crates/intrada-web/src/` — these were development markers from feature 004 and should not remain in the refactored codebase

**Checkpoint**: File structure matches plan exactly. Naming conventions are consistent. Dependency direction is clean (DAG, no cycles). No leftover development comments.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Full workspace validation and compliance checks.

- [X] T038 [P] Run `cargo test --workspace` to verify all 82+ existing tests pass without modification (SC-003 / FR-009)
- [X] T039 [P] Run `cargo clippy -- -D warnings` across entire workspace — zero warnings (SC-007)
- [X] T040 Run `cargo build --target wasm32-unknown-unknown -p intrada-web` to verify WASM compilation (SC-004 / FR-010)
- [X] T041 Verify no file exceeds 300 lines: `find crates/intrada-web/src -name '*.rs' -exec wc -l {} + | sort -rn` (SC-001)
- [X] T042 Verify total line count overhead is under 10%: total across all files must be under 2,097 lines (SC-008). Run `find crates/intrada-web/src -name '*.rs' -exec cat {} + | wc -l`
- [X] T043 Verify file count matches plan: exactly 17 `.rs` files in `crates/intrada-web/src/` (including subdirectories)
- [X] T044 Run quickstart.md Scenario 8 runtime smoke test: `trunk serve` in `crates/intrada-web/` and verify all 9 user flows in browser (list, detail, edit, back, add piece, add exercise, valid submit, invalid submit, delete with confirm)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 (module skeleton must exist). BLOCKS all user stories
- **US1 — Shared Components (Phase 3)**: Depends on Phase 2 (shared modules must be extracted first)
- **US2 — Views (Phase 4)**: Depends on Phase 3 (shared components must be in `components/` so views can import them)
- **US3 — App Component (Phase 5)**: Depends on Phase 4 (all views must be extracted before App can import from `views/`)
- **US4 — Conventions (Phase 6)**: Depends on Phase 5 (all code must be in final locations)
- **Polish (Phase 7)**: Depends on Phase 6 (all structural work complete)

### Why Sequential (Not Parallel Stories)

Unlike feature development where user stories are independent, this is a refactoring of a single monolithic file. Each phase removes code from `main.rs` and places it elsewhere. The phases must execute sequentially because:

1. **Phase 2** extracts non-visual logic that **Phase 3+** modules need to import
2. **Phase 3** extracts shared components that **Phase 4** views need to import
3. **Phase 4** extracts views that **Phase 5** `app.rs` needs to import
4. **Phase 5** extracts the App component, leaving `main.rs` minimal for **Phase 6** verification

### Within Each Phase

- Tasks marked [P] within a phase CAN run in parallel (they target different files)
- Verification tasks (T007, T013-T014, T019, T028-T029, T032-T033) must run after all other tasks in their phase

### Parallel Opportunities Per Phase

**Phase 2**: T008, T009, T010, T011 can all run in parallel (four independent module files)
**Phase 3**: T015, T016 can run in parallel (two independent component files)
**Phase 4**: T020, T021, T022, T023, T024, T025 can all run in parallel (six independent view files)
**Phase 7**: T038, T039 can run in parallel

---

## Implementation Strategy

### MVP First (Phase 1-3)

1. Complete Phase 1: Module skeleton
2. Complete Phase 2: Extract non-visual logic (helpers, validation, data, core_bridge, types)
3. Complete Phase 3: Extract shared components (FormFieldError, LibraryItemCard)
4. **VALIDATE**: `cargo check` + `cargo clippy` pass. Shared components importable from `crate::components::*`

### Incremental Delivery

1. Phase 1 + 2 → Foundation ready (non-visual logic extracted)
2. Phase 3 → Shared components extracted → Validate
3. Phase 4 → All views extracted → Validate (biggest phase — 6 parallel file extractions)
4. Phase 5 → App component extracted → Validate (main.rs now minimal)
5. Phase 6 → Conventions verified → Validate (structure matches plan)
6. Phase 7 → Full workspace validation → Complete

---

## Notes

- This is a pure **move + add pub + add imports** refactoring — no logic changes
- All code being moved already compiles and passes tests in `main.rs`
- The primary risk is incorrect imports or missing `pub` visibility markers
- If any phase fails to compile, the fix is almost certainly a missing `use` import or missing `pub` keyword
- [P] tasks within a phase target different files and have no interdependencies
- Line numbers reference the current `main.rs` state — they may shift slightly as earlier phases remove code; use function/component names as the primary identifier
- `cargo clippy` is run per-phase to catch issues early rather than accumulating them
- Commit after each phase checkpoint for clean git history and easy rollback
