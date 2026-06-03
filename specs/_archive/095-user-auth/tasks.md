# Tasks: User Authentication

**Input**: Design documents from `/specs/095-user-auth/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/auth-changes.md, quickstart.md

**Tests**: Not explicitly requested in the feature spec. Test tasks are omitted. Existing tests are preserved via auth-optional mode.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add new dependencies and create foundational auth module files

- [x] T001 [P] Add `jsonwebtoken = "9"` and `reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }` to `crates/intrada-api/Cargo.toml`
- [x] T002 [P] Add `js-sys = "0.3"` to `crates/intrada-web/Cargo.toml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Database migrations, auth module, error handling, and CORS — MUST be complete before any user story work

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Add migrations 0013–0018 (user_id columns + indexes on items, sessions, routines) in `crates/intrada-api/src/migrations.rs`
- [x] T004 Add `Unauthorized(String)` variant to `ApiError` enum and implement `IntoResponse` for 401 status in `crates/intrada-api/src/error.rs`
- [x] T005 Create `crates/intrada-api/src/auth.rs` with `AuthConfig` struct (issuer + decoding keys), `fetch_jwks()` async function, `Claims` struct, and `AuthUser` Axum extractor (`FromRequestParts`) that returns `AuthUser("")` when auth is not configured
- [x] T006 Update `AppState` to add `auth_config: Option<AuthConfig>` field and update constructor in `crates/intrada-api/src/state.rs`
- [x] T007 Update `main.rs` to optionally read `CLERK_ISSUER_URL` env var, fetch JWKS on startup if set, and pass `AuthConfig` to `AppState` in `crates/intrada-api/src/main.rs`
- [x] T008 Add `pub mod auth;` to `crates/intrada-api/src/lib.rs`
- [x] T009 Add `header::AUTHORIZATION` to CORS `allow_headers` in `crates/intrada-api/src/routes/mod.rs`
- [x] T010 Verify `cargo test -p intrada-api` passes (auth-optional mode — no CLERK_ISSUER_URL set)

**Checkpoint**: Foundation ready — API server compiles, all existing tests pass, auth infrastructure in place but not yet enforced on handlers

---

## Phase 3: User Story 1 — Sign In to Access Library (Priority: P1) MVP

**Goal**: Unauthenticated users see a sign-in screen; after signing in via Google OAuth, they see their personal library with user-scoped data

**Independent Test**: Sign in with Google account → library loads with user-specific data → create an item → it appears in the library

### API Server: User-Scoped Data

- [x] T011 [P] [US1] Add `user_id: &str` parameter to all functions in `crates/intrada-api/src/db/items.rs` — add `WHERE user_id = ?` to SELECT queries, include `user_id` in INSERT, add `AND user_id = ?` to UPDATE/DELETE
- [x] T012 [P] [US1] Add `user_id: &str` parameter to all functions in `crates/intrada-api/src/db/sessions.rs` — same scoping pattern as items
- [x] T013 [P] [US1] Add `user_id: &str` parameter to all functions in `crates/intrada-api/src/db/routines.rs` — same scoping pattern as items
- [x] T014 [P] [US1] Add `AuthUser(user_id): AuthUser` extractor to all handlers in `crates/intrada-api/src/routes/items.rs` and pass `&user_id` to DB functions
- [x] T015 [P] [US1] Add `AuthUser(user_id): AuthUser` extractor to all handlers in `crates/intrada-api/src/routes/sessions.rs` and pass `&user_id` to DB functions
- [x] T016 [P] [US1] Add `AuthUser(user_id): AuthUser` extractor to all handlers in `crates/intrada-api/src/routes/routines.rs` and pass `&user_id` to DB functions
- [x] T017 [US1] Verify `cargo test -p intrada-api` passes with user-scoped queries (auth-optional mode)

### Web Shell: Clerk Integration

