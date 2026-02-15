# Tasks: Shuttle API Server & Database

**Input**: Design documents from `/specs/017-shuttle-api-deploy/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the `intrada-api` crate, configure Shuttle, and set up the workspace

- [ ] T001 Create `crates/intrada-api/` directory and `crates/intrada-api/Cargo.toml` with dependencies: shuttle-runtime 0.57, shuttle-axum 0.57, shuttle-shared-db 0.57 (postgres, sqlx), axum 0.8, sqlx 0.8 (runtime-tokio, postgres, chrono), tower-http 0.6 (fs), tokio 1 (macros), serde (workspace), serde_json (workspace), chrono (workspace), ulid (workspace), tracing 0.1, tracing-subscriber 0.3 (env-filter), intrada-core (path = "../intrada-core")
- [ ] T002 Create Shuttle entry point in `crates/intrada-api/src/main.rs` with `#[shuttle_runtime::main]` function accepting `#[shuttle_shared_db::Postgres] pool: PgPool`, running migrations via `sqlx::migrate!()`, and returning the Axum router wrapped in `shuttle_axum::ShuttleAxum`
- [ ] T003 Create `Shuttle.toml` at workspace root with `assets = ["crates/intrada-web/dist/*"]` and `crate = "crates/intrada-api"`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Create `crates/intrada-api/src/state.rs` with `AppState` struct wrapping `sqlx::PgPool` (implements `Clone`) and constructor
- [ ] T005 [P] Create `crates/intrada-api/src/error.rs` with `ApiError` enum (Validation, NotFound, Internal) implementing `axum::response::IntoResponse` — Validation→400 `{"error":"..."}`, NotFound→404 `{"error":"..."}`, Internal→500 `{"error":"Internal server error"}`; implement `From<intrada_core::LibraryError>` for ApiError
- [ ] T006 [P] Create SQL migration `crates/intrada-api/migrations/001_create_pieces.sql` per data-model.md (pieces table with TEXT PK, TEXT NOT NULL title/composer, nullable key/tempo_marking/notes, SMALLINT tempo_bpm with CHECK 1-400, TEXT[] tags DEFAULT '{}', TIMESTAMPTZ created_at/updated_at)
- [ ] T007 [P] Create SQL migration `crates/intrada-api/migrations/002_create_exercises.sql` per data-model.md (exercises table with TEXT PK, TEXT NOT NULL title, nullable composer/category/key/tempo_marking/notes, SMALLINT tempo_bpm with CHECK, TEXT[] tags DEFAULT '{}', TIMESTAMPTZ created_at/updated_at)
- [ ] T008 [P] Create SQL migration `crates/intrada-api/migrations/003_create_sessions.sql` per data-model.md (practice_sessions table + setlist_entries table with FK ON DELETE CASCADE, CHECK constraints on completion_status/item_type/status, index on session_id)
- [ ] T009 Create `crates/intrada-api/src/routes/mod.rs` with `api_router()` function that builds the nested `/api` router with health, pieces, exercises, and sessions sub-routers; create `crates/intrada-api/src/routes/health.rs` with `GET /health` handler returning `{"status":"ok"}`
- [ ] T010 Create `crates/intrada-api/src/db/mod.rs` declaring the pieces, exercises, and sessions submodules

**Checkpoint**: Foundation ready — migrations, error handling, routing skeleton, and app state in place

---

## Phase 3: User Story 1 — Server-Persisted Library (Priority: P1) MVP

**Goal**: Full CRUD for pieces and exercises via REST API, persisted in Postgres

**Independent Test**: Launch server, POST a piece via curl, GET /api/pieces to confirm it persists. Clear browser, reload — piece still present.

### Implementation for User Story 1

