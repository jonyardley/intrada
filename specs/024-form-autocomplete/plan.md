# Implementation Plan: Form Autocomplete

**Branch**: `024-form-autocomplete` | **Date**: 2026-02-18 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/024-form-autocomplete/spec.md`

## Summary

Add autocomplete to tag and composer fields in library add/edit forms. Tags get a chip-based input with inline autocomplete from existing library tags; composers get a text input with suggestions from existing composer names. Both share a reusable `Autocomplete` dropdown component. Suggestions are derived from the already-loaded ViewModel — no new API endpoints or core changes needed. This is a pure shell (intrada-web) feature.

## Technical Context

**Language/Version**: Rust stable (1.89.0 in CI; workspace MSRV 1.75+, 2021 edition)
**Primary Dependencies**: leptos 0.8.x (CSR), leptos_router 0.8.x, web-sys (DOM events), wasm-bindgen
**Storage**: N/A (no new storage; reads from existing ViewModel populated by API)
**Testing**: cargo test (unit), wasm-bindgen-test (WASM), Playwright (E2E)
**Target Platform**: WASM (browser)
**Project Type**: Web application (Crux three-crate workspace)
**Performance Goals**: Suggestion dropdown appears within 100ms of typing; filtering operates on in-memory data only
**Constraints**: No new API endpoints; no changes to intrada-core; all logic in intrada-web shell
**Scale/Scope**: Library size up to ~10k items; suggestion lists typically <100 unique values

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Research Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | Reusable components with single responsibility; no dead code added |
| II. Testing Standards | ✅ Pass | Unit tests for filtering logic; WASM tests for component behaviour; E2E for user flows |
| III. UX Consistency | ✅ Pass | Follows existing form patterns (TextField, TypeTabs); consistent styling with Tailwind |
| IV. Performance | ✅ Pass | In-memory filtering only; no network requests; derived signals are efficient |
| V. Architecture Integrity | ✅ Pass | All changes in intrada-web (shell); core remains pure; no new effects/events |

### Post-Design Re-Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | Autocomplete is a single reusable component; TagInput composes it for multi-select |
| II. Testing Standards | ✅ Pass | Filtering logic testable in isolation; keyboard nav testable via WASM; full flows via E2E |
| III. UX Consistency | ✅ Pass | Chip pattern follows industry standard; ARIA combobox pattern for accessibility |
| IV. Performance | ✅ Pass | Derived signals recompute only when ViewModel changes; filtering is O(n) on small lists |
| V. Architecture Integrity | ✅ Pass | Zero changes to intrada-core or intrada-api; pure shell feature |

## Project Structure

### Documentation (this feature)

```text
specs/024-form-autocomplete/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 research decisions
├── data-model.md        # Transient data structures
├── quickstart.md        # Verification steps
├── contracts/           # Component interface contracts
│   └── components.md
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
crates/intrada-web/src/
├── components/
│   ├── autocomplete.rs    # NEW — Reusable autocomplete dropdown
│   ├── tag_input.rs       # NEW — Chip-based tag input with autocomplete
│   ├── text_field.rs      # EXISTING — unchanged
│   ├── mod.rs             # MODIFIED — export new components
│   └── ...
├── views/
│   ├── add_form.rs        # MODIFIED — replace tag TextField with TagInput; add Autocomplete to composer
│   ├── edit_form.rs       # MODIFIED — same changes as add_form
│   └── ...
├── helpers.rs             # EXISTING — parse_tags stays, may add suggestion extraction helpers
└── ...
```

**Structure Decision**: All changes are within `crates/intrada-web/`. Two new component files are added to `components/`; two existing view files are updated. No changes to `intrada-core/` or `intrada-api/`.

## Complexity Tracking

> No constitution violations. No complexity justifications needed.
