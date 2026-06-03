# Implementation Plan: URL Routing for Web App Views

**Branch**: `008-url-routing` | **Date**: 2026-02-14 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/008-url-routing/spec.md`

## Summary

Add client-side URL routing to the Intrada web application so every existing view has a unique, bookmarkable URL with full browser history support. The implementation uses `leptos_router 0.8.x` to replace the current signal-based `ViewState` navigation with declarative route definitions, `<A>` link components for accessible navigation, and `use_navigate()` for programmatic post-submission redirects. A not-found view handles unrecognised URLs.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition)
**Primary Dependencies**: leptos 0.8.x (CSR), leptos_router 0.8.x, crux_core 0.17.0-rc2 (workspace), tailwindcss v4 (standalone CLI), trunk 0.21.x, console_error_panic_hook, wasm-bindgen, ulid, chrono, send_wrapper 0.6, getrandom 0.3
**Storage**: N/A (in-memory stub data; no persistence changes)
**Testing**: cargo test (workspace), cargo clippy, cargo fmt
**Target Platform**: WASM (browser, CSR-only SPA)
**Project Type**: Single workspace crate (intrada-web) with shared intrada-core
**Performance Goals**: WASM binary size must not exceed 120% of pre-routing baseline; route transitions must feel instant (no perceptible delay)
**Constraints**: CSR-only (no SSR); hosting environment must serve SPA fallback (trunk dev server does this automatically)
**Scale/Scope**: 7 routes (6 views + 1 not-found fallback), 6 existing view components refactored, 1 new not-found view

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Code Quality ✅ PASS

- **Clarity over cleverness**: Route definitions are declarative (`<Route path=path!("/") view=... />`), more readable than the current match block
- **Single Responsibility**: Router owns navigation state; views own rendering; forms own submission logic — cleaner separation than current dual signal/view coupling
- **Consistent Style**: leptos_router is the canonical Leptos routing solution; follows established patterns
- **No Dead Code**: `ViewState` enum and `RwSignal<ViewState>` will be fully removed, not left alongside the router
- **Explicit over Implicit**: Route paths are explicit string literals; navigation is visible at call sites via `<A href=...>` or `navigate(path, opts)`
- **Type Safety**: Route parameters extracted via `use_params_map()` returning `String` (same type safety as current `ViewState(String)` pattern)

### II. Testing Standards ✅ PASS

- **Test Coverage**: Existing 82 workspace tests (64 core + 18 CLI) are unaffected — routing is a web shell concern, not a core concern. New not-found view has clear testable behavior.
- **Test Independence**: No shared mutable state introduced; routing state is owned by the browser URL
- **Meaningful Assertions**: Tests verify navigation behavior (correct view renders for correct URL), not router internals
- **Contract Tests**: Route table is documented and verifiable by manual smoke testing per quickstart.md

### III. User Experience Consistency ✅ PASS

- **Design System Adherence**: No visual changes; existing Tailwind classes and component styles preserved
- **Interaction Patterns**: Navigation via `<A>` components produces standard `<a>` elements — consistent with web conventions (right-click, middle-click, keyboard nav all work)
- **Error Communication**: Not-found view shows user-friendly message with clear "Back to Library" link (FR-007, FR-008)
- **Loading States**: Route transitions are synchronous (no async data fetching) — instant view swaps, no loading spinners needed
- **Accessibility**: `<A>` component sets `aria-current="page"` on active routes; all existing ARIA attributes preserved (FR-012)
- **Progressive Enhancement**: N/A — WASM app requires JavaScript; no degradation path applies

### IV. Performance Requirements ✅ PASS

- **WASM Binary Size**: leptos_router adds routing logic but no heavy dependencies; monitored against 120% budget threshold
- **Route Transitions**: Client-side only, no network requests — transitions are sub-millisecond
- **Resource Limits**: No new allocations beyond route parameter strings; ViewState removal actually reduces signal overhead
- **Lazy Loading**: N/A — all views are small and already included in the WASM bundle
- **Caching Strategy**: N/A — no cacheable data in routing layer
- **Measurement**: Pre/post WASM binary size comparison (same approach as feature 007)

### Post-Design Re-check ✅ PASS

No constitution violations introduced by the design. The approach simplifies the codebase (removes ViewState) while adding standard web routing capabilities.

## Project Structure

### Documentation (this feature)

```text
specs/008-url-routing/
├── plan.md              # This file
├── research.md          # Leptos router research and decisions
├── quickstart.md        # Verification scenarios and route table
├── checklists/
│   └── requirements.md  # Specification quality checklist
└── tasks.md             # Task breakdown (created by /speckit.tasks)
```

### Source Code (repository root)

```text
crates/intrada-web/
├── Cargo.toml                    # ADD: leptos_router dependency
├── Trunk.toml                    # UNCHANGED
├── index.html                    # UNCHANGED
├── input.css                     # UNCHANGED
└── src/
    ├── main.rs                   # UNCHANGED
    ├── app.rs                    # MAJOR REFACTOR: Replace ViewState match with Router/Routes
    ├── types.rs                  # MODIFY: Remove ViewState enum (keep SharedCore)
    ├── core_bridge.rs            # UNCHANGED
    ├── data.rs                   # UNCHANGED
    ├── helpers.rs                # UNCHANGED
    ├── validation.rs             # UNCHANGED
    ├── components/
    │   ├── mod.rs                # UNCHANGED
    │   ├── app_header.rs         # UNCHANGED
    │   ├── app_footer.rs         # UNCHANGED
    │   ├── back_link.rs          # MODIFY: Replace callback with <A> link
    │   ├── library_item_card.rs  # MODIFY: Replace on_click callback with <A> wrapper
    │   ├── button.rs             # UNCHANGED
    │   ├── card.rs               # UNCHANGED
    │   ├── page_heading.rs       # UNCHANGED
    │   ├── field_label.rs        # UNCHANGED
    │   ├── form_field_error.rs   # UNCHANGED
    │   ├── text_field.rs         # UNCHANGED
    │   ├── text_area.rs          # UNCHANGED
    │   └── type_badge.rs         # UNCHANGED
    └── views/
        ├── mod.rs                # MODIFY: Export NotFoundView
        ├── library_list.rs       # MODIFY: Remove view_state prop, use <A> for add links
        ├── detail.rs             # MODIFY: Remove view_state prop, use route params + <A> links
        ├── add_piece.rs          # MODIFY: Remove view_state prop, use navigate() for redirects
        ├── add_exercise.rs       # MODIFY: Remove view_state prop, use navigate() for redirects
        ├── edit_piece.rs         # MODIFY: Remove view_state prop, use route params + navigate()
        ├── edit_exercise.rs      # MODIFY: Remove view_state prop, use route params + navigate()
        └── not_found.rs          # NEW: 404 view with link to library

