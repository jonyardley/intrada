# Implementation Plan: Session Week Strip Navigator

**Branch**: `154-session-week-strip` | **Date**: 2026-03-04 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/154-session-week-strip/spec.md`

## Summary

Replace the flat session list on `/sessions` with a weekly calendar strip navigator. The week strip shows Mon–Sun with date numbers, dot indicators for days with sessions, and allows day selection to filter session cards. Navigation between weeks via arrows and mobile swipe gestures. This is a **pure frontend feature** — no new API endpoints or database changes needed. Session data is already available in the `ViewModel.sessions` and will be filtered client-side by date using `chrono`.

## Technical Context

**Language/Version**: Rust stable (1.89.0), compiled to WASM
**Primary Dependencies**: Leptos 0.8 (CSR), chrono 0.4, web-sys 0.3 (touch events)
**Storage**: N/A — no new storage; reads existing `ViewModel.sessions` from Crux core
**Testing**: cargo test (unit tests for date helpers in intrada-web), wasm-bindgen-test (component logic), Playwright (E2E)
**Target Platform**: WASM (browser), responsive mobile-first
**Project Type**: Web application (existing three-crate workspace)
**Performance Goals**: Week strip renders instantly; filtering ~1000 sessions by date completes in <16ms (single frame)
**Constraints**: No new API calls for week navigation; all filtering is client-side on already-fetched session data
**Scale/Scope**: Single page view refactor; 2 new Leptos components (WeekStrip, DayCell), modifications to sessions.rs view, new date utility helpers

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| # | Principle | Status | Notes |
|---|-----------|--------|-------|
| I | Code Quality | ✅ PASS | New components follow single responsibility (WeekStrip, DayCell). Date helpers are pure functions with unit tests. No dead code — existing `SessionRow` is reused. |
| II | Testing Standards | ✅ PASS | Date grouping/filtering logic tested via unit tests on pure functions. Week navigation tested via E2E. Component boundaries testable independently. |
| III | UX Consistency | ✅ PASS | Reuses existing Card, PageHeading, Icon, SkeletonBlock/SkeletonLine components. New WeekStrip/DayCell follow glassmorphism design language. Design tokens used throughout — no raw colours. ARIA attributes on interactive day cells. |
| IV | Performance | ✅ PASS | No new API calls — filters existing ViewModel data client-side. chrono date parsing uses already-available `started_at` RFC3339 strings. No bundle size concern — chrono already in dependency tree. |
| V | Architecture Integrity | ✅ PASS | Pure frontend change in intrada-web shell only. No changes to intrada-core or intrada-api. Session data flows through existing Event→Model→ViewModel pipeline. Week/day selection state is ephemeral UI state — lives in Leptos signals per the state boundary rules. |
| VI | Inclusive Design | ✅ PASS | Predictable navigation (arrows always in same position). No sudden animations or auto-play. Week strip provides visible time cues for practice history. No streak-based messaging — dot indicators simply show which days had sessions. |

**Result: All 6 principles pass. No violations to track.**

## Project Structure

### Documentation (this feature)

```text
specs/154-session-week-strip/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (empty — no new API contracts)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/intrada-web/
├── src/
│   ├── components/
│   │   ├── mod.rs              # Add week_strip module + re-exports
│   │   └── week_strip.rs       # NEW: WeekStrip + DayCell components
│   ├── views/
│   │   ├── sessions.rs         # MODIFY: Replace flat list with week strip view
│   │   └── sessions_all.rs     # NEW: "Show all sessions" full list view
│   ├── helpers.rs              # MODIFY: Add date grouping/week calculation helpers
│   └── app.rs                  # MODIFY: Add /sessions/all route
├── input.css                   # MODIFY: Add week-strip-specific utility classes if needed
└── tests/                      # Date helper unit tests (in helpers.rs #[cfg(test)])
```

**Structure Decision**: This feature fits entirely within the existing `intrada-web` crate (the Leptos shell). No changes to `intrada-core` or `intrada-api`. The WeekStrip and DayCell are new Leptos components in `components/week_strip.rs`. The existing `SessionRow` component in `sessions.rs` is reused as-is for rendering session cards. A new `sessions_all.rs` view handles the "Show all sessions" route.

## Complexity Tracking

> No constitution violations — this section is intentionally empty.
