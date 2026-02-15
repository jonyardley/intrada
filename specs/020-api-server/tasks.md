# Tasks: API Server

**Input**: Design documents from `/specs/020-api-server/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: No test tasks — tests are not explicitly requested in the feature specification. Verification is via quickstart.md curl commands and `cargo test` / `cargo clippy`.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Create the intrada-api crate, deployment config, and project scaffolding.

- [X] T001 Create `crates/intrada-api/Cargo.toml` with dependencies: axum 0.8, tokio 1 (full), tower-http 0.6 (cors), libsql 0.9 (remote feature, no default features), libsql_migration 0.2.2 (content feature, no default features), serde/serde_json (workspace), ulid (workspace), chrono (workspace), thiserror (workspace), tracing 0.1, tracing-subscriber 0.3 (env-filter), intrada-core (path = "../intrada-core")
- [X] T002 [P] Create `crates/intrada-api/src/main.rs` with minimal Axum server skeleton: read `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`, `ALLOWED_ORIGIN` from env, build libsql Database with `Builder::new_remote()`, run migrations, build router with CORS layer, bind to `0.0.0.0:8080`, start server with `axum::serve()`
- [X] T003 [P] Create `crates/intrada-api/src/state.rs` with `AppState` struct holding `libsql::Database` and `allowed_origin: String`
- [X] T004 [P] Create `crates/intrada-api/src/error.rs` with `ApiError` enum (Validation, NotFound, Internal variants), implement `IntoResponse` returning JSON `{ "error": message }` with appropriate status codes (400, 404, 500), implement `From<LibraryError>` for ApiError, implement `From<libsql::Error>` for ApiError
- [X] T005 [P] Create `Dockerfile` at workspace root with multi-stage build: cargo-chef planner → cargo-chef cook (release) → build intrada-api binary → debian:bookworm-slim runtime with ca-certificates
- [X] T006 [P] Create `fly.toml` at workspace root with app name "intrada-api", primary_region "lhr", internal_port 8080, force_https, auto_stop/auto_start machines, health check at `/api/health`

**Checkpoint**: `cargo check -p intrada-api` compiles. Dockerfile and fly.toml exist.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Database migrations and route assembly — required before any endpoint can be implemented.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T007 Create `crates/intrada-api/src/migrations.rs` with embedded SQL migrations as a const array: migration 0001 (create pieces table), migration 0002 (create exercises table), migration 0003 (create sessions and setlist_entries tables with index). Use `libsql_migration::content::migrate()` to run each migration sequentially. Export `pub async fn run_migrations(conn: &libsql::Connection)` function. Schema per data-model.md DDL.
- [X] T008 Create `crates/intrada-api/src/routes/mod.rs` with `pub fn api_router(state: AppState) -> Router` that nests health, pieces, exercises, and sessions routers under `/api`. Add CORS layer using `tower_http::cors::CorsLayer` with allowed origin from state, methods GET/POST/PUT/DELETE, and Content-Type header.
- [X] T009 Create `crates/intrada-api/src/db/mod.rs` as module declaration for `pub mod pieces;`, `pub mod exercises;`, `pub mod sessions;`

**Checkpoint**: Server starts, connects to Turso, runs migrations, returns 404 for undefined routes. `cargo check -p intrada-api` passes.

---

## Phase 3: User Story 1 — Library CRUD (Priority: P1) 🎯 MVP

**Goal**: Full CRUD for pieces and exercises via REST API with validation.

**Independent Test**: `curl` commands from quickstart.md V2-V8 all succeed against the running server.

### Implementation for User Story 1

- [X] T010 [P] [US1] Create `crates/intrada-api/src/db/pieces.rs` with async functions: `insert_piece(conn, &CreatePiece) -> Result<Piece, ApiError>` (generate ULID + timestamps, flatten tempo to two columns, serialise tags to JSON), `list_pieces(conn) -> Result<Vec<Piece>, ApiError>` (ORDER BY created_at DESC), `get_piece(conn, id) -> Result<Option<Piece>, ApiError>`, `update_piece(conn, id, &UpdatePiece) -> Result<Option<Piece>, ApiError>` (three-state semantics: fetch current, apply changes, update row), `delete_piece(conn, id) -> Result<bool, ApiError>`. All functions reconstruct Tempo from two columns and deserialise tags from JSON.
- [X] T011 [P] [US1] Create `crates/intrada-api/src/db/exercises.rs` with async functions: `insert_exercise(conn, &CreateExercise) -> Result<Exercise, ApiError>`, `list_exercises(conn) -> Result<Vec<Exercise>, ApiError>` (ORDER BY created_at DESC), `get_exercise(conn, id) -> Result<Option<Exercise>, ApiError>`, `update_exercise(conn, id, &UpdateExercise) -> Result<Option<Exercise>, ApiError>` (three-state semantics), `delete_exercise(conn, id) -> Result<bool, ApiError>`. Same patterns as pieces but with optional composer and category field.
- [X] T012 [US1] Create `crates/intrada-api/src/routes/pieces.rs` with router and handlers: `list_pieces` (GET /), `get_piece` (GET /{id}), `create_piece` (POST / — validate with `intrada_core::validation::validate_create_piece`, return 201), `update_piece` (PUT /{id} — validate with `validate_update_piece`, return 200 or 404), `delete_piece` (DELETE /{id} — return 200 or 404). Extract `State<AppState>` and call `state.db.connect()` per request.
- [X] T013 [US1] Create `crates/intrada-api/src/routes/exercises.rs` with router and handlers: `list_exercises` (GET /), `get_exercise` (GET /{id}), `create_exercise` (POST / — validate with `validate_create_exercise`, return 201), `update_exercise` (PUT /{id} — validate with `validate_update_exercise`, return 200 or 404), `delete_exercise` (DELETE /{id} — return 200 or 404). Same pattern as pieces routes.

**Checkpoint**: Pieces and exercises CRUD fully functional. quickstart.md V2-V8 pass. Validation errors return 400 with descriptive messages.

---

## Phase 4: User Story 2 — Practice Sessions (Priority: P2)

**Goal**: Save, list, get, and delete completed practice sessions with setlist entries.

**Independent Test**: `curl` commands from quickstart.md V9-V10 succeed. Sessions store and return all entries with correct data.

### Implementation for User Story 2

- [X] T014 [US2] Create `crates/intrada-api/src/db/sessions.rs` with async functions: `insert_session(conn, &SaveSessionRequest) -> Result<PracticeSession, ApiError>` (generate session ULID, insert session row, insert all setlist_entry rows in a transaction), `list_sessions(conn) -> Result<Vec<PracticeSession>, ApiError>` (ORDER BY completed_at DESC, join entries per session), `get_session(conn, id) -> Result<Option<PracticeSession>, ApiError>` (fetch session + entries), `delete_session(conn, id) -> Result<bool, ApiError>` (CASCADE deletes entries). Define `SaveSessionRequest` struct for the incoming JSON (same as PracticeSession but without `id`).
- [X] T015 [US2] Create `crates/intrada-api/src/routes/sessions.rs` with router and handlers: `list_sessions` (GET /), `get_session` (GET /{id}), `save_session` (POST / — validate session_notes and entry_notes lengths, validate setlist not empty, return 201), `delete_session` (DELETE /{id} — return 200 or 404). Use validation functions from `intrada_core::validation` (validate_session_notes, validate_entry_notes, validate_setlist_not_empty).

**Checkpoint**: Sessions CRUD functional. quickstart.md V9-V10 pass. Sessions store entries with positions, durations, statuses, notes.

---

## Phase 5: User Story 3 — Cross-Origin Access (Priority: P3)

**Goal**: CORS headers allow the Cloudflare Workers frontend to make API requests.

**Independent Test**: `curl` OPTIONS preflight from quickstart.md V11 returns correct CORS headers.

### Implementation for User Story 3

- [X] T016 [US3] Verify and refine CORS configuration in `crates/intrada-api/src/routes/mod.rs`: ensure CorsLayer uses the `ALLOWED_ORIGIN` env var as the allowed origin (parsed to `HeaderValue`), allows methods GET/POST/PUT/DELETE, allows Content-Type header, and handles preflight OPTIONS requests automatically. Test with curl OPTIONS request per quickstart.md V11. If CORS was already configured correctly in T008, this task verifies and documents the configuration.

**Checkpoint**: Preflight requests from the frontend origin succeed. Requests from unknown origins are rejected.

---

## Phase 6: User Story 4 — Health & Readiness (Priority: P4)

**Goal**: Health check endpoint reports server and database status for Fly.io monitoring.

**Independent Test**: `curl` from quickstart.md V1 returns `{ "status": "ok", "database": "ok" }`.

### Implementation for User Story 4

- [X] T017 [US4] Create `crates/intrada-api/src/routes/health.rs` with health check handler: execute `SELECT 1` query against the database via `state.db.connect()`. On success return 200 with `{ "status": "ok", "database": "ok" }`. On failure return 503 with `{ "status": "degraded", "database": "error" }`. Register at `/api/health` in the router.

**Checkpoint**: Health endpoint responds correctly. Fly.io health checks will monitor this path.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Verification, deployment, and cleanup.

- [X] T018 Run `cargo test` to confirm all existing tests still pass (intrada-core + intrada-web unmodified)
- [X] T019 Run `cargo clippy -- -D warnings` to confirm zero warnings across the entire workspace
- [X] T020 Run `cargo fmt --all -- --check` to confirm no formatting issues
- [ ] T021 Run quickstart.md verification steps V1-V12 against a locally running server
- [ ] T022 Deploy to Fly.io: run `fly deploy`, set secrets (`TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`, `ALLOWED_ORIGIN`), verify health check at production URL per quickstart.md V13

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on T001 (Cargo.toml must exist for compilation)
- **User Story 1 (Phase 3)**: Depends on Phase 2 (migrations + router must exist)
- **User Story 2 (Phase 4)**: Depends on Phase 2 (migrations + router must exist). Independent of US1.
- **User Story 3 (Phase 5)**: CORS is configured in T008 (Phase 2). T016 is verification only.
- **User Story 4 (Phase 6)**: Depends on Phase 2 (router must exist). Independent of US1/US2.
- **Polish (Phase 7)**: Depends on all user stories being complete

### Task Dependencies

```
T001 (Cargo.toml) ──→ T002, T003, T004 (parallel: main, state, error)
                  ──→ T005, T006 (parallel: Dockerfile, fly.toml)
                  ──→ T007, T008, T009 (Phase 2: migrations, router, db mod)
                           │
                           ▼
              ┌────────────┼────────────┬──────────┐
              ▼            ▼            ▼          ▼
         T010, T011   T014 (db/     T016       T017
         (db pieces,  sessions)   (CORS       (health)
          exercises)     │        verify)
              │          ▼
              ▼        T015
         T012, T013   (routes/
         (routes/     sessions)
          pieces,
          exercises)
              │          │
              ▼          ▼
         T018-T022 (verification + deploy)