crates/intrada-core/              # UNCHANGED — routing is a web shell concern
crates/intrada-cli/               # UNCHANGED
```

**Structure Decision**: Existing single-crate web structure is preserved. All changes are within `crates/intrada-web/`. No new crates, no structural changes. The routing refactor touches app.rs (major), types.rs (minor removal), 2 components, and all 6 view files, plus 1 new not-found view.

## Design Decisions

### Route Architecture

The `<Router>` wraps the entire app at the root level in `app.rs`. `<Routes>` with a `fallback` prop defines all route mappings. Each `<Route>` maps a path pattern to a view component.

```
Router
├── AppHeader (renders on all routes)
├── Routes (fallback → NotFoundView)
│   ├── Route "/" → LibraryListView
│   ├── Route "/library/:id" → DetailView
│   ├── Route "/pieces/new" → AddPieceForm
│   ├── Route "/exercises/new" → AddExerciseForm
│   ├── Route "/pieces/:id/edit" → EditPieceForm
│   └── Route "/exercises/:id/edit" → EditExerciseForm
└── AppFooter (renders on all routes)
```

### ViewState Removal

The `ViewState` enum is removed entirely. Navigation state is the URL itself. View components no longer receive a `view_state: RwSignal<ViewState>` prop. Instead:

- **Static links** (library list, add forms, back links) use `<A href="/path">` components
- **Dynamic links** (item detail, edit) use `<A href=format!("/library/{id}")>`
- **Post-form redirects** use `let navigate = use_navigate(); navigate("/path", NavigateOptions { replace: true, ..Default::default() })`
- **Route params** (item ID) extracted via `let params = use_params_map(); params.read().get("id")`

### Prop Signature Changes

All view components currently receive `view_state: RwSignal<ViewState>`. This prop is removed from every view. Components that need the item ID (DetailView, EditPieceForm, EditExerciseForm) extract it from route parameters instead of receiving it as a component prop.

| Component | Current Props | New Props |
|---|---|---|
| LibraryListView | view_model, view_state, core, sample_counter | view_model, core, sample_counter |
| DetailView | id, view_model, view_state, core | view_model, core |
| AddPieceForm | view_model, view_state, core | view_model, core |
| AddExerciseForm | view_model, view_state, core | view_model, core |
| EditPieceForm | id, view_model, view_state, core | view_model, core |
| EditExerciseForm | id, view_model, view_state, core | view_model, core |
| NotFoundView | (new) | (none) |

### Navigation Replacement Map

| Current Pattern | Replacement |
|---|---|
| `view_state.set(ViewState::List)` (cancel/back) | `<A href="/">` component |
| `view_state.set(ViewState::List)` (after form submit) | `navigate("/", NavigateOptions { replace: true, .. })` |
| `view_state.set(ViewState::Detail(id))` (after edit submit) | `navigate(&format!("/library/{id}"), NavigateOptions { replace: true, .. })` |
| `view_state.set(ViewState::Detail(id))` (card click) | `<A href=format!("/library/{id}")>` wrapping card content |
| `view_state.set(ViewState::AddPiece)` | `<A href="/pieces/new">` |
| `view_state.set(ViewState::AddExercise)` | `<A href="/exercises/new">` |
| `view_state.set(ViewState::EditPiece(id))` | `<A href=format!("/pieces/{id}/edit")>` |
| `view_state.set(ViewState::EditExercise(id))` | `<A href=format!("/exercises/{id}/edit")>` |
| `view_state.set(ViewState::List)` (item not found fallback) | `navigate("/", Default::default())` |

## Complexity Tracking

> No constitution violations. No complexity justifications needed.