- [x] T018 [US1] Add Clerk JS SDK `<script>` tag and `window.__intrada_auth` helper functions (getToken, isSignedIn, getUserId, signOut) to `crates/intrada-web/index.html`
- [x] T019 [US1] Create `crates/intrada-web/src/clerk_bindings.rs` with `wasm_bindgen(inline_js)` bindings for `get_auth_token()`, `is_signed_in()`, `get_user_id()`, `sign_out()`
- [x] T020 [US1] Add `pub mod clerk_bindings;` to `crates/intrada-web/src/lib.rs`
- [x] T021 [US1] Update `crates/intrada-web/src/api_client.rs` to call `clerk_bindings::get_auth_token()` and attach `Authorization: Bearer <token>` header to all HTTP requests (GET, POST, PUT, DELETE)
- [x] T022 [US1] Create auth gate in `crates/intrada-web/src/app.rs` — wrap existing router content in a `Show` component that checks `is_signed_in()` and shows a sign-in screen (centered layout with Intrada branding + "Sign in with Google" button using glassmorphism styling) when unauthenticated
- [x] T023 [US1] Verify `cargo fmt --check && cargo clippy` pass for both intrada-api and intrada-web

### E2E Test Compatibility

- [x] T024 [US1] Update `e2e/fixtures/api-mock.ts` to inject Clerk mock via `page.addInitScript()` — stub `window.__intrada_auth` with `isSignedIn: () => true`, `getToken: async () => "fake-test-token"`, `getUserId: () => "test-user-001"`, `signOut: async () => {}`; also stub `window.Clerk` and block Clerk CDN script via `page.route()`
- [x] T025 [US1] Verify all existing E2E tests pass with Clerk mock (`npx playwright test`)

**Checkpoint**: User Story 1 complete — unauthenticated users see sign-in screen, authenticated users see user-scoped library, all tests pass

---

## Phase 4: User Story 2 — Data Isolation Between Users (Priority: P1)

**Goal**: Two different users see completely isolated data — User A cannot access User B's items, sessions, or routines

**Independent Test**: Create data as User A, sign in as User B, verify User B's library is empty (or contains only their own data)

- [x] T026 [US2] Verify data isolation is enforced at the DB query level: all SELECT/UPDATE/DELETE queries include `AND user_id = ?` (code review of T011–T013 changes in `crates/intrada-api/src/db/items.rs`, `sessions.rs`, `routines.rs`)
- [x] T027 [US2] Verify that `GET /api/items/{id}` returns 404 (not the item) when accessed by a different user — confirm the get_item query uses `WHERE id = ? AND user_id = ?` in `crates/intrada-api/src/db/items.rs`
- [x] T028 [US2] Verify same isolation for `GET /api/sessions/{id}` and `GET /api/routines/{id}` — confirm get_session and get_routine queries use `WHERE id = ? AND user_id = ?` in `crates/intrada-api/src/db/sessions.rs` and `crates/intrada-api/src/db/routines.rs`

**Checkpoint**: User Story 2 complete — data isolation verified at query level for all entity types

---

## Phase 5: User Story 3 — Sign Out (Priority: P2)

**Goal**: Signed-in users can sign out, returning to the sign-in screen with session invalidated

**Independent Test**: Sign in, click sign out, verify sign-in screen appears and API requests are rejected

- [x] T029 [US3] Add sign-out button to `crates/intrada-web/src/components/app_header.rs` — positioned at trailing end of header, calls `clerk_bindings::sign_out()` on click, styled consistently with existing nav
- [x] T030 [US3] Ensure auth gate in `crates/intrada-web/src/app.rs` re-evaluates authentication state after sign-out and redirects to sign-in screen
- [x] T031 [US3] Verify sign-out flow works in E2E tests — existing tests should still pass since mock always returns `isSignedIn: true`

**Checkpoint**: User Story 3 complete — sign-out button visible, clicking it returns to sign-in screen

---

## Phase 6: User Story 4 — Persistent Session Across Page Reloads (Priority: P2)

**Goal**: Signed-in users remain signed in after page refresh or browser restart

**Independent Test**: Sign in, refresh the page, verify user is still signed in with data visible

