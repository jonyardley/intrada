# Research: URL Routing for Web App Views

**Feature Branch**: `008-url-routing`
**Date**: 2026-02-14

## R1: Leptos Router Crate Selection

**Decision**: Use `leptos_router 0.8.x` (separate crate from leptos monorepo)

**Rationale**: `leptos_router` is the official, first-party routing solution for Leptos. Version 0.8.x is compatible with the project's existing `leptos 0.8` dependency. No additional feature flags are needed for CSR mode — the router inherits CSR/SSR mode from the `leptos` crate's feature configuration.

**Alternatives Considered**:
- Custom signal-based routing (current approach): Rejected because it cannot integrate with the browser History API or support deep linking without reimplementing what leptos_router already provides
- Third-party routing crates: None exist with meaningful adoption in the Leptos ecosystem; leptos_router is the canonical choice

**Dependency**: `leptos_router = { version = "0.8" }` — resolves to 0.8.11+ (latest as of 2026-02-14)

## R2: URL Path Design

**Decision**: Map existing ViewState variants to descriptive URL paths

| ViewState Variant | URL Path | Route Parameter |
|---|---|---|
| `List` | `/` | None |
| `Detail(id)` | `/library/:id` | `:id` (ULID string) |
| `AddPiece` | `/pieces/new` | None |
| `AddExercise` | `/exercises/new` | None |
| `EditPiece(id)` | `/pieces/:id/edit` | `:id` (ULID string) |
| `EditExercise(id)` | `/exercises/:id/edit` | `:id` (ULID string) |
| (not found) | `/*any` | Wildcard |

**Rationale**: Paths follow REST-like conventions (`/resource/new`, `/resource/:id/edit`). The library list is at the root since it is the primary view. Detail uses `/library/:id` rather than `/pieces/:id` because detail handles both pieces and exercises — the item type is resolved at render time from the ViewModel, not from the URL path.

**Alternatives Considered**:
- `/pieces/:id` and `/exercises/:id` as separate detail routes: Rejected because the existing DetailView handles both types via a single ID lookup. Splitting would require duplicating the detail component or adding unnecessary routing complexity. The core domain model already uses a unified item list.
- Hash-based routing (`#/path`): Rejected — leptos_router 0.8 uses History API only (no built-in HashRouter), and History API is the modern standard for SPAs.

## R3: Route Parameter Strategy

**Decision**: Use `use_params_map()` (untyped) for extracting route parameters

**Rationale**: The app uses ULID strings as IDs, which are simple string values. The untyped approach avoids the boilerplate of deriving `Params` structs with `Option<T>` fields (required on stable Rust). Since all route parameters are single string IDs, `params.read().get("id")` is direct and clear.

**Alternatives Considered**:
- Typed params with `#[derive(Params)]`: Adds a struct per parameterised route. Useful when routes have multiple typed parameters, but overkill for single-string IDs. May be adopted later if query parameters or multiple route params are added.

## R4: Navigation Pattern (Links vs Programmatic)

**Decision**: Use `<A>` component for user-initiated navigation; `use_navigate()` with `replace: true` for post-form-submission redirects

**Rationale**:
- `<A>` component produces semantic `<a>` elements with `href`, preserving accessibility (keyboard navigation, screen readers, right-click "open in new tab")
- `<A>` automatically sets `aria-current="page"` on active routes
- `use_navigate()` is needed for post-submission redirects where `replace: true` prevents the back button from returning to the submitted form (FR-011)
- The leptos_router documentation explicitly recommends `<A>` over programmatic navigation for user-clickable elements

**Alternatives Considered**:
- Programmatic navigation everywhere: Rejected because it loses `<a>` semantics, hurts accessibility (no href for screen readers), and prevents right-click/middle-click to open in new tab

## R5: ViewState Removal Strategy

**Decision**: Remove the `ViewState` enum and `RwSignal<ViewState>` entirely; let the router own all navigation state

**Rationale**: Keeping ViewState alongside the router would create dual-state management (signal + URL), which violates the single source of truth principle and risks synchronisation bugs. The router's URL IS the navigation state. Each view component receives its data from the ViewModel signal and its identity from route parameters.

**Alternatives Considered**:
- Keep ViewState synchronised with URL: Rejected — adds complexity for zero benefit. If the router drives navigation, ViewState becomes redundant state that must be kept in sync (a maintenance hazard per Constitution I: Code Quality — "No Dead Code", "Single Responsibility")

## R6: Trunk and Deployment Configuration

**Decision**: No changes needed to Trunk.toml or index.html

**Rationale**: Trunk's dev server automatically serves index.html for all paths (SPA fallback). The leptos_router uses the History API which works with the existing setup. Production deployment will need server-side SPA fallback configuration (e.g., nginx `try_files`), but that is out of scope for this feature (documented in spec assumptions).

## R7: Form Submission and History Replacement

**Decision**: After form submission (add/edit), navigate with `NavigateOptions { replace: true }` to prevent back-button return to the completed form

**Rationale**: FR-011 requires that after form submission, the browser back button does NOT return to the submitted form. Using `replace: true` replaces the form's history entry with the post-submission destination, so the back button skips over it. This matches standard web application behavior (POST-redirect-GET pattern adapted for SPA).

**Implementation pattern**:
```
Form view (URL: /pieces/new)
  → User submits
  → navigate("/", NavigateOptions { replace: true, ..Default::default() })
  → History entry for /pieces/new is replaced with /
  → Back button goes to whatever was before /pieces/new
```

## R8: Not-Found Route Handling

**Decision**: Use the `Routes` `fallback` prop for 404 handling, with a dedicated `NotFoundView` component

**Rationale**: leptos_router 0.8 requires a `fallback` prop on `<Routes>`. This is the natural place to render a not-found view. A dedicated component (rather than inline closure) keeps the code clean and testable, and allows the 404 page to include a link back to the library list (FR-008).

For missing item IDs (valid route structure but non-existent ID), the existing DetailView already handles this by navigating back to List when the item is not found in the ViewModel. This can be refined to show the not-found view instead.
