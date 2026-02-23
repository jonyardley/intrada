# Implementation Plan: Rep History Tracking

**Branch**: `104-rep-history` | **Date**: 2026-02-21 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/104-rep-history/spec.md`

## Summary

Extend the repetition counter with three changes: (1) record the full sequence of Got it / Missed actions as a `rep_history` field on each setlist entry, persisted through the full data pipeline; (2) change hide/show toggle to preserve rep state instead of clearing it; (3) add an icon to the "Rep Counter" enable button. The history enables future analytics to distinguish clean runs from volatile practice patterns.

## Technical Context

**Language/Version**: Rust stable (1.89.0 in CI; workspace MSRV 1.75+, intrada-api requires 1.78+)
**Primary Dependencies**: crux_core 0.17.0-rc2, serde 1, axum 0.8, leptos 0.8.x CSR, libsql 0.9
**Storage**: Turso (managed libsql/SQLite) via HTTP protocol; localStorage for crash recovery only
**Testing**: cargo test (unit/integration), wasm-bindgen-test 0.3
**Target Platform**: WASM (web shell) + Linux server (API)
**Project Type**: Three-crate Rust workspace (core, web, api)
**Performance Goals**: No perceptible delay on rep counter interactions; history serialisation must be fast for typical sequences (<50 actions)
**Constraints**: SQLite column storage for history (JSON text); no new tables
**Scale/Scope**: Single new field added to existing SetlistEntry pipeline

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | New `RepAction` enum is explicit, typed, single-purpose. No dead code introduced. |
| II. Testing Standards | ✅ Pass | Each behaviour change has unit tests in core. API validation tested in integration tests. |
| III. UX Consistency | ✅ Pass | Uses existing Button component with icon prefix. Summary display follows existing rep count pattern. |
| IV. Performance | ✅ Pass | History is a small Vec serialised to JSON in a single column. No additional queries. |
| V. Architecture Integrity | ✅ Pass | `RepAction` enum defined in core. Serialisation via serde. Shell renders from ViewModel. API persists via existing column pattern. |
| VI. Inclusive Design | ✅ Pass | Hide/show preserves progress (reduces anxiety about losing work). No new decision points added. |

No violations. No complexity tracking needed.

### Post-Phase 1 Re-check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | `RepAction` enum is single-purpose. `DisableRepCounter` removal eliminates dead code path (state clearing). |
| II. Testing Standards | ✅ Pass | Quickstart defines 8+ new unit tests covering history append, freeze, persistence, and backward compat. |
| III. UX Consistency | ✅ Pass | Attempt count follows the existing "Reps: X / Y" pattern with `·` separator. Icon uses Unicode consistent with ✓ and ✗. |
| IV. Performance | ✅ Pass | JSON column avoids extra queries. Typical history <50 items = negligible serialisation cost. |
| V. Architecture Integrity | ✅ Pass | Counter visibility is a Leptos signal (UI state per state boundary rules). `RepAction` is a core domain type. API persists via existing column pattern. No shell→core leaks. |
| VI. Inclusive Design | ✅ Pass | Hide/show preserves progress (no data loss anxiety). Button with icon is more discoverable. No new decision points. |

All gates pass. Ready for `/speckit-tasks`.

## Project Structure

### Documentation (this feature)

```text
specs/104-rep-history/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit-tasks)
```

### Source Code (repository root)

```text
crates/
  intrada-core/
    src/
      domain/
        session.rs       # SetlistEntry + RepAction enum, event handlers, freeze_rep_state
      model.rs           # ActiveSessionView + SetlistEntryView (add rep_history field)
      validation.rs      # Validation constants (no changes needed)
  intrada-web/
    src/
      components/
        session_timer.rs   # Icon on enable button, hide/show semantics
        session_summary.rs # Attempt count display
      views/
        sessions.rs        # Rep badge (no changes needed — badge already shows count/target)
  intrada-api/
    src/
      db/
        sessions.rs        # SaveSessionEntry + SQL queries (add rep_history column)
      routes/
        sessions.rs        # API validation (add rep_history consistency check)
      migrations.rs        # New migration for rep_history column
```

**Structure Decision**: All changes fit within the existing three-crate structure. No new files or modules needed — only field additions and behaviour modifications to existing types and handlers.
