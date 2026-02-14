# Implementation Plan: UI Primitive Components

**Branch**: `006-ui-primitives` | **Date**: 2026-02-14 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/006-ui-primitives/spec.md`

## Summary

Extract duplicated UI markup from Intrada web views into a library of reusable primitive components. This is a pure refactoring — no new functionality, no visual changes. The existing `components/` directory gains approximately 10 new component files (Button, TextField, TextArea, Card, TypeBadge, PageHeading, BackLink, FieldLabel, AppHeader, AppFooter). Every view file is updated to use these shared components, eliminating 60+ instances of duplicated CSS class strings.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition)
**Primary Dependencies**: leptos 0.7 (CSR), crux_core 0.17.0-rc2 (workspace), send_wrapper 0.6, wasm-bindgen, console_error_panic_hook, tailwindcss v4 (standalone CLI)
**Storage**: N/A (stub data in-memory; no persistence changes)
**Testing**: `cargo test` (workspace-level, 82 existing tests), `cargo clippy`, WASM build verification
**Target Platform**: WASM (browser via trunk)
**Project Type**: Rust workspace — web shell crate (`intrada-web`)
**Performance Goals**: N/A (pure refactoring; no runtime behaviour change)
**Constraints**: Each file under 300 lines; total line overhead under 10% of current; zero clippy warnings
**Scale/Scope**: 10 new component files, 6 view files updated, 2 root files updated (app.rs, components/mod.rs)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Code Quality — Single Responsibility | ✅ PASS | Each new component has exactly one visual purpose |
| I. Code Quality — Explicit over Implicit | ✅ PASS | Components use explicit typed props; no generic className override |
| I. Code Quality — No Dead Code | ✅ PASS | All replaced inline markup is removed from views |
| I. Code Quality — Consistent Style | ✅ PASS | All components follow the same Leptos `#[component]` pattern |
| I. Code Quality — Clarity over Cleverness | ✅ PASS | Simple wrapper components with no complex logic |
| II. Testing — Coverage | ✅ PASS | Existing 82 tests maintained (SC-003); no new logic to test |
| II. Testing — Contract Tests | ✅ N/A | No API boundaries changed |
| III. UX Consistency — Design System | ✅ PASS | This feature creates the design system primitive layer |
| III. UX Consistency — Interaction Patterns | ✅ PASS | Button variants enforce consistent interaction patterns |
| III. UX Consistency — Accessibility | ✅ PASS | Existing ARIA attributes preserved within components |
| IV. Performance | ✅ PASS | No runtime changes; pure code-move refactoring |

No violations. No Complexity Tracking entries needed.

## Project Structure

### Documentation (this feature)

```text
specs/006-ui-primitives/
├── plan.md              # This file
├── research.md          # Phase 0: component pattern research
├── quickstart.md        # Phase 1: verification scenarios
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created by /speckit.plan)
```

Note: No `data-model.md` or `contracts/` needed — this feature involves no data changes or API contracts.

### Source Code (repository root)