- [ ] T011 [P] [US1] Create `crates/intrada-api/src/db/pieces.rs` with functions: `insert_piece(pool, &CreatePiece) -> Piece`, `list_pieces(pool) -> Vec<Piece>`, `get_piece(pool, id) -> Option<Piece>`, `update_piece(pool, id, &UpdatePiece) -> Option<Piece>`, `delete_piece(pool, id) -> bool` — flatten Tempo to tempo_marking/tempo_bpm columns, handle TEXT[] tags, generate ULID + timestamps server-side
- [ ] T012 [P] [US1] Create `crates/intrada-api/src/db/exercises.rs` with functions: `insert_exercise(pool, &CreateExercise) -> Exercise`, `list_exercises(pool) -> Vec<Exercise>`, `get_exercise(pool, id) -> Option<Exercise>`, `update_exercise(pool, id, &UpdateExercise) -> Option<Exercise>`, `delete_exercise(pool, id) -> bool` — same Tempo/tags pattern as pieces
- [ ] T013 [US1] Create `crates/intrada-api/src/routes/pieces.rs` with handlers: `list_pieces` (GET /pieces), `get_piece` (GET /pieces/{id}), `create_piece` (POST /pieces — validate via `intrada_core::validation::validate_create_piece`, return 201), `update_piece` (PUT /pieces/{id} — validate via `validate_update_piece`, return 200), `delete_piece` (DELETE /pieces/{id} — return 200 or 404); register routes in `routes/mod.rs`
- [ ] T014 [US1] Create `crates/intrada-api/src/routes/exercises.rs` with handlers: `list_exercises` (GET /exercises), `get_exercise` (GET /exercises/{id}), `create_exercise` (POST /exercises — validate via `validate_create_exercise`, return 201), `update_exercise` (PUT /exercises/{id} — validate via `validate_update_exercise`, return 200), `delete_exercise` (DELETE /exercises/{id} — return 200 or 404); register routes in `routes/mod.rs`
- [ ] T015 [US1] Wire pieces and exercises routers into `api_router()` in `crates/intrada-api/src/routes/mod.rs` and verify the full CRUD flow compiles with `cargo check -p intrada-api`

**Checkpoint**: Pieces and exercises CRUD endpoints functional. Test with curl/quickstart script against `cargo shuttle run`.

---

## Phase 4: User Story 2 — Server-Persisted Practice Sessions (Priority: P2)

**Goal**: Create and read practice sessions via REST API (immutable once saved — no update/delete per FR-005)

**Independent Test**: POST a session with entries via curl, GET /api/sessions to confirm it persists with all entries and correct ordering.

### Implementation for User Story 2

- [ ] T016 [P] [US2] Create `crates/intrada-api/src/db/sessions.rs` with functions: `insert_session(pool, &PracticeSession) -> PracticeSession` (inserts session + entries in a transaction, validates `completion_status`/`item_type`/`status` enums), `list_sessions(pool) -> Vec<PracticeSession>` (joins setlist_entries ordered by position), `get_session(pool, id) -> Option<PracticeSession>` (same join pattern)
- [ ] T017 [US2] Create `crates/intrada-api/src/routes/sessions.rs` with handlers: `list_sessions` (GET /sessions), `get_session` (GET /sessions/{id}), `create_session` (POST /sessions — validate setlist not empty via `intrada_core::validation::validate_setlist_not_empty`, validate session_notes/entry_notes lengths, return 201); register routes in `routes/mod.rs`
- [ ] T018 [US2] Wire sessions router into `api_router()` in `crates/intrada-api/src/routes/mod.rs` and verify full session create/read flow compiles

**Checkpoint**: Sessions endpoint functional. Test with curl: create a session with entries, list sessions, get by ID.

---

## Phase 5: User Story 3 — Static WASM Hosting (Priority: P3)

**Goal**: Serve the compiled WASM app as static files from the Axum server with SPA fallback routing

**Independent Test**: Build WASM with trunk, run `cargo shuttle run`, navigate to `http://localhost:8000/` — app loads. Refresh on `/library` — app still loads (SPA fallback works).

### Implementation for User Story 3

- [ ] T019 [US3] Add static file serving to `crates/intrada-api/src/main.rs` using `tower_http::services::{ServeDir, ServeFile}` — configure the Axum router with `.fallback_service(ServeDir::new("dist").fallback(ServeFile::new("dist/index.html")))` so that API routes match first and all other paths return the SPA shell; ensure the `dist/` path resolves correctly for both local dev (`crates/intrada-web/dist/`) and Shuttle deployment (from `Shuttle.toml` assets)
- [ ] T020 [US3] Verify SPA routing works: build WASM with `trunk build` in `crates/intrada-web/`, copy `dist/` to `crates/intrada-api/dist/`, run `cargo shuttle run`, and confirm root URL loads the app and deep links like `/library` return the app shell

