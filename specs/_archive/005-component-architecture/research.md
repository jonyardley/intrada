# Research: Web App Component Architecture

**Feature**: 005-component-architecture
**Date**: 2026-02-14

## Research Questions

### RQ-1: Leptos 0.7 CSR Multi-File Module Patterns

**Decision**: Use standard Rust module system with `mod.rs` re-exports for component organisation.

**Rationale**: Leptos 0.7 CSR components are regular Rust functions annotated with `#[component]`. They follow standard Rust module visibility and import rules. No framework-specific module system or configuration is required.

**Key Findings**:

1. **Component visibility**: Components need `pub fn` to be accessible across modules. The `#[component]` macro generates a Props struct automatically — no need to manually re-export it (handled by Leptos since 0.5).

2. **Signal passing**: `RwSignal<T>` implements `Copy`, so signals pass freely across module boundaries without ownership issues. This means parent components can create signals and pass them to child components in other files without cloning or wrapping.

3. **Required imports per file**: Each component file needs `use leptos::prelude::*;` and optionally `use leptos::ev;` for event types. The `SendWrapper` type and `SharedCore` alias need to be imported from the shared types module.

4. **Trunk/WASM compatibility**: Trunk handles multi-file Rust projects with zero configuration changes. The `index.html` and `Trunk.toml` remain unchanged. WASM compilation processes the entire crate as a unit regardless of file structure.

5. **SendWrapper pattern**: The `SendWrapper<Rc<RefCell<Core<Intrada>>>>` pattern works identically across module boundaries. It should be defined once in a shared types module and imported where needed.

**Alternatives Considered**:

- **Feature flags for conditional compilation**: Rejected — adds unnecessary complexity for a pure organisational refactor.
- **Workspace sub-crates per component**: Rejected — overkill for ~10 components. Standard modules within a single crate are sufficient and maintain fast compilation.
- **Dynamic component loading**: Rejected — not applicable to CSR WASM applications where the entire app is compiled to a single WASM binary.

### RQ-2: Component Architecture Pattern for Small-to-Medium Leptos Apps

**Decision**: Flat module structure with three layers — shared components, views, and helpers — rather than strict atomic design hierarchy.

**Rationale**: The application has ~10 components. A full atoms/molecules/organisms/templates/pages hierarchy (5 levels) for 10 components would create more indirection than value. A three-layer approach provides clear separation while keeping navigation simple.

**Key Findings**:

1. **Three-layer structure**:
   - `components/` — Shared, reusable building blocks (FormFieldError, LibraryItemCard, type badges, tag displays)
   - `views/` — Full-page views composed from shared components (LibraryListView, DetailView, AddPieceForm, etc.)
   - Root-level modules — Helpers, types, data, and the App root component

2. **Naming convention**: Files named after their primary component in `snake_case` matching Rust conventions. E.g., `form_field_error.rs` for `FormFieldError`, `detail_view.rs` for `DetailView`.

3. **Module re-exports**: `components/mod.rs` and `views/mod.rs` re-export all public items, allowing consumers to use `use crate::components::FormFieldError;` or `use crate::views::*;`.

4. **Shared state pattern**: The `SharedCore` type alias and `process_effects` function should live in a dedicated module (e.g., `core_bridge.rs`) since they're used by most views.

**Alternatives Considered**:

- **Strict atomic design (atoms/molecules/organisms/templates/pages)**: Rejected — 5 levels of hierarchy for 10 components creates unnecessary indirection. Would result in many folders with only 1-2 files each.
- **Feature-based grouping (by domain entity)**: Rejected — the web shell's views don't align cleanly to single domain entities (e.g., DetailView shows both pieces and exercises). Component-type grouping is more natural for this UI.
- **Single flat directory**: Rejected — while simpler, it doesn't provide the visual/helper/view separation the spec requires, and would make it harder to understand component relationships.

### RQ-3: File Size and Module Boundary Strategy

**Decision**: Split along natural boundaries — each `#[component]` function gets its own file; non-visual logic grouped by purpose.

**Rationale**: The current monolithic file has clear natural boundaries at each `#[component]` annotation. These boundaries already represent logical units with their own state, event handling, and view rendering.

**Key Findings**:

1. **Current component sizes** (approximate line counts):
   - `App` (root): ~115 lines (L350-466)
   - `FormFieldError`: ~15 lines (L468-483)
   - `LibraryListView`: ~165 lines (L485-649)
   - `LibraryItemCard`: ~95 lines (L651-744)
   - `DetailView`: ~215 lines (L746-959)
   - `AddPieceForm`: ~195 lines (L961-1154)
   - `AddExerciseForm`: ~215 lines (L1156-1372)
   - `EditPieceForm`: ~235 lines (L1374-1606)
   - `EditExerciseForm`: ~270 lines (L1608-1879)

2. **Non-visual logic sizes**:
   - Type definitions (ViewState, SharedCore): ~20 lines
   - Validation (piece + exercise): ~95 lines each (~190 total)
   - Parsing helpers: ~45 lines
   - Stub data + constants: ~55 lines
   - Effect processing: ~20 lines
   - Main entry point: ~5 lines

3. **Largest resulting file**: `EditExerciseForm` at ~270 lines — well under the 300-line SC-001 limit.

4. **Total overhead estimate**: Module declarations, `mod.rs` files, and additional `use` imports will add approximately 80-120 lines (~5-6% overhead), well within the SC-008 10% limit.

### RQ-4: Circular Dependency Prevention

**Decision**: Use a unidirectional dependency graph: `main` → `app` → `views` → `components`, with shared types/helpers accessible to all.

**Rationale**: Leptos components communicate via signals and callbacks (closures), not by importing each other directly. View navigation is handled through ViewState changes, not direct component references.

**Key Findings**:

1. **No actual circular dependencies exist**: The current code uses `ViewState` enum variants to switch between views. The `App` component matches on `ViewState` and renders the appropriate view. Individual views don't import each other — they set `ViewState` via signals to trigger navigation.

2. **Dependency direction**: `App` imports all views. Views import shared components. No view imports another view. This creates a clean DAG (directed acyclic graph).

3. **Shared state**: `SharedCore` and signal types flow downward through component props. `process_effects` is called by views but defined in a shared module.
