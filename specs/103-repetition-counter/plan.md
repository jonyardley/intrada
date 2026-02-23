# Implementation Plan: Repetition Counter

**Branch**: `103-repetition-counter` | **Date**: 2026-02-21 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/103-repetition-counter/spec.md`

## Summary

Add an optional per-item repetition counter to active practice sessions. The counter tracks consecutive correct repetitions toward a configurable target (default 5, range 3–10). "Got it" increments, "missed" decrements (never below zero). When the target is reached, the counter freezes and prompts (but does not force) transition to the next item. Rep targets are configured per entry in the building phase via an opt-in toggle. Final counts and targets persist to the database and display in session summary and history. The feature is fully additive — backward compatible via `Option` fields with `#[serde(default)]`.

## Technical Context

**Language/Version**: Rust stable (1.89.0 CI, MSRV 1.75+)
**Primary Dependencies**: crux_core 0.17.0-rc2, serde 1, ulid 1, chrono 0.4 (core); axum 0.8, libsql 0.9, tokio 1 (API); leptos 0.8.x CSR (web)
**Storage**: Turso (managed libsql/SQLite) via HTTP; localStorage for crash recovery only
**Testing**: cargo test (unit/integration), wasm-bindgen-test 0.3, Playwright (E2E)
**Target Platform**: WASM (web shell), Linux (API server)
**Project Type**: Three-crate workspace (core + web + api)
**Performance Goals**: Counter interaction must feel instant (< 16ms render); no additional API calls during active practice
**Constraints**: Counter state is transient in core during active practice; only final count + target persist to DB on save
**Scale/Scope**: Per-entry data, no performance concern — adds 3 nullable columns to setlist_entries table

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | New fields use `Option<T>` with `#[serde(default)]` — explicit, type-safe. Validation centralised in `validation.rs`. |
| II. Testing Standards | ✅ Pass | Core logic tested via unit tests on events. API persistence tested via integration tests. UI testable via E2E. Boundary test: crash recovery round-trip. |
| III. UX Consistency | ✅ Pass | Reuses existing Button, Card components. New RepCounter component follows glassmorphism design language. "Add rep target" toggle follows same opt-in pattern as intentions. |
| IV. Performance | ✅ Pass | Counter is pure state mutation in core — no I/O. No additional API calls during practice. Crash recovery serialises counter state to existing localStorage key. |
| V. Architecture Integrity | ✅ Pass | Counter logic is pure in `intrada-core`. Shell renders from ViewModel. API handles persistence. No cross-boundary violations. |
| VI. Inclusive Design | ✅ Pass | Counter provides concrete visible goal (supports task focus for ADHD). Achievement prompt is non-blocking. Feature is fully optional — zero disruption when not used. "Got it"/"Missed" buttons are large touch targets (≥44px). |

No violations — gate passes.

## Project Structure

### Documentation (this feature)

```text
specs/103-repetition-counter/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (API contract changes)
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/
  intrada-core/
    src/
      domain/session.rs      # RepetitionState, new events, counter logic
      validation.rs          # MIN_REP_TARGET, MAX_REP_TARGET, DEFAULT_REP_TARGET
      app.rs                 # SetlistEntryView + ActiveSessionView extensions
  intrada-web/
    src/
      components/
        setlist_builder.rs   # RepTargetInput per entry (opt-in toggle)
        session_timer.rs     # RepCounter display + got-it/missed buttons
        session_summary.rs   # Rep count display per entry
      views/
        sessions.rs          # Rep count display in history
  intrada-api/
    src/
      migrations.rs          # 3 new ALTER TABLE migrations
      db/sessions.rs         # Updated SELECT/INSERT, SaveSessionEntry extension
```

**Structure Decision**: No new files needed beyond spec artifacts. All changes extend existing modules following established patterns. No new crates, no new component files (RepCounter inlined in session_timer.rs, RepTargetInput inlined in setlist_builder.rs — both are small enough to not warrant separate files).
