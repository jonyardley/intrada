# Implementation Plan: Shuttle API Server & Database

**Branch**: `017-shuttle-api-deploy` | **Date**: 2026-02-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/017-shuttle-api-deploy/spec.md`

## Summary

Deploy the Intrada web app to Shuttle.rs with an Axum API server that serves the compiled WASM build as static files and provides REST CRUD endpoints for library items (pieces, exercises) and practice sessions. Data is persisted in Shuttle-managed Postgres via `shuttle-shared-db`. The web shell switches from localStorage-only persistence to API calls as the primary data source, keeping localStorage as a read cache for fast initial loads. The Crux core (`intrada-core`) remains untouched — only the web shell's storage layer changes. Single-user, no auth (auth deferred to feature 019).

## Technical Context

**Language/Version**: Rust stable (1.75+ workspace minimum; axum 0.8 requires 1.78+)
**Primary Dependencies**: shuttle-runtime 0.57, shuttle-axum 0.57, axum 0.8, sqlx 0.8 (Postgres), tower-http 0.6 (fs), intrada-core (workspace)
**Storage**: Shuttle-managed Postgres via `shuttle-shared-db` (zero-config, auto-provisioned)
**Testing**: `cargo test` (unit + integration), contract tests for API endpoints, existing E2E tests (Playwright)
**Target Platform**: Shuttle.rs (Linux container) for server; WASM (browser) for frontend
**Project Type**: Web application (API server + WASM frontend served from same origin)
**Performance Goals**: API response < 200ms, cached data display < 500ms, full page load < 5s
**Constraints**: Single-user, no auth, no offline write queuing, free tier (0.5 GB DB, 1 GB egress)
**Scale/Scope**: Single user, ~100s of library items, ~1000s of practice sessions

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Architecture Integrity ✅ PASS

- **Pure Core**: `intrada-core` remains unchanged (FR-015). The new server crate imports domain types and validation only — no Crux events, effects, or state machine types.
- **Shell Isolation**: Web shell changes are confined to the storage/data-access layer in `intrada-web`. The server (`intrada-api`) is a new shell that reuses core types.
- **Effect-Driven Communication**: Not applicable to the server crate (server is not a Crux shell — it's a REST API that uses core types directly for validation and serialization).
- **Portable by Design**: `intrada-core` continues to compile and test without WASM or browser dependencies.
- **Validation Sharing**: Server reuses `intrada-core::validation::*` functions directly (FR-007). No duplication.

### II. Code Quality ✅ PASS

- Single responsibility: server crate handles HTTP/DB; core handles domain logic/validation.
- Consistent style: `cargo clippy -- -D warnings`, `cargo fmt` applied to new crate.
- No dead code: only types actually needed by the API are imported from core.

### III. Testing Standards ✅ PASS

- Server API endpoints tested via contract tests (HTTP request/response assertions).
- Core unit tests (142+) remain unchanged and passing.
- E2E tests updated to work against API-backed app.
- Boundary: core↔server boundary tested via validation round-trips in contract tests.

### IV. User Experience Consistency ✅ PASS

- Error messages from API validation use the same `LibraryError` messages as the client.
- Loading states: web shell shows cached data immediately, then refreshes from server.
- No visual changes — only data source changes.

### V. Performance Requirements ✅ PASS

- WASM bundle size: no change (server is separate crate).
- Cached data loads instantly from localStorage (< 500ms target).
- Server responses expected < 200ms for single-user Postgres queries.

### Post-Design Re-check ✅ PASS

No violations introduced during Phase 1 design. The server crate adds a third workspace member but follows the existing `crates/*` pattern. All types imported from core are public and already exported.

## Project Structure

### Documentation (this feature)

```text
specs/017-shuttle-api-deploy/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── api.md           # REST API endpoint contracts
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── intrada-core/          # Pure Crux core (unchanged)
│   └── src/
│       ├── app.rs
│       ├── domain/
│       ├── error.rs
│       ├── model.rs
│       └── validation.rs
├── intrada-web/           # Web shell (Leptos CSR + WASM)
│   └── src/
│       ├── app.rs
│       ├── core_bridge.rs # Modified: API calls + localStorage cache
│       ├── components/
│       └── views/
└── intrada-api/           # NEW: Axum API server for Shuttle
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs        # Shuttle entry point (#[shuttle_runtime::main])
    │   ├── routes/        # API route handlers
    │   │   ├── mod.rs
    │   │   ├── pieces.rs
    │   │   ├── exercises.rs
    │   │   ├── sessions.rs
    │   │   └── health.rs
    │   ├── db/            # Database access layer (sqlx queries)
    │   │   ├── mod.rs
    │   │   ├── pieces.rs
    │   │   ├── exercises.rs
    │   │   └── sessions.rs
    │   ├── error.rs       # ApiError enum (IntoResponse impl)
    │   └── state.rs       # AppState (PgPool wrapper)
    └── migrations/        # SQL migrations (sqlx)
        ├── 001_create_pieces.sql
        ├── 002_create_exercises.sql
        └── 003_create_sessions.sql

Shuttle.toml               # NEW: Shuttle project configuration
.github/workflows/ci.yml   # Modified: add deploy job
e2e/                       # Existing: Playwright tests (may need config updates)
```

**Structure Decision**: New `crates/intrada-api/` server crate added to the workspace, following the existing `crates/*` pattern. The server imports `intrada-core` for domain types and validation. The web shell (`intrada-web`) is modified to use API calls instead of direct localStorage writes.

## Complexity Tracking

> No constitution violations to justify. The third workspace crate follows the established pattern and each crate has a clear single responsibility (core logic, web shell, API server).