- [x] T032 [US4] Verify Clerk JS SDK handles session persistence automatically via its built-in cookie/localStorage mechanism — the auth gate in `crates/intrada-web/src/app.rs` should call `is_signed_in()` on mount and detect the persisted session
- [x] T033 [US4] Handle token expiry gracefully in `crates/intrada-web/src/api_client.rs` — if a 401 response is received, trigger re-authentication by clearing Clerk session state; Clerk's `getToken()` silently refreshes tokens before expiry
- [x] T034 [US4] Verify page reload works in E2E tests — existing tests navigate to pages directly (e.g., `page.goto("/library/new")`) and should still pass with the Clerk mock

**Checkpoint**: User Story 4 complete — session persists across reloads, token refresh is silent

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final verification, code quality, and deployment readiness

- [x] T035 Run full test suite: `cargo test` (all crates), `cargo fmt --check`, `cargo clippy -- -D warnings`
- [x] T036 Run E2E tests: `npx playwright test` — all 30+ tests must pass
- [x] T037 Run quickstart.md verification steps 1–3 (API tests, unit tests, code quality)
- [x] T038 [P] Add `CLERK_PUBLISHABLE_KEY: ${{ secrets.CLERK_PUBLISHABLE_KEY }}` to WASM build env in `.github/workflows/ci.yml`
- [x] T039 Verify CI pipeline passes on push

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 Sign In (Phase 3)**: Depends on Foundational phase completion
- **US2 Data Isolation (Phase 4)**: Depends on US1 (verifies the scoping from US1)
- **US3 Sign Out (Phase 5)**: Depends on US1 (requires auth gate and sign-in flow)
- **US4 Session Persistence (Phase 6)**: Depends on US1 (requires Clerk integration)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational (Phase 2) — no dependencies on other stories
- **US2 (P1)**: Verifies work done in US1 — depends on US1 completion
- **US3 (P2)**: Requires auth gate from US1 — depends on US1 completion
- **US4 (P2)**: Requires Clerk integration from US1 — depends on US1 completion

### Within User Story 1

- DB functions (T011–T013) can run in parallel [P]
- Route handlers (T014–T016) can run in parallel [P], but depend on DB functions
- Web shell work (T018–T022) depends on API server work (T011–T017) for a working end-to-end flow
- E2E mock (T024) depends on web shell changes (T018–T022)

### Parallel Opportunities

- T001 and T002 (dependency additions) can run in parallel
- T011, T012, T013 (DB functions) can run in parallel
- T014, T015, T016 (route handlers) can run in parallel
- T038 (CI config) can run in parallel with T035–T037

---

## Parallel Example: User Story 1 API Server

```bash
# Launch all DB function updates in parallel:
Task: "Add user_id to items DB functions in crates/intrada-api/src/db/items.rs"
Task: "Add user_id to sessions DB functions in crates/intrada-api/src/db/sessions.rs"
Task: "Add user_id to routines DB functions in crates/intrada-api/src/db/routines.rs"

# Then launch all route handler updates in parallel:
Task: "Add AuthUser to items handlers in crates/intrada-api/src/routes/items.rs"
Task: "Add AuthUser to sessions handlers in crates/intrada-api/src/routes/sessions.rs"
Task: "Add AuthUser to routines handlers in crates/intrada-api/src/routes/routines.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (add dependencies)
2. Complete Phase 2: Foundational (migrations, auth module, error handling, CORS)
3. Complete Phase 3: User Story 1 (API scoping + web shell auth gate + E2E mock)
4. **STOP and VALIDATE**: Test US1 independently — sign in, see library, create item
5. Deploy/demo if ready — this is a viable MVP

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 (Sign In) → Test independently → Deploy (MVP!)
3. Add US2 (Data Isolation) → Verify query scoping → Confidence boost
4. Add US3 (Sign Out) → Test sign-out flow → Deploy
5. Add US4 (Session Persistence) → Test page reload → Deploy
6. Polish → CI config, full test suite, quickstart validation

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Auth-optional mode (no `CLERK_ISSUER_URL`) preserves all existing test behavior
- The `intrada-core` crate requires zero changes — architecture integrity maintained
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
