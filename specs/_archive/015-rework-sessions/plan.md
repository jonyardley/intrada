# Implementation Plan: Rework Sessions (Setlist Model)

**Branch**: `015-rework-sessions` | **Date**: 2026-02-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/015-rework-sessions/spec.md`

## Summary

Replace the flat session model (one session = one library item + duration) with a setlist-based model where users build an ordered list of items to practise, work through them with a running timer, and receive an end-of-session summary with per-item notes. The implementation extends the existing Crux event/effect architecture with new domain types, a session lifecycle state machine in the core `Model`, and reworked web shell UI components. All old session code, types, and stored data are removed and replaced.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition)
**Primary Dependencies**: crux_core 0.17.0-rc2 (workspace), leptos 0.8.x (CSR), leptos_router 0.8.x, web-sys (Storage+Window), wasm-bindgen, serde/serde_json 1, ulid 1, chrono 0.4, send_wrapper 0.6
**Storage**: localStorage (web: `intrada:sessions` key for completed sessions, `intrada:session-in-progress` key for crash recovery)
**Testing**: `cargo test` (core unit tests), `wasm-bindgen-test` (WASM boundary tests), Playwright (E2E)
**Target Platform**: WASM (browser, CSR)
**Project Type**: Workspace with pure core (`intrada-core`) + web shell (`intrada-web`)
**Performance Goals**: view() with 10k items + sessions < 200ms, UI interactions feel instant, session recovery on reload
**Constraints**: Zero I/O in core, localStorage-only persistence, client-side timer, single-threaded WASM
**Scale/Scope**: Solo musician user, hundreds of sessions over time, setlists of 1–20 items

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Code Quality ✅ PASS

- **Clarity over cleverness**: Session state machine with explicit `SessionStatus` enum makes lifecycle states self-documenting
- **Single Responsibility**: Session domain logic stays in `session.rs`; shell persistence in `core_bridge.rs`; UI in dedicated components
- **Consistent Style**: Same patterns as existing piece/exercise domain modules
- **No Dead Code**: Old session code (`Session`, `SessionEvent`, `LogSession`, `UpdateSession`) fully removed (FR-016)
- **Explicit over Implicit**: All state transitions via events; timer timestamps explicit
- **Type Safety**: `EntryStatus` and `CompletionStatus` enums prevent invalid states

### II. Testing Standards ✅ PASS

- **Test Coverage**: All session events tested in core; boundary tests for localStorage round-trips
- **Test Independence**: Each test creates its own model instance (existing pattern)
- **Meaningful Assertions**: Tests verify event outcomes (model state), not internals
- **Fast Feedback**: Core tests are pure Rust, sub-second execution
- **Boundary Tests**: Core tests (events → effects), WASM tests (persistence round-trips), E2E (full session flow)

### III. User Experience Consistency ✅ PASS

- **Visual Consistency**: Session UI reuses existing component library (Button, Card, TextField, TextArea, TypeBadge)
- **Interaction Patterns**: Setlist building follows same add/remove/reorder patterns as library management
- **Error Communication**: Validation errors use existing `last_error` pattern displayed consistently
- **Loading States**: Session recovery on load provides feedback if restoring in-progress session

### IV. Performance Requirements ✅ PASS

- **WASM Bundle Size**: No new dependencies added; only new domain types and events
- **Render Performance**: `view()` computation remains O(sessions × entries) which is acceptable for expected scale
- **Data Efficiency**: In-progress session uses separate localStorage key; completed sessions serialised on save only
- **Startup Time**: Loading sessions + checking for in-progress session adds minimal overhead

### V. Architecture Integrity ✅ PASS

- **Pure Core**: All session logic (state machine, validation, time computation from timestamps) in `intrada-core` with zero I/O
- **Shell Isolation**: Timer (`setInterval`), localStorage, DOM all in `intrada-web` shell only
- **Effect-Driven Communication**: New `StorageEffect` variants for session persistence; shell handles all I/O
- **Portable by Design**: `cargo test` in `intrada-core` requires no browser; core compiles on any Rust target
- **Validation Sharing**: Session note validation reuses `MAX_NOTES` constant from core

## Project Structure

### Documentation (this feature)

```text
specs/015-rework-sessions/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── session-events.md
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── intrada-core/
│   └── src/
│       ├── app.rs                  # Updated: Event enum, StorageEffect variants, view()
│       ├── model.rs                # Updated: Model fields, ViewModel shape, new view types
│       ├── validation.rs           # Updated: New session validation functions
│       ├── error.rs                # Unchanged
│       └── domain/
│           ├── mod.rs              # Unchanged
│           ├── piece.rs            # Unchanged
│           ├── exercise.rs         # Unchanged
│           ├── types.rs            # Updated: Replace session types, add new types
│           └── session.rs          # Rewritten: New state machine, event handler
│
└── intrada-web/
    └── src/
        ├── app.rs                  # Updated: New routes for session flow
        ├── core_bridge.rs          # Updated: New StorageEffect handling, in-progress key
        ├── types.rs                # Unchanged
        ├── helpers.rs              # Unchanged
        ├── validation.rs           # Updated: New session form validation
        ├── data.rs                 # Unchanged
        ├── components/
        │   ├── practice_timer.rs   # Removed (replaced by session practice view)
        │   ├── session_history.rs  # Removed (replaced by session history view)
        │   ├── setlist_builder.rs  # New: Setlist assembly UI
        │   ├── session_timer.rs    # New: Active session timer UI
        │   ├── session_summary.rs  # New: Post-session summary UI
        │   ├── setlist_entry.rs    # New: Individual setlist item card
        │   └── [existing components unchanged]
        └── views/
            ├── sessions.rs         # Rewritten: Session history list
            ├── session_new.rs      # New: Setlist building view
            ├── session_active.rs   # New: Active practice view
            ├── session_summary.rs  # New: Summary/review view
            └── detail.rs           # Updated: Practice summary from new model
```

**Structure Decision**: Existing workspace structure maintained — pure core in `intrada-core`, web shell in `intrada-web`. No new crates or structural changes. Session UI split into dedicated views matching the session lifecycle phases (building → active → summary) rather than inline components on the detail page.

## Complexity Tracking

> No constitution violations identified. All five principles pass without exceptions.
