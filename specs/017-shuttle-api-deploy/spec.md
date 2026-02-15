# Feature Specification: Shuttle API Server & Database

**Feature Branch**: `017-shuttle-api-deploy`
**Created**: 2026-02-15
**Status**: Draft
**Input**: User description: "Deploy the Intrada web app to Shuttle.rs. Create an Axum API server that serves the static WASM build and provides REST CRUD endpoints for library items (pieces, exercises) and practice sessions. Choose between Turso (LibSQL) or Shuttle Postgres for the database during planning. Single-user, no auth for now (auth will be a separate follow-up feature). The web app should switch from localStorage to API calls as the primary data source, keeping localStorage as a read cache for fast initial loads. The Crux core remains untouched — only the web shell's storage layer changes. This is feature 017 in the roadmap, followed by 018 (offline-first sync) and 019 (auth)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Server-Persisted Library (Priority: P1)

The user manages their music library (pieces and exercises) through the web app and all data is stored on the server rather than only in the browser. When they add a new piece, edit an exercise, or delete an item, the change is saved to the server's database and survives browser clears, device switches, or incognito sessions.

**Why this priority**: Without server-side persistence the app remains browser-only, which is the core problem this feature solves. Every other story depends on the API and database being available.

**Independent Test**: Can be fully tested by launching the deployed server, opening the web app, creating a piece, closing the browser, reopening it, and confirming the piece is still present.

**Acceptance Scenarios**:

1. **Given** the user has the web app open, **When** they add a new piece (title, composer, optional fields), **Then** the piece is saved to the server database and appears in the library list.
2. **Given** a piece exists in the library, **When** the user edits its title, **Then** the updated title is persisted on the server and reflected in the UI.
3. **Given** an exercise exists in the library, **When** the user deletes it, **Then** it is removed from the server database and no longer appears in the library list.
4. **Given** the user has items in their library, **When** they clear browser storage and reload, **Then** all items are fetched from the server and displayed.

---

### User Story 2 — Server-Persisted Practice Sessions (Priority: P2)

Completed practice sessions are saved to the server. The user can view their session history and it persists across browsers and devices.

**Why this priority**: Practice sessions are the second major data type. Storing them server-side gives the user a durable practice log, but the app provides core value (library management) even without this.

**Independent Test**: Can be tested by completing a practice session, verifying it appears in session history, clearing browser storage, and confirming it reloads from the server.

**Acceptance Scenarios**:

1. **Given** the user completes a practice session, **When** the session summary is saved, **Then** the session is persisted to the server database.
2. **Given** the user has past sessions stored on the server, **When** they view session history, **Then** all sessions are listed with correct dates, durations, and items practised.
3. **Given** sessions exist on the server, **When** the user opens the app on a different browser, **Then** the full session history is available.

---

### User Story 3 — Static WASM Hosting (Priority: P3)

The server hosts the compiled WASM web app as static files, so users access the app at a single URL without needing a separate CDN or static host.

**Why this priority**: Simplifies deployment to a single service. The app is already functional locally; this story is about making it accessible over the internet.

**Independent Test**: Can be tested by navigating to the deployed URL in a browser and confirming the app loads and renders correctly.

**Acceptance Scenarios**:

1. **Given** the server is running, **When** a user navigates to the root URL, **Then** the WASM app loads and displays the home screen.
2. **Given** the app uses client-side routing, **When** the user refreshes on a deep link (e.g. `/library/abc123`), **Then** the server returns the app shell and client-side routing resolves the page.

---

### User Story 4 — Local Cache for Fast Loads (Priority: P4)

The web app loads data from localStorage first for instant rendering, then refreshes from the server in the background. This gives the user a fast initial experience without a loading spinner on every page.

**Why this priority**: This is a UX polish layer. The app works correctly without it (by fetching from the server on every load), but the cache makes the experience feel snappy.

**Independent Test**: Can be tested by loading the app with a populated localStorage cache while on a slow connection, confirming data appears immediately, and then verifying the server data replaces it shortly after.

**Acceptance Scenarios**:

1. **Given** the user has previously loaded library data, **When** they reopen the app, **Then** cached data appears immediately while the server fetch happens in the background.
2. **Given** the server has newer data than the local cache, **When** the background fetch completes, **Then** the UI updates to reflect the server's current state.
3. **Given** the server is unreachable, **When** the app loads with cached data, **Then** the cached data is displayed and the user is informed that the data may be stale.

---

### Edge Cases

