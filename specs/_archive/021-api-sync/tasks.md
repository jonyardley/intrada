# Tasks: API Sync

**Input**: Design documents from `/specs/021-api-sync/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/api-client.md, quickstart.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Add new dependencies and create the API client module infrastructure

- [ ] T001 Add `gloo-net`, `wasm-bindgen-futures`, and `serde` dependencies to `crates/intrada-web/Cargo.toml`
- [ ] T002 Create `crates/intrada-web/src/api_client.rs` with `ApiError` enum, `API_BASE_URL` const (via `option_env!("INTRADA_API_URL")` with `https://intrada-api.fly.dev` fallback), and helper function for building endpoint URLs
- [ ] T003 Register the `api_client` module in `crates/intrada-web/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the API client functions and loading infrastructure that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Implement `fetch_pieces()` and `fetch_exercises()` async functions in `crates/intrada-web/src/api_client.rs` — `GET /api/pieces` and `GET /api/exercises`, returning `Result<Vec<Piece>, ApiError>` and `Result<Vec<Exercise>, ApiError>`
- [ ] T005 [P] Implement `create_piece()`, `update_piece()`, `delete_piece()` async functions in `crates/intrada-web/src/api_client.rs` — `POST`, `PUT /api/pieces/{id}`, `DELETE /api/pieces/{id}`
- [ ] T006 [P] Implement `create_exercise()`, `update_exercise()`, `delete_exercise()` async functions in `crates/intrada-web/src/api_client.rs` — `POST`, `PUT /api/exercises/{id}`, `DELETE /api/exercises/{id}`
- [ ] T007 [P] Implement `fetch_sessions()`, `create_session()`, `delete_session()` async functions in `crates/intrada-web/src/api_client.rs` — `GET /api/sessions`, `POST /api/sessions`, `DELETE /api/sessions/{id}`
- [ ] T008 Implement `ApiError`-to-user-message conversion in `crates/intrada-web/src/api_client.rs` — map network errors, HTTP 400/404/500 status codes, and deserialisation failures to user-friendly strings per research.md R5
- [ ] T009 Add `is_loading: RwSignal<bool>` loading signal to `crates/intrada-web/src/types.rs` and provide it via Leptos context in `crates/intrada-web/src/app.rs` alongside `SharedCore` and `view_model`

**Checkpoint**: API client module complete with all typed HTTP functions and error handling. Loading signal infrastructure ready.

---

## Phase 3: User Story 1 — Library Persists to Server (Priority: P1) 🎯 MVP

**Goal**: Pieces and exercises are fetched from the API on load, and all create/update/delete operations go through the API instead of localStorage.

**Independent Test**: Open the app in one browser, add a piece. Open in a different browser — the piece appears. Delete it — it disappears from both.

### Implementation for User Story 1

- [ ] T010 [US1] Update `process_effects()` `LoadAll` handler in `crates/intrada-web/src/core_bridge.rs` — replace synchronous `load_library_data()` with `spawn_local()` that calls `fetch_pieces()` + `fetch_exercises()`, sets `is_loading` to true before and false after, and dispatches `Event::DataLoaded` on success or `Event::LoadFailed` on error
- [ ] T011 [US1] Update `process_effects()` `SavePiece` handler in `crates/intrada-web/src/core_bridge.rs` — replace localStorage write with `spawn_local()` that calls `create_piece()`, then re-fetches all pieces/exercises (refresh-after-mutate) and dispatches `Event::DataLoaded`; on error dispatches `Event::LoadFailed`
- [ ] T012 [US1] Update `process_effects()` `SaveExercise` handler in `crates/intrada-web/src/core_bridge.rs` — same pattern as T011 but using `create_exercise()`
- [ ] T013 [US1] Update `process_effects()` `UpdatePiece` and `UpdateExercise` handlers in `crates/intrada-web/src/core_bridge.rs` — replace localStorage update with `spawn_local()` calling `update_piece()`/`update_exercise()`, then refresh-after-mutate
- [ ] T014 [US1] Update `process_effects()` `DeleteItem` handler in `crates/intrada-web/src/core_bridge.rs` — replace localStorage delete with `spawn_local()` that tries `delete_piece()` and `delete_exercise()` (item type unknown at this level), then refresh-after-mutate
- [ ] T015 [US1] Update `App()` initialisation in `crates/intrada-web/src/app.rs` — replace synchronous `load_library_data()` call with dispatching `Event::Piece(PieceEvent::Init)` or simply calling `core.process_event(Event::DataLoaded { pieces: vec![], exercises: vec![] })` initially and letting `LoadAll` effect handle async fetch; remove `create_stub_data()` import and seed logic (FR-009)
- [ ] T016 [US1] Remove or empty `crates/intrada-web/src/data.rs` — delete `create_stub_data()` function and stub data constants; update `crates/intrada-web/src/lib.rs` to remove the `data` module if emptied
- [ ] T017 [US1] Remove thread_local `LIBRARY: RefCell<LibraryData>` from `crates/intrada-web/src/core_bridge.rs` and the `load_library_data()`, `save_to_local_storage()` functions — library data now comes from the API, not localStorage (keep session-in-progress localStorage functions per FR-008)
- [ ] T018 [US1] Add a loading spinner/indicator component to `crates/intrada-web/src/components/` (e.g., `loading_indicator.rs`) — a simple full-page or inline spinner that reads `is_loading` from context and displays when true (FR-007)
- [ ] T019 [US1] Wire loading indicator into `crates/intrada-web/src/views/library_list.rs` — show spinner during initial data load; show empty state (not stub data) when API returns empty library (FR-009)
- [ ] T020 [US1] Add submit-in-progress state to `crates/intrada-web/src/views/add_form.rs` — disable the submit button and show loading indicator while the API request is in-flight to prevent duplicate submissions (FR-010)
- [ ] T021 [US1] Add submit-in-progress state to `crates/intrada-web/src/views/edit_form.rs` — same pattern as T020 for the edit form
- [ ] T022 [US1] Add delete-in-progress state to `crates/intrada-web/src/views/detail.rs` — disable the delete button while the API delete request is in-flight (FR-010)
- [ ] T023 [US1] Verify `cargo test` passes in `crates/intrada-core` (no core changes), `cargo clippy -- -D warnings` clean, and WASM build succeeds with `trunk build` in `crates/intrada-web`

**Checkpoint**: Library CRUD operations work through the API. Pieces and exercises persist to the server and are visible from any browser.

---

## Phase 4: User Story 2 — Practice Sessions Persist to Server (Priority: P2)

**Goal**: Completed practice sessions are saved to and fetched from the API. Session-in-progress crash recovery remains in localStorage.

**Independent Test**: Complete a practice session in one browser. Open the app in another browser — the session appears in practice history.

### Implementation for User Story 2

- [ ] T024 [US2] Update `process_effects()` `LoadSessions` handler in `crates/intrada-web/src/core_bridge.rs` — replace `load_sessions_data()` with `spawn_local()` calling `fetch_sessions()`, dispatching `Event::SessionsLoaded` on success or `Event::LoadFailed` on error
- [ ] T025 [US2] Update `process_effects()` `SavePracticeSession` handler in `crates/intrada-web/src/core_bridge.rs` — replace localStorage write with `spawn_local()` calling `create_session()`, then refresh-after-mutate with `fetch_sessions()` and `Event::SessionsLoaded`; keep `clear_session_in_progress()` call (FR-008)
- [ ] T026 [US2] Update `process_effects()` `DeletePracticeSession` handler in `crates/intrada-web/src/core_bridge.rs` — replace localStorage delete with `spawn_local()` calling `delete_session()`, then refresh-after-mutate
- [ ] T027 [US2] Remove thread_local `SESSIONS: RefCell<SessionsData>` and `load_sessions_from_local_storage()`, `save_sessions_to_local_storage()` functions from `crates/intrada-web/src/core_bridge.rs` — sessions data now comes from the API (keep `save_session_in_progress`, `clear_session_in_progress`, `load_session_in_progress` for crash recovery per FR-008)
- [ ] T028 [US2] Update `App()` initialisation in `crates/intrada-web/src/app.rs` — replace synchronous `load_sessions_data()` with letting `LoadSessions` effect handle async fetch; keep `load_session_in_progress()` crash recovery from localStorage (FR-008)
- [ ] T029 [US2] Wire loading indicator into `crates/intrada-web/src/views/sessions.rs` — show spinner during sessions fetch, show empty state when no sessions exist
- [ ] T030 [US2] Verify session-in-progress crash recovery still works via localStorage — `SaveSessionInProgress` and `ClearSessionInProgress` handlers in `process_effects()` remain unchanged (FR-008)

**Checkpoint**: Completed sessions persist to the server. In-progress crash recovery still uses localStorage. Sessions visible across browsers.

---

## Phase 5: User Story 3 — Graceful Error Handling (Priority: P3)

**Goal**: When the API is unreachable or returns errors, the app shows clear, user-friendly feedback instead of failing silently.

**Independent Test**: Disconnect from the network, try to add a piece — see an error message. Point at a bad API URL, load the app — see an error that data could not be loaded.

### Implementation for User Story 3

- [ ] T031 [US3] Add a global error banner/toast component to `crates/intrada-web/src/components/` (e.g., `error_banner.rs`) — reads `ViewModel.error` from context and displays a dismissible error message with user-friendly text; include a "dismiss" button that dispatches `Event::ClearError` (FR-006)
- [ ] T032 [US3] Wire the error banner into the main layout in `crates/intrada-web/src/app.rs` — show it above the `<main>` content area so errors are visible on any page
- [ ] T033 [US3] Handle initial load failure in `crates/intrada-web/src/views/library_list.rs` — when `ViewModel.error` is set and items list is empty, show a clear "Could not load library" message with the error details instead of just an empty state (FR-006 acceptance scenario 3)
- [ ] T034 [US3] Handle validation errors from the API in form views (`crates/intrada-web/src/views/add_form.rs` and `crates/intrada-web/src/views/edit_form.rs`) — when `Event::LoadFailed` is dispatched with a validation error (HTTP 400), display the server's error message in the form context (FR-006 acceptance scenario 2)
- [ ] T035 [US3] Ensure all `spawn_local()` async handlers in `crates/intrada-web/src/core_bridge.rs` properly catch and report errors — network failures, server errors, and deserialisation errors all map to `Event::LoadFailed` with user-friendly messages per api_client `ApiError` conversion (FR-006 acceptance scenario 1)

**Checkpoint**: All error scenarios show user-friendly messages. No silent failures, no crashes on network errors.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup, CI/CD configuration, and full verification

- [ ] T036 [P] Set `INTRADA_API_URL` environment variable in `.github/workflows/deploy.yml` for production builds — add `env: INTRADA_API_URL: https://intrada-api.fly.dev` to the Trunk build step
- [ ] T037 [P] Clean up unused imports and dead code across all modified files in `crates/intrada-web/src/` — remove any remaining localStorage-only imports (e.g., `load_from_local_storage`, `STORAGE_KEY`, `SESSIONS_KEY` if no longer used), run `cargo clippy -- -D warnings`
- [ ] T038 Run full verification: `cargo test` (all workspace tests pass), `cargo clippy -- -D warnings` (zero warnings), `cargo fmt --check` (formatted), `trunk build` in `crates/intrada-web/` (WASM builds successfully)
- [ ] T039 Run quickstart.md verification steps V1–V10 against the running API server (local or Fly.io) to confirm all acceptance scenarios pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Phase 2 — can start after foundational API client is built
- **User Story 2 (Phase 4)**: Depends on Phase 2 — can run in parallel with US1 but recommended after US1 since sessions reference library items
- **User Story 3 (Phase 5)**: Depends on Phase 2 — builds on error infrastructure from US1/US2 so recommended after both
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Requires Phase 2 foundational API client. No other story dependencies. **This is the MVP.**
- **User Story 2 (P2)**: Requires Phase 2 foundational API client. Logically follows US1 (sessions reference library items that must exist on server) but can be implemented independently.
- **User Story 3 (P3)**: Requires Phase 2 foundational API client. Enhances error handling already established in US1/US2 — best done after US1 and US2 are functional.

