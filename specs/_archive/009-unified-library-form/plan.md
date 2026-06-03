# Implementation Plan: Unified Library Item Form

**Branch**: `009-unified-library-form` | **Date**: 2026-02-14 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/009-unified-library-form/spec.md`

## Summary

Replace the four separate add/edit form views (`add_piece.rs`, `add_exercise.rs`, `edit_piece.rs`, `edit_exercise.rs`) with two unified components: `AddLibraryItemForm` and `EditLibraryItemForm`. Each renders a tabbed interface (Piece / Exercise) that dynamically adjusts form fields, validation rules, and submission logic based on the active tab. The add form uses interactive tabs; the edit form uses display-only tabs with the item's type pre-selected. Routes are consolidated from 4 to 2, the "Add" dropdown in the library list is replaced with a single "Add Item" button, and the detail view's edit link is simplified. No core domain or data model changes are required.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition)
**Primary Dependencies**: leptos 0.7 (CSR), leptos_router 0.8, crux_core 0.17.0-rc2 (workspace), send_wrapper 0.6, tailwindcss v4 (standalone CLI), trunk 0.21.x
**Storage**: N/A (in-memory stub data; no persistence changes)
**Testing**: `cargo test` (82+ existing tests in intrada-core and intrada-cli; web shell has no automated tests — manual verification via trunk serve)
**Target Platform**: WASM (browser via trunk, CSR mode)
**Project Type**: Workspace — `crates/intrada-core/` (pure Crux core), `crates/intrada-web/` (Leptos web shell), `crates/intrada-cli/` (CLI shell)
**Performance Goals**: Tab switching must be instant (local signal update, no async); form render under 16ms
**Constraints**: Pure UI change — no modifications to intrada-core domain types, events, or model. All existing 82+ tests must continue to pass unchanged.
**Scale/Scope**: 6 route paths → 4 route paths; 4 form view files → 2 form view files; ~1,100 lines of form code consolidated

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality — Clarity over cleverness | ✅ PASS | Unified form reduces duplication; tab state is a simple signal enum |
| I. Code Quality — Single Responsibility | ✅ PASS | Each form component still has one purpose (add or edit); tab logic is an internal concern |
| I. Code Quality — Consistent Style | ✅ PASS | Follows existing component patterns (RwSignal, HashMap errors, TextField/TextArea) |
| I. Code Quality — No Dead Code | ✅ PASS | Old form files and unused routes will be removed |
| I. Code Quality — Explicit over Implicit | ✅ PASS | Tab state is an explicit enum signal; validation rules are explicit per-type |
| I. Code Quality — Type Safety | ✅ PASS | ItemType enum replaces stringly-typed tab state |
| II. Testing — Test Coverage | ✅ PASS | All 82+ core tests preserved; manual testing via quickstart scenarios |
| II. Testing — Contract Tests | ✅ PASS | Core domain API (CreatePiece, CreateExercise, etc.) unchanged |
| III. UX Consistency — Interaction Patterns | ✅ PASS | Tabs follow standard pattern (active/inactive visual states, keyboard navigation) |
| III. UX Consistency — Accessibility | ✅ PASS | Tabs use ARIA roles (tablist, tab, tabpanel), keyboard nav (Arrow keys, Enter/Space) |
| III. UX Consistency — Error Communication | ✅ PASS | Errors cleared on tab switch; same FormFieldError pattern |
| IV. Performance — Response Time | ✅ PASS | Tab switch is a local signal update (no network, no core event) |

**Gate result**: All principles pass. No violations. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/009-unified-library-form/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (minimal — no data changes)
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
  intrada-core/            # NO CHANGES — pure Crux core
    src/
      domain/
        types.rs           # CreatePiece, CreateExercise, UpdatePiece, UpdateExercise (unchanged)
        piece.rs           # PieceEvent (unchanged)
        exercise.rs        # ExerciseEvent (unchanged)
      model.rs             # ViewModel, LibraryItemView (unchanged)
      validation.rs        # Core validation (unchanged)

  intrada-web/             # ALL CHANGES HERE
    src/
      app.rs               # MODIFY — update routes (4 add/edit routes → 2)
      validation.rs        # MODIFY — replace two functions with one unified function
      views/
        mod.rs             # MODIFY — replace exports
        add_form.rs        # NEW — unified add form (replaces add_piece.rs + add_exercise.rs)
        edit_form.rs       # NEW — unified edit form (replaces edit_piece.rs + edit_exercise.rs)
        add_piece.rs       # DELETE
        add_exercise.rs    # DELETE
        edit_piece.rs      # DELETE
        edit_exercise.rs   # DELETE
        library_list.rs    # MODIFY — single "Add Item" button (replace dropdown)
        detail.rs          # MODIFY — unified edit URL
      components/
        mod.rs             # MODIFY — add TypeTabs export
        type_tabs.rs       # NEW — reusable tab bar component
      types.rs             # MODIFY — add ItemType enum
```