```text
crates/intrada-web/src/
├── main.rs                         # Entry point (unchanged)
├── app.rs                          # MODIFIED: uses AppHeader, AppFooter
├── types.rs                        # Unchanged
├── helpers.rs                      # Unchanged
├── validation.rs                   # Unchanged
├── data.rs                         # Unchanged
├── core_bridge.rs                  # Unchanged
├── components/
│   ├── mod.rs                      # MODIFIED: new pub mod + pub use declarations
│   ├── form_field_error.rs         # Unchanged (existing)
│   ├── library_item_card.rs        # MODIFIED: uses TypeBadge
│   ├── button.rs                   # NEW: Button component (primary/secondary/danger)
│   ├── text_field.rs               # NEW: TextField (label + input + error)
│   ├── text_area.rs                # NEW: TextArea (label + textarea + error)
│   ├── card.rs                     # NEW: Card container
│   ├── type_badge.rs               # NEW: TypeBadge (piece/exercise)
│   ├── page_heading.rs             # NEW: PageHeading
│   ├── back_link.rs                # NEW: BackLink navigation
│   ├── field_label.rs              # NEW: FieldLabel (definition-term style)
│   ├── app_header.rs               # NEW: AppHeader
│   └── app_footer.rs               # NEW: AppFooter
└── views/
    ├── mod.rs                      # Unchanged
    ├── library_list.rs             # MODIFIED: uses Button
    ├── detail.rs                   # MODIFIED: uses Button, Card, TypeBadge, PageHeading, BackLink, FieldLabel
    ├── add_piece.rs                # MODIFIED: uses Button, TextField, TextArea, PageHeading, BackLink, Card
    ├── add_exercise.rs             # MODIFIED: uses Button, TextField, TextArea, PageHeading, BackLink, Card
    ├── edit_piece.rs               # MODIFIED: uses Button, TextField, TextArea, PageHeading, BackLink, Card
    └── edit_exercise.rs            # MODIFIED: uses Button, TextField, TextArea, PageHeading, BackLink, Card
```

**Structure Decision**: All new components go into the existing `components/` directory as flat sibling files. No subdirectory nesting — the component count (12 total including 2 existing) does not warrant it. This matches the flat `views/` structure established in feature 005.

## Component API Design

### Button

```
Props: variant (enum: Primary | Secondary | Danger | DangerOutline), on_click (callback), button_type (optional: "submit" | "button"), children (slot)
```

The Button component renders a `<button>` element with the correct Tailwind classes for the given variant. It passes through the `on:click` handler. The `button_type` prop defaults to `"button"` — the consumer must explicitly set `"submit"` for form submit buttons. The `Danger` variant renders a solid red button (bg-red-600, white text) for irreversible confirmations. The `DangerOutline` variant renders an outlined red button (white bg, red text, red border) for initial destructive action triggers like the "Delete" button.

### TextField

```
Props: id (string), label (string), value (RwSignal<String>), required (bool), placeholder (optional string), field_name (string), errors (RwSignal<HashMap<String, String>>), input_type (optional: "text" | "number", default "text")
```

Wraps a `<label>`, `<input>`, and `<FormFieldError>`. The `input_type` prop supports `"number"` for the BPM field.

### TextArea

```
Props: id (string), label (string), value (RwSignal<String>), rows (optional u32, default 3), field_name (string), errors (RwSignal<HashMap<String, String>>)
```

Same pattern as TextField but renders a `<textarea>` instead of `<input>`.

### Card

```
Props: children (slot)
```

Renders a `<div>` with the standard card classes: `bg-white rounded-xl shadow-sm border border-slate-200 p-6`. Content is passed as children.

### TypeBadge

```
Props: item_type (string)
```

Renders a `<span>` with the correct badge classes: violet for "piece", emerald for "exercise", grey for unknown types.

### PageHeading

```
Props: text (string)
```

Renders an `<h2>` with the standard heading classes: `text-2xl font-bold text-slate-900 mb-6`.

### BackLink

```
Props: label (string), on_click (callback)
```

Renders a `<button>` styled as a back-navigation link with the left arrow icon.

### FieldLabel

```
Props: text (string)
```

Renders a `<dt>` with the standard definition-term label classes: `text-xs font-medium text-slate-400 uppercase tracking-wider`.

### AppHeader

```
Props: (none)
```

Renders the full `<header>` block with application name, tagline, and version badge.

### AppFooter

```
Props: (none)
```

Renders the full `<footer>` block with informational text.

## Execution Strategy

This is a pure refactoring with the same sequential dependency pattern as feature 005:

1. **Create new component files first** (all independent — can be done in parallel)
2. **Update components/mod.rs** with new declarations and re-exports
3. **Update view files** to use new components (each view is independent)
4. **Update app.rs** to use AppHeader/AppFooter
5. **Verify** compilation, clippy, WASM build, and existing tests

The key risk is the same as feature 005: incorrect imports or missing `pub` visibility markers. The fix is always a missing `use` statement or `pub` keyword.
