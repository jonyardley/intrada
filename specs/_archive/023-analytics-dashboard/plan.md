# Implementation Plan: Practice Analytics Dashboard

**Branch**: `023-analytics-dashboard` | **Date**: 2026-02-17 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/023-analytics-dashboard/spec.md`

## Summary

Add a read-only analytics dashboard that computes practice insights (weekly summary, streak, 28-day trend line chart, item rankings, score trends) from existing session data. All computation lives in `intrada-core` as pure functions, exposed through the ViewModel. The web shell renders an SVG-based line chart and reuses existing Card/PageHeading components. No new API endpoints or database changes required — the dashboard consumes the already-fetched session list.

## Technical Context

**Language/Version**: Rust stable (1.89.0 in CI; workspace MSRV 1.75+, 2021 edition)
**Primary Dependencies**: crux_core 0.17.0-rc2, leptos 0.8.x (CSR), chrono 0.4, serde 1
**Storage**: N/A (read-only — computed from existing session data, no new persistence)
**Testing**: `cargo test` (core unit tests), `wasm-bindgen-test` (WASM), Playwright (E2E)
**Target Platform**: WASM (browser CSR via Leptos + Trunk)
**Project Type**: Web application (Crux core + Leptos shell + Axum API)
**Performance Goals**: Dashboard renders within 2 seconds, handles 100+ sessions without visible delay
**Constraints**: No new crate dependencies for charts (SVG rendered inline), no WASM bundle size increase beyond the new code
**Scale/Scope**: Single new route (`/analytics`), ~5 new core functions, 1 new view, 1 new SVG chart component

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | Pure functions for analytics, single responsibility per computation |
| II. Testing Standards | ✅ Pass | Core analytics functions testable with synthetic session data, no browser needed |
| III. UX Consistency | ✅ Pass | Reuses Card, PageHeading, Button components; skeleton loading; glassmorphism styling |
| IV. Performance | ✅ Pass | No new dependencies; SVG inline rendering; client-side computation over already-fetched data |
| V. Architecture Integrity | ✅ Pass | All analytics computation in `intrada-core` (pure, no I/O); shell only renders ViewModel |

**Post-Phase 1 re-check**: All gates remain passing. Line chart is pure SVG markup generated from computed data — no DOM access in core.

## Project Structure

### Documentation (this feature)

```text
specs/023-analytics-dashboard/
├── plan.md              # This file
├── research.md          # Phase 0: technology decisions
├── data-model.md        # Phase 1: analytics view model structures
├── quickstart.md        # Phase 1: verification steps
├── contracts/           # Phase 1: no new API contracts (read-only from existing endpoints)
│   └── api-changes.md   # Documents: no API changes needed
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
  intrada-core/
    src/
      analytics.rs           # NEW: pure analytics computation functions
      model.rs               # MODIFIED: add AnalyticsView to ViewModel
      app.rs                 # MODIFIED: compute analytics when sessions load
      lib.rs                 # MODIFIED: export analytics types
  intrada-web/
    src/
      views/
        analytics.rs         # NEW: dashboard page view
        mod.rs               # MODIFIED: add analytics module
      components/
        line_chart.rs        # NEW: SVG line chart component
        stat_card.rs         # NEW: metric display card (streak, weekly total)
        mod.rs               # MODIFIED: add new component modules
      app.rs                 # MODIFIED: add /analytics route + nav link
```

**Structure Decision**: Follows existing workspace structure. New analytics logic in `intrada-core/src/analytics.rs` (pure functions), new view in `intrada-web/src/views/analytics.rs`, new reusable components for chart and stats.

## Complexity Tracking

> No constitution violations — no entries needed.