**Structure Decision**: All changes are confined to the `intrada-web` crate. The `intrada-core` crate and `intrada-cli` crate are completely untouched. The unified form components follow the existing pattern of one component per view file, with a new shared `TypeTabs` component extracted into the components directory.

## Architecture

### Tab State Design

The tab state is a simple Leptos signal holding an enum:

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Piece,
    Exercise,
}
```

- **Add form**: `active_tab: RwSignal<ItemType>` — initialized to `ItemType::Piece`, togglable via TypeTabs callback
- **Edit form**: `active_tab: ItemType` — set from the item's `item_type` field at load time, not a signal (display-only, immutable)

The `ItemType` enum is defined in `types.rs` alongside `SharedCore` for reuse across components.

### TypeTabs Component

A reusable component that renders the two-tab bar:

```rust
#[component]
pub fn TypeTabs(
    active: Signal<ItemType>,                              // Current active tab (read-only signal)
    #[prop(optional)] on_change: Option<Callback<ItemType>>,  // None = display-only
) -> impl IntoView
```

- When `on_change` is `Some`, tabs are interactive (clickable, keyboard-navigable with Left/Right arrow keys)
- When `on_change` is `None`, tabs are display-only (active tab shown as selected, inactive tab styled as disabled/muted)
- Uses `role="tablist"`, `role="tab"`, `aria-selected`, `aria-controls`, `tabindex`
- Active tab: indigo-600 bg, white text (matches Primary button style)
- Inactive tab (interactive): white bg, slate border, hover state
- Inactive tab (display-only): slate-100 bg, slate-400 text, no cursor pointer

### Unified Add Form Signal Layout

All field signals are created once and shared across both tab configurations:

| Signal | Type | Shared? | Notes |
|--------|------|---------|-------|
| `active_tab` | `RwSignal<ItemType>` | — | Controls which fields render / which validation applies |
| `title` | `RwSignal<String>` | ✅ Shared | Always visible, always required |
| `composer` | `RwSignal<String>` | ✅ Shared | Always visible; required for Piece, optional for Exercise |
| `category` | `RwSignal<String>` | Exercise-only | Hidden on Piece tab; value preserved in signal when hidden |
| `key_sig` | `RwSignal<String>` | ✅ Shared | Always visible |
| `tempo_marking` | `RwSignal<String>` | ✅ Shared | Always visible |
| `bpm` | `RwSignal<String>` | ✅ Shared | Always visible |
| `notes` | `RwSignal<String>` | ✅ Shared | Always visible |
| `tags_input` | `RwSignal<String>` | ✅ Shared | Always visible |
| `errors` | `RwSignal<HashMap<String, String>>` | ✅ Shared | Cleared on tab switch |

### Conditional Rendering Strategy

Rather than destroying and recreating fields, use conditional `view!` blocks:

- **Composer label**: Changes from "Composer *" (Piece) to "Composer" (Exercise) based on `active_tab`
- **Composer `required` prop**: `true` when Piece, `false` when Exercise
- **Category field**: Rendered only when `active_tab == ItemType::Exercise` — wrapping in a `Show` or `{move || if ... { Some(...) } else { None }}` block

The `category` signal retains its value even when hidden (signal persists, only the DOM element is conditionally rendered). On form submission, category is only included in the `CreateExercise` event, never in `CreatePiece`.

### Tab Switch Behavior

When the user switches tabs on the add form:

1. Update `active_tab` signal → triggers reactive re-render of conditional fields
2. Clear `errors` signal → `errors.set(HashMap::new())` — removes stale validation messages (FR-007)
3. Shared field signals (title, composer, key, tempo, bpm, notes, tags) are **untouched** — their values persist (FR-004)
4. Category signal is **untouched** — its value is preserved in memory even when hidden (FR-005)

### Validation Strategy

A unified validation function replaces the two existing ones:

```rust
pub fn validate_library_form(
    item_type: ItemType,
    title: &str,
    composer: &str,
    category: &str,
    notes: &str,
    bpm_str: &str,
    tempo_marking: &str,
    tags_str: &str,
) -> HashMap<String, String>
```

- When `item_type == Piece`: composer is required (empty → error), category is ignored
- When `item_type == Exercise`: composer is optional (empty → OK), category validated if present (max 100)
- Common validations (title required 1-500, notes max 5000, bpm 1-400, tempo_marking max 100, tags each max 100) are shared

The old `validate_piece_form` and `validate_exercise_form` functions are kept temporarily during incremental migration (Phases 2–5) and removed in the Polish phase once all callers have been migrated to `validate_library_form`.

### Submission Strategy

The submit handler inspects `active_tab.get()` to determine which core event to dispatch:

**Add form:**
- **Piece**: Build `CreatePiece { title, composer, key, tempo, notes, tags }` → `Event::Piece(PieceEvent::Add(...))`
- **Exercise**: Build `CreateExercise { title, composer: Option, category: Option, key, tempo, notes, tags }` → `Event::Exercise(ExerciseEvent::Add(...))`

**Edit form:**
- **Piece**: Build `UpdatePiece { title, composer, key, tempo, notes, tags }` → `Event::Piece(PieceEvent::Update { id, input })`
- **Exercise**: Build `UpdateExercise { title, composer, category, key, tempo, notes, tags }` → `Event::Exercise(ExerciseEvent::Update { id, input })`

The double-Option pattern for UpdatePiece/UpdateExercise is preserved: `Some(None)` clears a field, `Some(Some(value))` sets it.

### Route Changes

| Before | After | Notes |
|--------|-------|-------|
| `/pieces/new` | `/library/new` | Unified add form |
| `/exercises/new` | *(removed)* | Merged into `/library/new` |
| `/pieces/:id/edit` | `/library/:id/edit` | Unified edit form |
| `/exercises/:id/edit` | *(removed)* | Merged into `/library/:id/edit` |
| `/` | `/` | Unchanged |
| `/library/:id` | `/library/:id` | Unchanged |

Total: 6 routes → 4 routes (SC-005: 4 add/edit routes → 2 add/edit routes)

### Detail View Edit Link Change

Current: `if type == "piece" { "/pieces/{id}/edit" } else { "/exercises/{id}/edit" }`
After: `format!("/library/{}/edit", id)` — always the same, regardless of type.

### Library List Change

Current: Dropdown menu with "Piece" and "Exercise" links, toggle signal.
After: Single `<A href="/library/new">` button styled as Primary. Remove `show_add_menu` signal, dropdown `<div>`, and the two `<A>` links inside it.

### Edit Form — Composer Recovery

The existing edit_exercise.rs has a known limitation: when an exercise has both a composer and a category, the ViewModel's `subtitle` field contains the category (not the composer), and the composer value is not directly accessible from the ViewModel.

This is addressed by the `LibraryItemView` structure which has:
- `subtitle: String` — for pieces: composer; for exercises: `category.or(composer)`
- `category: Option<String>` — only for exercises

For the unified edit form, the composer pre-population logic:
- **Piece**: `composer = item.subtitle.clone()` (subtitle is always composer for pieces)
- **Exercise**: If `item.category.is_some()` → subtitle is category, so `composer = String::new()` (cannot recover original composer from ViewModel). Otherwise → `composer = item.subtitle.clone()`

This is the same behavior as the current edit_exercise.rs — no regression.

## Complexity Tracking

> No constitution violations to justify — all gates pass.

*No entries needed.*
