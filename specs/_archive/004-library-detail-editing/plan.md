# Implementation Plan: Library Add, Detail View & Editing

**Branch**: `004-library-detail-editing` | **Date**: 2026-02-14 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/004-library-detail-editing/spec.md`

## Summary

Add full CRUD UI to the Intrada web application: add piece/exercise forms, item detail view, edit form with pre-populated values, and delete with confirmation. All views use a full-page pattern managed through application state. The Crux core already handles all events (Add, Update, Delete for both pieces and exercises); this feature adds the web shell UI components that dispatch those events and display their results.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition)
**Primary Dependencies**: leptos 0.8.x (CSR), crux_core 0.17.0-rc2, tailwindcss v4 (standalone CLI), trunk 0.21.x, console_error_panic_hook, wasm-bindgen, ulid, chrono
**Storage**: N/A (in-memory stub data; no persistence)
**Testing**: cargo test (82+ existing tests in core + CLI; web crate has no unit tests — UI verified via trunk build + manual testing)
**Target Platform**: WASM (wasm32-unknown-unknown), browser CSR
**Project Type**: Workspace with shared core — `crates/intrada-core` (pure logic), `crates/intrada-cli` (CLI shell), `crates/intrada-web` (web shell)
**Performance Goals**: Instant UI response (<100ms) for all actions — single-user, in-memory, no network
**Constraints**: No browser persistence, no URL routing, all state resets on page reload
**Scale/Scope**: Single-user, <100 items typical (stub data), 4 views (list, detail, add form, edit form)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | PASS | Single-responsibility Leptos components, self-documenting Crux event flow, no dead code |
| II. Testing Standards | PASS | Core logic already tested (82+ tests cover add/update/delete/validation). Web components verified via trunk build + WASM CI job. No new core logic introduced. |
| III. UX Consistency | PASS | Forms follow consistent pattern (same field layout, validation display, cancel/submit actions). Design tokens from Tailwind v4, consistent with MVP styling. ARIA roles on all interactive elements. |
| IV. Performance | PASS | All operations are in-memory, single-user. No network, no async, no loading states needed. |

No violations. No Complexity Tracking entries needed.

## Project Structure

### Documentation (this feature)

```text
specs/004-library-detail-editing/
├── plan.md              # This file
├── research.md          # Phase 0 output (minimal — no unknowns)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (Crux events, not REST)
│   └── events.md
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── intrada-core/        # Pure Crux core (UNCHANGED — all events already exist)
│   └── src/
│       ├── app.rs       # Event, Effect, Intrada App — already has Add/Update/Delete
│       ├── model.rs     # Model, ViewModel, LibraryItemView — already complete
│       ├── domain/
│       │   ├── piece.rs     # PieceEvent handlers — already complete
│       │   ├── exercise.rs  # ExerciseEvent handlers — already complete
│       │   └── types.rs     # CreatePiece, UpdatePiece, etc. — already complete
│       ├── validation.rs    # All validation rules — already complete
│       └── lib.rs           # Public exports — may need new exports
│
└── intrada-web/         # Leptos web shell (PRIMARY CHANGES HERE)
    └── src/
        └── main.rs      # App component + new view components
```

**Structure Decision**: All changes are in the web shell (`crates/intrada-web/src/main.rs`). The Crux core already has complete event handling, validation, and domain logic. No core changes are needed — view state is managed shell-side via a Leptos `RwSignal<ViewState>` enum (see Design Decision #1). No new crates or files needed — Leptos components are defined in `main.rs` (consistent with MVP pattern; extraction to modules deferred until file exceeds ~600 lines).

## Design Decisions

### 1. View State Management

The Crux `ViewModel` needs to communicate which view to display. Two approaches:

**Chosen: Shell-side view state (RwSignal enum)**
- Add a `ViewState` enum in the web shell: `List`, `Detail(String)`, `AddPiece`, `AddExercise`, `EditPiece(String)`, `EditExercise(String)`
- Managed as a Leptos `RwSignal<ViewState>` alongside the existing `RwSignal<ViewModel>`
- The `App` component matches on `ViewState` to render the appropriate view

**Rejected: Core-side view state**
- Would require adding navigation events to the Crux core (e.g., `Event::Navigate`)
- Navigation is a shell concern, not a core concern — the Crux philosophy keeps I/O and UI navigation in the shell
- Would bloat the core with web-specific state that the CLI doesn't need

### 2. Form State Management

**Chosen: Shell-side form signals**
- Each form field is a Leptos `RwSignal<String>` (or `RwSignal<Option<String>>`)
- Form validation runs client-side before dispatching the Crux event
- The Crux core's validation runs again on event processing (belt-and-suspenders)
- Validation errors from the core are displayed via `view_model.error`

**Rejected: Core-side form state**
- Would require adding form-specific events to the core (field change, focus, etc.)
- Forms are UI concerns; the core should only see the final validated submission

### 3. Component Organization

**Chosen: All components in main.rs**
- Consistent with the MVP pattern
- Single file is manageable at the expected ~500-600 lines
- Components: `App`, `LibraryItemCard` (existing), `DetailView`, `AddPieceForm`, `AddExerciseForm`, `EditPieceForm`, `EditExerciseForm`, `FormFieldError` (inline delete confirmation is part of `DetailView`, not a separate component)

**Rejected: Multi-file module structure**
- Premature for 4-5 new components
- Would add complexity (module declarations, re-exports)
- Can be refactored later if main.rs exceeds ~600 lines

### 4. Form Reuse Strategy

**Chosen: Separate add/edit components with shared field layout**
- `AddPieceForm` and `EditPieceForm` are distinct components
- They share the same field layout but differ in: initial values (empty vs pre-populated), submit handler (Add vs Update event), title ("Add Piece" vs "Edit Piece")
- A helper function `piece_form_fields()` could extract the shared field markup
- Similarly for exercises

**Rejected: Single generic form component**
- Type-level generics in Leptos components are complex
- The difference between add/edit is small enough that mild duplication is clearer than abstraction

### 5. Delete Confirmation

**Chosen: Inline confirmation banner in the detail view**
- When "Delete" is clicked, a confirmation banner appears within the detail view: "Are you sure you want to delete [title]? [Confirm] [Cancel]"
- No browser `window.confirm()` or modal overlay needed
- Consistent with the full-page view pattern

### 6. Tag Input

**Chosen: Comma-separated text field**
- Single `<input>` field where tags are entered as comma-separated values (e.g., "classical, piano, romantic")
- On submit, split by comma, trim whitespace, filter empty strings
- Display existing tags as comma-separated text in the edit form
- Per spec assumption: richer tag input deferred

## Complexity Tracking

> No violations — table intentionally empty.