```

### Parallel Opportunities

- T002, T003, T004, T005, T006 can all run in parallel (different files, no dependencies on each other)
- T010 and T011 can run in parallel (different files, same pattern)
- T012 and T013 are sequential after T010/T011 respectively but can be parallelised if both DB modules are done
- T014 (db/sessions) is independent of T010/T011 and can run in parallel with them
- T016 (CORS verify) and T017 (health) are independent of each other and of US1/US2

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T006)
2. Complete Phase 2: Foundational (T007-T009)
3. Complete Phase 3: User Story 1 — Library CRUD (T010-T013)
4. **STOP and VALIDATE**: Run quickstart.md V1-V8 with curl
5. This delivers a working API for pieces and exercises

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add User Story 1 (Library CRUD) → Test independently → **MVP!**
3. Add User Story 2 (Practice Sessions) → Test independently
4. Add User Story 3 (CORS verify) → Test independently
5. Add User Story 4 (Health check) → Test independently
6. Polish → Deploy to Fly.io

### Recommended Execution Order (Solo Developer)

Since this is a solo project, execute sequentially in priority order:
1. T001 → T002-T006 (parallel setup)
2. T007-T009 (foundational)
3. T010-T011 (parallel DB modules) → T012-T013 (routes)
4. T014 → T015 (sessions)
5. T016 (CORS verify)
6. T017 (health)
7. T018-T022 (verification + deploy)

---

## Notes

- Total implementation tasks: 17 (T001-T017)
- Total verification/deploy tasks: 5 (T018-T022)
- User Story 1: 4 tasks (T010-T013) — pieces + exercises CRUD
- User Story 2: 2 tasks (T014-T015) — sessions save/list/get/delete
- User Story 3: 1 task (T016) — CORS verification
- User Story 4: 1 task (T017) — health endpoint
- Setup: 6 tasks (T001-T006)
- Foundational: 3 tasks (T007-T009)
- Parallel opportunities: 8 tasks can run in parallel with others
- All validation reuses `intrada_core::validation` — no duplication
- The `intrada-core` and `intrada-web` crates are NOT modified
- DB functions use `state.db.connect()` per request (lightweight, no pool needed)
- Three-state update semantics: fetch current row, apply non-None fields, write back
