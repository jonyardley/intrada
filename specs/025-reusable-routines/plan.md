# Implementation Plan: Reusable Routines

**Branch**: `025-reusable-routines` | **Date**: 2026-02-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/025-reusable-routines/spec.md`

## Summary

Add a "routines" feature that allows musicians to save practice setlists as named, reusable templates. A new `Routine` domain entity (with `RoutineEntry` children) is introduced across all three crates: core domain types, events, and validation; two new database tables with API CRUD endpoints; and web shell UI extensions to the SetlistBuilder and Session Summary components plus two new pages (`/routines`, `/routines/:id/edit`). Routines are persisted to the server and fetched on startup alongside library and session data. Loading a routine into a session is additive — entries are appended, not replaced.

## Technical Context

**Language/Version**: Rust stable (1.89.0 in CI; workspace MSRV 1.75+, 2021 edition)
**Primary Dependencies**: crux_core 0.17.0-rc2, leptos 0.8.x (CSR), leptos_router 0.8.x, axum 0.8, libsql 0.9, serde 1, ulid 1, chrono 0.4
**Storage**: Turso (managed libsql/SQLite) via REST API; localStorage for session-in-progress crash recovery only
**Testing**: `cargo test` (core unit tests), `wasm-bindgen-test` (WASM boundary tests), Playwright (E2E)
**Target Platform**: WASM (web frontend via Leptos CSR) + Linux server (API via Axum)
**Project Type**: Workspace with 3 crates (core, web, api)
**Performance Goals**: Save/load routine interactions must not add perceptible delay; routine list fetch must complete within existing startup time
**Constraints**: Pure core must remain I/O-free; new entity follows existing patterns exactly; additive loading means no mutation of existing setlist entries
**Scale/Scope**: Single-user app; realistic dataset: dozens of routines, each with 3–15 entries

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --------- | ------ | ----- |
| V. Architecture Integrity — Pure Core | ✅ Pass | Routine domain types, events, and validation live in `intrada-core`. No I/O in core. Events emit `StorageEffect` commands processed by the shell. |
| V. Architecture Integrity — Shell Isolation | ✅ Pass | All HTTP calls, DOM rendering, and localStorage access in `intrada-web` only. Database access in `intrada-api` only. |
| V. Architecture Integrity — Effect-Driven | ✅ Pass | New `StorageEffect` variants (`SaveRoutine`, `UpdateRoutine`, `DeleteRoutine`, `LoadRoutines`) follow existing pattern. Core communicates via `Command<Effect, Event>`. |
| V. Architecture Integrity — Portable | ✅ Pass | `cargo test` in `intrada-core` requires no browser. Routine types use only `String`, `usize`, `DateTime<Utc>`. |
| V. Architecture Integrity — Validation Sharing | ✅ Pass | `MAX_ROUTINE_NAME` and `validate_routine_name()` defined once in core `validation.rs`, reused by API routes. |
| I. Code Quality | ✅ Pass | New types follow identical patterns to existing Piece/Exercise/Session. No new abstractions or design patterns introduced. |
| II. Testing Standards | ✅ Pass | Core unit tests for all routine events. API integration tests for CRUD. Boundary tests via existing WASM pattern. |
| III. UX Consistency | ✅ Pass | Reuses existing components (glass-card, TextField, BackLink, PageHeading, Badge). Save form follows inline expand/collapse pattern. Glassmorphism styling maintained. |
| IV. Performance | ✅ Pass | Routines fetched in parallel with library data on startup. Dozens of routines with 3–15 entries each — negligible payload. |
| VI. Inclusive Design | ✅ Pass | Core benefit: reduces decisions to start practising (load routine = one tap). Predictable navigation (consistent page patterns). No new sounds, animations, or streaks. |

**Result**: All gates pass. No violations to track.

## Project Structure

### Documentation (this feature)

```text
specs/025-reusable-routines/
├── spec.md
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── api-changes.md
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (files modified by this feature)

```text
crates/
  intrada-core/
  ├── src/
  │   ├── domain/
  │   │   ├── mod.rs                # Add pub mod routine + re-exports
  │   │   └── routine.rs            # NEW — Routine, RoutineEntry, RoutineEvent, handler
  │   ├── model.rs                  # Add routines to Model; RoutineView, RoutineEntryView to ViewModel
  │   ├── validation.rs             # Add MAX_ROUTINE_NAME, validate_routine_name()
  │   ├── app.rs                    # Add Routine(RoutineEvent), RoutinesLoaded, StorageEffect variants, view() update
  │   └── lib.rs                    # Re-export new types
  intrada-web/
  ├── src/
  │   ├── components/
  │   │   ├── mod.rs                # Add routine component module declarations
  │   │   ├── setlist_builder.rs    # Add "Load Routine" section + "Save as Routine" form
  │   │   ├── session_summary.rs    # Add "Save as Routine" button + inline form
  │   │   ├── routine_save_form.rs  # NEW — shared inline save form component
  │   │   └── routine_loader.rs     # NEW — routine list in SetlistBuilder with Load buttons
  │   ├── views/
  │   │   ├── mod.rs                # Add routine view module declarations
  │   │   ├── routines.rs           # NEW — /routines management page
  │   │   └── routine_edit.rs       # NEW — /routines/:id/edit page
  │   ├── app.rs                    # Add /routines and /routines/:id/edit routes
  │   ├── api_client.rs             # Add fetch_routines, create_routine, update_routine, delete_routine
  │   └── core_bridge.rs            # Handle new StorageEffects, add refresh_routines, update fetch_initial_data
  intrada-api/
  ├── src/
  │   ├── db/
  │   │   ├── mod.rs                # Add pub mod routines
  │   │   └── routines.rs           # NEW — CRUD for routines + routine_entries tables
  │   ├── routes/
  │   │   ├── mod.rs                # Nest /routines in api_routes()
  │   │   └── routines.rs           # NEW — Axum handlers (list, get, create, update, delete)
  │   └── migrations.rs             # Add migrations 0007 (routines table) + 0008 (routine_entries table)
```

**Structure Decision**: Existing 3-crate workspace. No new crates or directories at the workspace level. New files added within existing module structures following established patterns. Two new database tables (parent + child) mirror the sessions/setlist_entries pattern.

## Complexity Tracking

No entries needed — all gates pass without violations.