**Checkpoint**: Full app accessible at localhost:8000 — API endpoints under /api/, WASM app at all other paths.

---

## Phase 6: User Story 4 — Local Cache for Fast Loads (Priority: P4)

**Goal**: Web shell uses API as primary data source with localStorage as read cache; cached data renders instantly, background fetch updates UI

**Independent Test**: Load the app with populated localStorage on a throttled connection — cached data appears instantly. Clear localStorage, reload — data fetches from server. Disconnect server — cached data shown with stale warning.

### Implementation for User Story 4

- [ ] T021 [US4] Add HTTP client dependency to `crates/intrada-web/Cargo.toml`: `gloo-net` (for WASM-compatible fetch); add `wasm-bindgen-futures` if not already present for spawning async tasks
- [ ] T022 [US4] Create `crates/intrada-web/src/api_client.rs` with async functions for all API operations: `fetch_pieces() -> Result<Vec<Piece>>`, `fetch_exercises() -> Result<Vec<Exercise>>`, `create_piece(&CreatePiece) -> Result<Piece>`, `update_piece(id, &UpdatePiece) -> Result<Piece>`, `delete_piece(id) -> Result<()>`, `create_exercise(&CreateExercise) -> Result<Exercise>`, `update_exercise(id, &UpdateExercise) -> Result<Exercise>`, `delete_exercise(id) -> Result<()>`, `fetch_sessions() -> Result<Vec<PracticeSession>>`, `save_session(&PracticeSession) -> Result<PracticeSession>` — use relative URLs (`/api/pieces`, etc.) since same-origin
- [ ] T023 [US4] Modify `crates/intrada-web/src/core_bridge.rs` to implement the cache-first data flow: on `StorageEffect::LoadAll` → load from localStorage first (existing code) → fire `Event::DataLoaded` → spawn async background fetch from API → on API success, update thread-local state + localStorage cache + fire `Event::DataLoaded` again with fresh data; on `StorageEffect::LoadSessions` → same pattern for sessions
- [ ] T024 [US4] Modify `crates/intrada-web/src/core_bridge.rs` to route write operations through API: on `StorageEffect::SavePiece` → call `api_client::create_piece()` → on success, update thread-local + localStorage; on `StorageEffect::UpdatePiece` → call `api_client::update_piece()` → update cache; on `StorageEffect::DeleteItem` → call `api_client::delete_piece()` or `delete_exercise()` → update cache; on `StorageEffect::SaveExercise` / `UpdateExercise` → same pattern; on `StorageEffect::SavePracticeSession` → call `api_client::save_session()`; on `StorageEffect::DeletePracticeSession` → keep localStorage-only deletion (no API endpoint per FR-005, sessions are immutable server-side) and log a console warning
- [ ] T025 [US4] Add error handling to `crates/intrada-web/src/core_bridge.rs` for API failures: on network/server error during writes, log to console and fire an error event to the Crux core (or show a user-visible error via `web_sys::window().alert()`); on background fetch failure, keep cached data and log a warning; ensure in-progress data is never lost on API failure (FR-013)
- [ ] T026 [US4] Remove stub data seeding from `crates/intrada-web/src/core_bridge.rs` — on first load with empty localStorage AND empty API response, show an empty library (no more `create_stub_data()` call); keep `crates/intrada-web/src/data.rs` for now but remove the call site

**Checkpoint**: Web app loads cached data instantly, fetches from API in background, writes go through API first. Clear localStorage → data reloads from server. API down → cached data shown.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: CI/CD, testing, and deployment validation

