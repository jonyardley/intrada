# Implementation Plan: Session Item Scoring

**Branch**: `022-session-scoring` | **Date**: 2026-02-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/022-session-scoring/spec.md`

## Summary

Add an optional 1–5 confidence score field to each completed setlist entry during the session summary review. Persist scores through the existing Crux effect pipeline to the Turso database via a new `score` column on `setlist_entries`. Display scores on session history views and add a chronological progress summary (with latest-score highlight) to each library item's detail page. No new crates, endpoints, or architectural patterns — this extends existing session data flow end-to-end.

## Technical Context

**Language/Version**: Rust stable (1.89.0 in CI; workspace MSRV 1.75+, 2021 edition)
**Primary Dependencies**: crux_core 0.17.0-rc2, leptos 0.8.x (CSR), axum 0.8, libsql 0.9, serde 1, ulid 1, chrono 0.4
**Storage**: Turso (managed libsql/SQLite) via REST API; localStorage for session-in-progress crash recovery only
**Testing**: `cargo test` (core unit tests), `wasm-bindgen-test` (WASM boundary tests), Playwright (E2E)
**Target Platform**: WASM (web frontend via Leptos CSR) + Linux server (API via Axum)
**Project Type**: Workspace with 3 crates (core, web, api)
**Performance Goals**: Scoring interaction must not add perceptible delay to the summary screen; progress query must return within existing page-load time
**Constraints**: Pure core must remain I/O-free; backward compatible with existing sessions (no score = NULL)
**Scale/Scope**: Single-user app; realistic dataset: hundreds of sessions, thousands of entries

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --------- | ------ | ----- |
| V. Architecture Integrity — Pure Core | ✅ Pass | Score field added to core domain types. No I/O in core. New `UpdateEntryScore` event processed purely. Shell renders UI and handles persistence as before. |
| V. Architecture Integrity — Shell Isolation | ✅ Pass | Score input UI lives in `intrada-web` components only. API persistence in `intrada-api` only. |
| V. Architecture Integrity — Effect-Driven | ✅ Pass | Score flows through existing `SavePracticeSession` effect — no new effect type needed. |
| V. Architecture Integrity — Portable | ✅ Pass | `cargo test` in `intrada-core` requires no browser. Score is a plain `Option<u8>`. |
| V. Architecture Integrity — Validation Sharing | ✅ Pass | Score range validation (1–5) defined once in `intrada-core/src/validation.rs`, reused by API. |
| I. Code Quality | ✅ Pass | `Option<u8>` is the simplest possible representation. No new abstractions introduced. |
| II. Testing Standards | ✅ Pass | Core unit tests for score events, API integration tests for score persistence, E2E test for scoring flow. Boundary tested at core↔shell via existing pattern. |
| III. UX Consistency | ✅ Pass | Score input follows existing interaction patterns (inline controls on summary entries, like notes). Consistent with glassmorphism styling. |
| IV. Performance | ✅ Pass | Adding one `Option<u8>` per entry has negligible impact. Progress query is a filter over existing data — no new indexes needed at this scale. |

**Result**: All gates pass. No violations to track.

## Project Structure

### Documentation (this feature)

```text
specs/022-session-scoring/
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
  │   │   └── session.rs          # Add score field to SetlistEntry, SummarySession event
  │   ├── model.rs                # Add score to view types, add ItemScoreHistory
  │   ├── validation.rs           # Add score range validation constant
  │   ├── app.rs                  # Add UpdateEntryScore event, update compute_practice_summary
  │   └── lib.rs                  # Re-export new types if needed
  intrada-web/
  ├── src/
  │   ├── components/
  │   │   └── session_summary.rs  # Add score input controls per entry
  │   ├── views/
  │   │   ├── detail.rs           # Add progress/score history section
  │   │   └── session_detail.rs   # Display scores on past session entries
  │   └── api_client.rs           # No changes (PracticeSession shape updated via core)
  intrada-api/
  ├── src/
  │   ├── db/
  │   │   └── sessions.rs         # Add score to SaveSessionEntry, SQL queries
  │   ├── routes/
  │   │   └── sessions.rs         # Validate score range on save
  │   └── migrations.rs           # Add migration: ALTER TABLE setlist_entries ADD COLUMN score INTEGER
```

**Structure Decision**: Existing 3-crate workspace. No new crates or directories. Changes are additive within existing modules.
