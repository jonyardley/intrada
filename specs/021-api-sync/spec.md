# Feature Specification: API Sync

**Feature Branch**: `021-api-sync`
**Created**: 2026-02-15
**Status**: Draft
**Input**: User description: "Connect the web frontend to the REST API — replace localStorage persistence with HTTP calls to the Fly.io API server so that library data (pieces, exercises) and practice sessions sync to Turso via the intrada-api."

## Clarifications

### Session 2026-02-15

- Q: Should existing localStorage data be migrated to the server on first API-backed load? → A: No migration — existing localStorage library/sessions data is ignored; users start fresh from the API.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Library Persists to Server (Priority: P1)

As a musician, when I add, edit, or delete pieces and exercises in the web app, I want my changes saved to the server so that my library persists across devices and browsers rather than being trapped in a single browser's local storage.

**Why this priority**: This is the core value of the feature — without server persistence for the library, no other sync functionality matters. Pieces and exercises are the foundation of the app.

**Independent Test**: Open the app in one browser, add a piece. Open the app in a different browser (or clear local storage and reload). The piece should appear in both. Delete a piece in one browser, refresh the other — it should be gone.

**Acceptance Scenarios**:

1. **Given** the app is loaded, **When** the library view appears, **Then** pieces and exercises are fetched from the API server (not from localStorage).
2. **Given** a user adds a new piece via the form, **When** they submit, **Then** the piece is created on the server via the API and appears in the library list.
3. **Given** a user edits a piece, **When** they save changes, **Then** the piece is updated on the server and the updated version is shown.
4. **Given** a user deletes a piece, **When** they confirm deletion, **Then** the piece is removed from the server and disappears from the library list.
5. **Given** a user adds, edits, or deletes an exercise, **When** the operation completes, **Then** the same server-backed behaviour applies as for pieces.

---

### User Story 2 - Practice Sessions Persist to Server (Priority: P2)

As a musician, when I complete a practice session, I want it saved to the server so that my practice history is available across all my devices and won't be lost if I clear my browser data.

**Why this priority**: Sessions are write-once records of practice activity. They depend on the library (pieces/exercises) existing on the server, making this a natural second step after US1.

**Independent Test**: Complete a practice session in one browser. Open the app in another browser. The completed session should appear in the practice history.

**Acceptance Scenarios**:

1. **Given** a user completes a practice session, **When** they finish and see the summary, **Then** the session is saved to the server via the API.
2. **Given** the app loads, **When** the sessions history view is shown, **Then** completed sessions are fetched from the server.
3. **Given** a user deletes a completed session, **When** they confirm, **Then** it is removed from the server.

---

### User Story 3 - Graceful Error Handling (Priority: P3)

As a musician, when the server is temporarily unavailable (network error, cold start delay, server down), I want clear feedback about what went wrong so that I understand why my action didn't complete, rather than seeing a silent failure or a broken UI.

**Why this priority**: Network failures are inevitable. Without clear error handling, users will lose trust in the app when operations fail silently. However, this is lower priority than getting the core sync working.

**Independent Test**: Disconnect from the network (or point the app at a non-existent API URL), then try to add a piece. The app should show an error message rather than failing silently or crashing.

**Acceptance Scenarios**:

1. **Given** the API server is unreachable, **When** the user tries to add a piece, **Then** the app displays a user-friendly error message indicating the operation failed.
2. **Given** the API returns a validation error (e.g., missing required field), **When** the user submits a form, **Then** the specific validation error is displayed.
3. **Given** the API server is unreachable, **When** the app loads, **Then** the app shows a clear message that data could not be loaded, rather than displaying an empty library with no explanation.

---

### Edge Cases

- What happens when the API is slow to respond (e.g., Fly.io machine resuming from suspend)? The app should show a loading state rather than appearing frozen.
- What happens if the user submits a form and the API request is in-flight? Buttons should be disabled or show a loading indicator to prevent duplicate submissions.
- What happens to the session-in-progress crash recovery? In-progress sessions should continue to use local storage for crash recovery (they are not yet completed and should not be sent to the server until finished).
- What happens to the stub/seed data for first-time users? On first load, if the API returns an empty library, the app should display an empty state rather than seeding fake data.
- What happens if the API returns data that fails to deserialise? The app should show an error rather than crashing.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The web app MUST fetch all pieces and exercises from the API server on initial load, replacing the current localStorage-based loading.
- **FR-002**: The web app MUST send create, update, and delete operations for pieces and exercises to the API server instead of writing to localStorage.
- **FR-003**: The web app MUST fetch completed practice sessions from the API server on initial load.
- **FR-004**: The web app MUST save completed practice sessions to the API server instead of localStorage.
- **FR-005**: The web app MUST delete practice sessions via the API server instead of localStorage.
- **FR-006**: The web app MUST display user-friendly error messages when API requests fail (network errors, server errors, validation errors).
- **FR-007**: The web app MUST show loading indicators while waiting for API responses on initial data load and on form submissions.
- **FR-008**: The web app MUST continue to use localStorage for session-in-progress crash recovery (in-progress sessions are not sent to the server).
- **FR-009**: The web app MUST NOT seed stub/fake data on first load — if the API returns an empty library, the app shows an empty state. Existing localStorage data (from before this feature) is not migrated to the server; it is simply ignored.
- **FR-010**: The web app MUST prevent duplicate form submissions while an API request is in-flight.
- **FR-011**: The API base URL MUST be configurable (not hard-coded) so that the app can point to different environments (local development vs production).

### Key Entities

- **Piece**: A musical piece in the user's library. Created, updated, and deleted via the API. Has title, composer, key, tempo, notes, tags.
- **Exercise**: A practice exercise in the user's library. Same lifecycle as pieces. Has title, optional composer, optional category, key, tempo, notes, tags.
- **Practice Session**: A completed practice session. Write-once — created via the API after the user finishes practising. Has setlist entries, timing, notes. Can be deleted but not edited.
- **Active Session (in-progress)**: A session currently being practised. Stored in localStorage only for crash recovery. Not synced to the server until completed.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All library operations (add, edit, delete pieces and exercises) persist to the server and are visible from any browser accessing the app.
- **SC-002**: Completed practice sessions persist to the server and appear in the history from any browser.
- **SC-003**: When the server is unreachable, every failed operation shows an error message within 10 seconds (no silent failures, no infinite spinners).
- **SC-004**: The app shows a loading indicator during initial data load and on form submissions so the user always knows when a request is in progress.
- **SC-005**: Existing tests continue to pass — no regressions in core logic or web shell behaviour.

### Assumptions

- The API server (intrada-api on Fly.io) is already deployed and operational with the endpoints defined in feature 020-api-server.
- CORS is already configured on the API server to accept requests from the Cloudflare Workers frontend origin.
- There is no authentication — the API is open. Authentication may be added in a future feature.
- There is no offline/local-first fallback — if the server is unreachable, operations fail with an error message. Offline support may be added in a future feature.
- The API base URL for production will be the Fly.io app URL (e.g., `https://intrada-api.fly.dev`).
