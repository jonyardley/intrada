# Implementation Plan: Focus Mode

**Branch**: `048-focus-mode` | **Date**: 2026-02-23 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/048-focus-mode/spec.md`

## Summary

Strip the active practice screen to its essentials: current item name, a circular
progress ring with timer, rep counter (when active), and session controls. Hide
the navigation bar, completed items list, and non-essential UI. Add a toggle
button to temporarily reveal hidden elements. Add transition prompts when an
item's planned duration elapses. Requires adding an optional per-item planned
duration field across the full stack (DB → API → Core → Web).

## Technical Context

**Language/Version**: Rust stable 1.89.0 (workspace MSRV 1.75+, API requires 1.78+)
**Primary Dependencies**: crux_core 0.17.0-rc2, leptos 0.8.x (CSR), axum 0.8, libsql 0.9
**Storage**: Turso (managed libsql/SQLite) — new `planned_duration_secs` column on `setlist_entries`
**Testing**: cargo test (unit/integration), wasm-bindgen-test 0.3, Playwright (E2E)
**Target Platform**: WASM (web shell) + Linux server (API)
**Project Type**: Web application (three-crate workspace: core, web, api)
**Performance Goals**: Progress ring animation smooth at 60fps; timer tick <1ms overhead
**Constraints**: No new external dependencies; SVG progress ring via inline markup
**Scale/Scope**: ~10 files modified, 2 new components, 1 DB migration

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | No complexity added beyond requirements; progress ring is self-contained |
| II. Testing Standards | ✅ Pass | Core unit tests for new event + duration validation; E2E for focus mode toggle |
| III. UX Consistency | ✅ Pass | Duration input follows existing rep target pattern; new components use design tokens |
| IV. Performance | ✅ Pass | No new API calls; progress ring is CSS-animated SVG; timer comparison is O(1) |
| V. Architecture Integrity | ✅ Pass | `planned_duration_secs` is domain data in core model; focus mode toggle is Leptos signal (UI state); no core↔shell boundary violations |
| VI. Inclusive Design | ✅ Pass | This feature IS the inclusive design feature — reduces visual noise, externalises time via progress ring, supports ADHD-informed minimal UI |

**Post-design re-check**: All gates still pass. No new dependencies, no architectural violations.

## Project Structure

### Documentation (this feature)

```text
specs/048-focus-mode/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: technical decisions
├── data-model.md        # Phase 1: entity changes
├── quickstart.md        # Phase 1: verification steps
├── contracts/
│   └── api-changes.md   # Phase 1: API contract changes
└── checklists/
    └── requirements.md  # Spec quality checklist
```

### Source Code (files to modify)

```text
crates/
  intrada-api/
    src/
      migrations.rs          # Migration 0025: add planned_duration_secs column
      db/sessions.rs          # Update SQL queries, row parsing
      routes/sessions.rs      # Update SaveSessionEntry struct
  intrada-core/
    src/
      domain/session.rs       # Add planned_duration_secs to SetlistEntry, new event
      model.rs                # Update view models (ActiveSessionView, SetlistEntryView)
      validation.rs           # Add duration validation constants
      app.rs                  # Handle SetEntryDuration event
  intrada-web/
    src/
      app.rs                  # Add focus_mode signal, conditional header/footer
      components/
        session_timer.rs      # Restructure for focus mode, add progress ring + transition prompt
        setlist_builder.rs    # Add duration input per entry
        mod.rs                # Export new components
      views/
        session_active.rs     # Set/clear focus_mode signal on mount/unmount
    input.css                 # Progress ring utilities, focus mode styles
```

**Structure Decision**: Existing three-crate structure preserved. No new crates, no new
directories. Changes fit within established patterns.

## Complexity Tracking

No constitution violations to justify. All changes follow established patterns.