- [ ] T027 [P] Update `.github/workflows/ci.yml` to add a `deploy` job that: depends on `test`, `clippy`, `wasm-build`, and `e2e`; runs only on push to `main`; uses `shuttle-hq/deploy-action@v2` with `SHUTTLE_API_KEY` secret; set working directory to `crates/intrada-api`
- [ ] T028 [P] Update `.github/workflows/ci.yml` to ensure the `test` and `clippy` jobs build and test `intrada-api` (already covered by workspace-level `cargo test` and `cargo clippy`)
- [ ] T029 Run `cargo test` in workspace root — all existing tests (142+) must pass plus any new API crate compilation checks
- [ ] T030 Run `cargo clippy -- -D warnings` — zero warnings across all crates including intrada-api
- [ ] T031 Update E2E test infrastructure in `e2e/` to run against the API-backed app: configure Playwright to launch `cargo shuttle run` as the server (or point at the running instance), ensure the 14 existing E2E tests pass against the API server serving the WASM app (SC-007)
- [ ] T032 Run the quickstart.md verification checklist (V1–V9): start server with `cargo shuttle run`, test health endpoint, test piece CRUD, test exercise CRUD, test session persistence, verify WASM app loads, verify SPA routing, verify cache behaviour
- [ ] T033 Update `CLAUDE.md` by running `.specify/scripts/bash/update-agent-context.sh claude`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 Library CRUD (Phase 3)**: Depends on Foundational (Phase 2)
- **US2 Sessions (Phase 4)**: Depends on Foundational (Phase 2), can run in parallel with US1
- **US3 WASM Hosting (Phase 5)**: Depends on Foundational (Phase 2), can run in parallel with US1/US2
- **US4 Local Cache (Phase 6)**: Depends on US1 and US2 (needs API endpoints to exist before web shell can call them)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Phase 2 — no dependencies on other stories
- **US2 (P2)**: Can start after Phase 2 — no dependencies on other stories (can be parallel with US1)
- **US3 (P3)**: Can start after Phase 2 — no dependencies on other stories (can be parallel with US1/US2)
- **US4 (P4)**: Depends on US1 + US2 (needs API endpoints to exist for the web shell to call)

### Within Each User Story

- DB layer before route handlers (models before services)
- Route handlers wire into router after DB functions exist
- Compile check after wiring

### Parallel Opportunities

- T005, T006, T007, T008 can all run in parallel (different files, no dependencies)
- T011 and T012 can run in parallel (pieces DB and exercises DB are independent files)
- T016 can run in parallel with US1 tasks (sessions DB is a separate file)
- T027 and T028 can run in parallel (CI config changes)

---

## Parallel Example: Phase 2 Foundation

```bash
# Launch all migration files + error module in parallel:
Task: "Create error.rs in crates/intrada-api/src/error.rs"
Task: "Create 001_create_pieces.sql migration"
Task: "Create 002_create_exercises.sql migration"
Task: "Create 003_create_sessions.sql migration"
```

## Parallel Example: User Story 1

```bash
# Launch both DB modules in parallel:
Task: "Create pieces DB layer in crates/intrada-api/src/db/pieces.rs"
Task: "Create exercises DB layer in crates/intrada-api/src/db/exercises.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T010)
3. Complete Phase 3: User Story 1 — Library CRUD (T011–T015)
4. **STOP and VALIDATE**: Test piece/exercise CRUD with curl
5. Can demo API-only (no web shell changes yet)

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 (Library CRUD) → Test with curl → MVP!
3. Add US2 (Sessions) → Test with curl → Sessions work
4. Add US3 (WASM Hosting) → App accessible at URL
5. Add US4 (Local Cache) → Full integrated experience
6. Polish → CI/CD, deploy, final validation

### Task Summary

- **Total tasks**: 33
- **Phase 1 (Setup)**: 3 tasks
- **Phase 2 (Foundational)**: 7 tasks (4 parallelizable)
- **Phase 3 / US1 (Library CRUD)**: 5 tasks (2 parallelizable)
- **Phase 4 / US2 (Sessions)**: 3 tasks (1 parallelizable)
- **Phase 5 / US3 (WASM Hosting)**: 2 tasks
- **Phase 6 / US4 (Local Cache)**: 6 tasks
- **Phase 7 (Polish)**: 7 tasks (2 parallelizable)
- **Suggested MVP scope**: Phases 1–3 (T001–T015) = 15 tasks

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- intrada-core remains completely unchanged (FR-015)
- No auth — all endpoints are public (FR-014)
- Session in-progress (`intrada:session-in-progress`) stays localStorage-only (not sent to API)
