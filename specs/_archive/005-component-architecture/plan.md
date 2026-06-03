# Implementation Plan: Web App Component Architecture

**Branch**: `005-component-architecture` | **Date**: 2026-02-14 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/005-component-architecture/spec.md`

## Summary

Refactor the monolithic 1,906-line `crates/intrada-web/src/main.rs` into a multi-file component architecture using standard Rust module patterns. The restructuring uses a three-layer approach — shared components, views, and helpers — to decompose the single file into ~15 focused modules. This is a pure refactoring effort: zero new features, zero behaviour changes, zero new dependencies. All 82+ existing workspace tests must pass without modification.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition)
**Primary Dependencies**: leptos 0.7 (CSR), crux_core 0.17.0-rc2 (workspace), send_wrapper 0.6, wasm-bindgen, console_error_panic_hook
**Storage**: N/A (stub data in-memory; no persistence changes)
**Testing**: `cargo test` (workspace-wide, 82+ tests across intrada-core and intrada-cli)
**Target Platform**: WASM (browser), built via Trunk
**Project Type**: Single crate (intrada-web within workspace)
**Performance Goals**: Zero regression — identical WASM binary behaviour, no measurable load time increase
**Constraints**: No file exceeds 300 lines (SC-001); total line count overhead < 10% (SC-008); zero clippy warnings (SC-007)
**Scale/Scope**: ~10 components, ~1,900 lines → ~15 files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Code Quality

| Rule | Status | Notes |
|------|--------|-------|
| Single Responsibility | ✅ PASS | Each module will have one clear purpose — this refactoring directly improves SRP compliance |
| No Dead Code | ✅ PASS | No new dead code introduced; existing dead code (validate_piece_form impossible condition) is out of scope for this refactoring but noted |
| Consistent Style | ✅ PASS | All new files follow existing Rust/Leptos conventions with `cargo clippy` verification |
| Clarity over cleverness | ✅ PASS | File/module naming follows obvious conventions; no clever abstractions |
| Explicit over Implicit | ✅ PASS | Module re-exports are explicit; no wildcard re-exports hiding dependencies |
| Type Safety | ✅ PASS | No type changes; existing types moved to shared module |

### II. Testing Standards

| Rule | Status | Notes |
|------|--------|-------|
| Test Coverage | ✅ PASS | All 82+ existing tests must pass unchanged (FR-009) |
| Test Independence | ✅ PASS | No test changes — refactoring is transparent to test suite |
| Contract Tests | ✅ PASS | No new API boundaries introduced; existing Crux core contract unchanged |

### III. User Experience Consistency

| Rule | Status | Notes |
|------|--------|-------|
| Interaction Patterns | ✅ PASS | Zero behaviour changes — pure code reorganisation |
| Error Communication | ✅ PASS | Error display components moved but not modified |
| Accessibility | ✅ PASS | All ARIA attributes and semantic HTML preserved exactly |

### IV. Performance Requirements

| Rule | Status | Notes |
|------|--------|-------|
| Response Time Budgets | ✅ PASS | No runtime changes; WASM binary compiles from same code |
| Resource Limits | ✅ PASS | No new allocations or resource usage patterns |
| Measurement | ✅ PASS | Verified by: compilation success + identical test results + manual smoke test |

**Gate Result**: ✅ ALL GATES PASS — proceed to implementation.

## Project Structure

### Documentation (this feature)

```text
specs/005-component-architecture/
├── plan.md              # This file
├── research.md          # Phase 0 output — Leptos module patterns research
├── quickstart.md        # Phase 1 output — verification scenarios
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

*Note: No data-model.md or contracts/ needed — this is a pure refactoring with no new entities or API boundaries.*

### Source Code (repository root)

**Before** (current state):
```text
crates/intrada-web/src/
└── main.rs              # 1,906 lines — everything in one file
```

**After** (target state):
```text
crates/intrada-web/src/
├── main.rs              # ~10 lines — entry point only (wasm_bindgen_start, mount)
├── app.rs               # ~120 lines — root App component with ViewState matching
├── types.rs             # ~25 lines — ViewState enum, SharedCore type alias
├── helpers.rs           # ~50 lines — parse_tags, parse_tempo, parse_tempo_display
├── validation.rs        # ~195 lines — validate_piece_form, validate_exercise_form
├── data.rs              # ~60 lines — create_stub_data, SAMPLE_PIECES constant
├── core_bridge.rs       # ~25 lines — process_effects function
├── components/
│   ├── mod.rs           # ~5 lines — re-exports
│   ├── form_field_error.rs  # ~20 lines — FormFieldError component
│   └── library_item_card.rs # ~100 lines — LibraryItemCard component
└── views/
    ├── mod.rs           # ~10 lines — re-exports
    ├── library_list.rs  # ~170 lines — LibraryListView component
    ├── detail.rs        # ~220 lines — DetailView component
    ├── add_piece.rs     # ~200 lines — AddPieceForm component
    ├── add_exercise.rs  # ~220 lines — AddExerciseForm component
    ├── edit_piece.rs    # ~240 lines — EditPieceForm component
    └── edit_exercise.rs # ~275 lines — EditExerciseForm component
```

**Structure Decision**: Three-layer flat structure with `components/` (shared reusable building blocks), `views/` (full-page view components), and root-level modules for non-visual logic. This aligns with the spec's layered architecture requirement while keeping navigation simple for ~15 files. Each `#[component]` function becomes its own file. Non-visual logic (types, validation, helpers, data, core bridge) gets its own dedicated module grouped by purpose.

**Dependency Direction** (DAG, no cycles):
```
main.rs → app.rs → views/* → components/*
              ↓         ↓           ↓
         types.rs   helpers.rs  validation.rs
              ↓         ↓           ↓
         core_bridge.rs  data.rs
```

All modules may import from `types.rs`. Views import from `components/`, `helpers.rs`, `validation.rs`, `data.rs`, and `core_bridge.rs`. Components import from `types.rs` only. No circular dependencies.

## Complexity Tracking

> No constitution violations — table not needed.