- What happens when the server returns an error (500, network timeout) during a save operation? The user should see an error message and the data should not be lost from the local state.
- What happens when the user submits invalid data (e.g. empty title)? Validation should occur client-side (existing Crux validation) before the API call is made, and the server should also reject invalid data.
- What happens when the server database is empty on first deployment? The app should show an empty library with no errors.
- What happens when the user creates items faster than the server can respond? Operations should be sequential per resource — the UI should prevent double submissions.
- What happens if a delete request fails? The item should remain visible in the UI and an error should be shown.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a hosted server that serves the compiled WASM web application as static files at the root URL.
- **FR-002**: The server MUST handle client-side routing by returning the app shell (index.html) for all non-API, non-static-asset URL paths.
- **FR-003**: The system MUST provide REST API endpoints for creating, reading, updating, and deleting pieces.
- **FR-004**: The system MUST provide REST API endpoints for creating, reading, updating, and deleting exercises.
- **FR-005**: The system MUST provide REST API endpoints for creating and reading practice sessions (sessions are immutable once completed — no update or delete).
- **FR-006**: The server MUST persist all data in a relational database.
- **FR-007**: The server MUST validate all incoming data using the same rules as the client (title 1–500 chars, composer 1–200 chars, notes max 5000 chars, tags max 100 chars each, BPM 1–400, etc.).
- **FR-008**: The server MUST return appropriate HTTP status codes: 200/201 for success, 400 for validation errors, 404 for not found, 500 for server errors.
- **FR-009**: The server MUST return JSON responses for all API endpoints.
- **FR-010**: The web app MUST use API calls as the primary data source for all CRUD operations instead of localStorage alone.
- **FR-011**: The web app MUST write API responses to localStorage as a read cache so that subsequent page loads can display cached data before the server responds.
- **FR-012**: The web app MUST fetch fresh data from the server after displaying cached data, and update the UI if the server data differs.
- **FR-013**: The web app MUST show an error indicator when a server write operation fails, without losing the user's in-progress data.
- **FR-014**: The system MUST operate as single-user with no authentication or access protection. All API endpoints are publicly accessible. All data belongs to one implicit user. (Auth will be added in feature 019.)
- **FR-015**: The Crux core logic (intrada-core) MUST remain unchanged. Only the web shell's data access layer changes.
- **FR-016**: The server MUST be deployable to the Shuttle.rs platform.
- **FR-017**: The CI pipeline MUST build and test the server crate alongside existing crates.
- **FR-018**: The CI pipeline MUST automatically deploy the application (WASM build + server) to Shuttle.rs on merge to main, after all tests pass.

### Key Entities

- **Piece**: A musical composition with title (required), composer (required), key, tempo (marking and/or BPM), notes, and tags. Identified by a ULID. Has created/updated timestamps.
- **Exercise**: A practice exercise with title (required), optional composer, optional category, key, tempo, notes, and tags. Identified by a ULID. Has created/updated timestamps.
- **PracticeSession**: A completed practice session containing a list of setlist entries, optional session notes, start/end times, total duration, and a completion status (completed or ended early). Identified by a ULID. Immutable once saved.
- **SetlistEntry**: An individual item within a practice session, referencing a library item by ID and title, with position, duration, status (completed/skipped/not attempted), and optional notes.

## Clarifications

### Session 2026-02-15

- Q: Should the publicly hosted API require any access protection (e.g. shared secret) before auth is added in feature 019? → A: No — fully open. The URL obscurity is sufficient for this phase; auth comes in 019.
- Q: How should the application be deployed to Shuttle.rs — manually, via CI auto-deploy, or CI-built artifact with manual trigger? → A: CI auto-deploy. The CI pipeline builds the WASM and deploys to Shuttle automatically on merge to main.

## Assumptions

- The database engine (Turso/LibSQL or Shuttle Postgres) will be selected during the planning phase based on Shuttle.rs platform support, cost, and simplicity. This spec is intentionally database-agnostic.
- No authentication or multi-user support is needed for this feature. Auth will be added in feature 019.
- Offline write queuing is out of scope — if the server is unreachable during a write, the operation fails with an error. Offline-first sync will be added in feature 018.
- The in-progress session state (crash recovery data in `intrada:session-in-progress`) remains localStorage-only. Only completed sessions are sent to the server.
- Existing E2E tests may need minor updates to work against the API-backed app, but the core user flows should remain the same.
- The WASM build output is embedded into or served alongside the API server as static files — there is no separate CDN or static hosting service.
- The server does not need to handle high concurrency. Single-user, single-device access is the expected pattern for this phase.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The web app is accessible at a public URL and loads the full WASM application within 5 seconds on a standard broadband connection.
- **SC-002**: All library CRUD operations (create, read, update, delete for pieces and exercises) round-trip through the server and persist across browser clears.
- **SC-003**: Completed practice sessions are retrievable from the server after browser storage is cleared.
- **SC-004**: The app displays cached data within 500 milliseconds of page load, before the server response arrives.
- **SC-005**: All existing unit tests (142+) continue to pass unchanged (core logic is not modified).
- **SC-006**: The server validates input and returns 400 errors for invalid data (e.g. empty title, BPM out of range) with a descriptive error message.
- **SC-007**: All existing E2E user flows (14 tests) pass against the API-backed application (with test infrastructure updates as needed).
- **SC-008**: Merging to main triggers an automated deployment to Shuttle.rs, and the updated application is accessible at the public URL within minutes.