### Within Each Phase

- Setup: T001 → T002 → T003 (sequential, each depends on prior)
- Foundational: T004 first (fetch functions used everywhere), then T005/T006/T007 in parallel, then T008, then T009
- US1: T010 first (LoadAll is the entry point), then T011–T014 can proceed (write handlers), T015–T017 for init cleanup, T018–T022 for UI, T023 verification last
- US2: T024 first, then T025/T026, T027/T028 cleanup, T029/T030 UI and verification
- US3: T031/T032 (error component), then T033/T034/T035 (wiring)
- Polish: T036/T037 in parallel, T038 then T039 sequential

### Parallel Opportunities

**Phase 2** (after T004):
```
Task: T005 "Implement create/update/delete piece functions in api_client.rs"
Task: T006 "Implement create/update/delete exercise functions in api_client.rs"
Task: T007 "Implement session API functions in api_client.rs"
```

**Phase 3** (after T010):
```
Task: T011 "Update SavePiece handler in core_bridge.rs"
Task: T012 "Update SaveExercise handler in core_bridge.rs"
Task: T013 "Update UpdatePiece/UpdateExercise handlers in core_bridge.rs"
Task: T014 "Update DeleteItem handler in core_bridge.rs"
```

**Phase 6**:
```
Task: T036 "Set INTRADA_API_URL in deploy.yml"
Task: T037 "Clean up unused imports and dead code"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T009)
3. Complete Phase 3: User Story 1 (T010–T023)
4. **STOP and VALIDATE**: Library CRUD works via API across browsers
5. Deploy and verify on production

### Incremental Delivery

1. Setup + Foundational → API client ready
2. Add User Story 1 → Library persists to server → Deploy (MVP!)
3. Add User Story 2 → Sessions persist to server → Deploy
4. Add User Story 3 → Error handling polished → Deploy
5. Polish → CI/CD updated, cleanup done → Final deploy

---

## Notes

- **intrada-core is NOT modified** — all changes are in `crates/intrada-web/`
- Session-in-progress crash recovery stays in localStorage (FR-008) — `SaveSessionInProgress`, `ClearSessionInProgress`, `load_session_in_progress` are untouched
- The refresh-after-mutate pattern (re-fetch full list after any write) keeps the UI consistent with server state and handles server-generated IDs
- [P] tasks = different files or independent code paths, no dependencies
- Commit after each task or logical group
- Stop at any checkpoint to validate the story independently
