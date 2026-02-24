# Implementation Plan: Tempo Progress Charts

**Branch**: `151-tempo-progress-charts` | **Date**: 2026-02-24 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/151-tempo-progress-charts/spec.md`

## Summary

Add a line chart to the item detail page that visualises achieved tempo over time with an optional target BPM reference line and progress percentage. This is a client-side-only feature — no API, database, or core model changes. A new `TempoProgressChart` Leptos component renders SVG following the same pattern as the existing `LineChart`, consuming data already available via `ItemPracticeSummary.tempo_history` from tempo tracking (#52).

## Technical Context

**Language/Version**: Rust stable (1.89.0), 2021 edition
**Primary Dependencies**: leptos 0.8.x (CSR), intrada-core (model types), Tailwind CSS v4
**Storage**: N/A — no new data storage; reads existing precomputed `ItemPracticeSummary`
**Testing**: cargo test (unit), wasm-bindgen-test (component), Playwright (E2E — existing tests only)
**Target Platform**: WASM (browser, CSR via trunk)
**Project Type**: Web application (three-crate workspace: core, web, api)
**Performance Goals**: Chart renders in <100ms for up to 500 data points; no perceptible delay on page load
**Constraints**: SVG-only rendering (no external charting library); must use design system colour tokens
**Scale/Scope**: 1 new component, 1 modified view, 1 modified CSS file; ~200-300 lines of new code

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | New component follows existing LineChart patterns; single responsibility |
| II. Testing Standards | ✅ Pass | Unit tests for chart data preparation; WASM tests not required (pure SVG view) |
| III. UX Consistency | ✅ Pass | Reuses design system tokens, Card container, established chart visual language |
| IV. Performance | ✅ Pass | SVG renders client-side from precomputed data; no new API calls |
| V. Architecture Integrity | ✅ Pass | No core changes; chart is a web shell component consuming ViewModel data |
| VI. Inclusive Design | ✅ Pass | Target line uses dual differentiation (colour + dash pattern); no flashing/sound; aria-label on SVG |

**Post-design re-check**: All gates still pass. No model mutations, no new effects, no architecture boundary crossings.

## Project Structure

### Documentation (this feature)

```text
specs/151-tempo-progress-charts/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 research decisions
├── data-model.md        # Data model (no changes — documents existing entities)
├── quickstart.md        # Verification steps
├── contracts/           # No new API contracts
│   └── README.md
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (files touched)

```text
crates/intrada-web/
├── src/
│   ├── components/
│   │   ├── mod.rs                    # Add tempo_progress_chart export
│   │   └── tempo_progress_chart.rs   # NEW — TempoProgressChart component
│   └── views/
│       ├── detail.rs                 # Replace tempo history list with chart
│       └── design_catalogue.rs       # Add chart showcase entry
├── input.css                         # Add --color-chart-target token (if needed)
└── tests/
    └── wasm.rs                       # No changes expected
```

**Structure Decision**: Web-only change within the existing three-crate workspace. Only the `intrada-web` crate is modified. No core or API changes.

## Complexity Tracking

No constitution violations. This table is intentionally empty.
