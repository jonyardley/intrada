# Phase 0 Research: UI Primitive Components

**Feature**: 006-ui-primitives
**Date**: 2026-02-14
**Status**: Complete — no NEEDS CLARIFICATION items in Technical Context

## Research Tasks

### R1: Leptos Component Pattern for UI Primitives

**Decision**: Use standard `#[component]` functions with typed props. Children passed via Leptos `Children` type for slot-based components (Button, Card). No generic `class` override prop.

**Rationale**: This matches the existing `FormFieldError` and `LibraryItemCard` patterns established in feature 005. Leptos `Children` is the idiomatic way to pass slot content. Typed props enforce consistency and prevent callers from overriding styles.

**Alternatives Considered**:
- Generic `class` prop for style overrides — rejected per spec assumption (FR-013 requires visual consistency; overrides undermine that)
- Macro-based component generation — rejected as over-engineered for 10 simple wrappers

### R2: Button Variant Modelling

**Decision**: Use a Rust enum `ButtonVariant { Primary, Secondary, Danger }` passed as a required prop. Each variant maps to a fixed set of Tailwind classes.

**Rationale**: Enums are type-safe and exhaustive — the compiler catches missing variants. This is more robust than a string-based variant prop.

**Alternatives Considered**:
- String prop (`"primary"`, `"secondary"`, `"danger"`) — rejected because typos are silent failures
- Separate components (`PrimaryButton`, `SecondaryButton`) — rejected because it increases the component count without benefit

### R3: Form Field Signal Integration

**Decision**: TextField and TextArea accept `RwSignal<String>` for value and `RwSignal<HashMap<String, String>>` for errors, matching the existing pattern in all four form views.

**Rationale**: The four form views (add_piece, add_exercise, edit_piece, edit_exercise) all use identical signal patterns. Accepting the same signal types means zero refactoring of the form logic — only the view markup changes.

**Alternatives Considered**:
- Getter/setter callback pair — rejected because it adds indirection without benefit; all callers already use RwSignal
- Uncontrolled inputs with ref — rejected because it breaks the existing controlled-input validation pattern

### R4: Component File Organisation

**Decision**: All 10 new components go into the existing `components/` directory as flat sibling files (e.g., `components/button.rs`, `components/card.rs`). No subdirectory nesting.

**Rationale**: The total component count (12 including the 2 existing) does not warrant subdirectory organisation. This matches the flat `views/` structure. All components are re-exported from `components/mod.rs`.

**Alternatives Considered**:
- Subdirectory grouping (e.g., `components/form/`, `components/layout/`) — rejected because 12 files is manageable as a flat list; nesting adds import complexity
- Separate `primitives/` directory — rejected because it splits closely-related components across two locations

### R5: AppHeader and AppFooter Props

**Decision**: Both components take zero props. They render static content (app name, tagline, version badge, footer text).

**Rationale**: The header and footer content is application-level static content. Parameterising it would add complexity without benefit — there is only one header and one footer in the entire application.

**Alternatives Considered**:
- Parameterised version string prop — rejected because the version is a constant; changing it means editing the component, which is acceptable for a single-app context

## Summary

All research items are resolved. No NEEDS CLARIFICATION markers remain. The component patterns are well-established by the existing codebase (feature 005's `FormFieldError` and `LibraryItemCard`), so the primary decisions involve prop design and file organisation rather than novel technical challenges.
