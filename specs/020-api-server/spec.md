# Feature Specification: API Server

**Feature Branch**: `020-api-server`
**Created**: 2026-02-15
**Status**: Draft
**Input**: User description: "API server on Fly.io with Turso/libsql database backend. REST CRUD endpoints for pieces, exercises, and practice sessions. Reuse existing intrada-api Axum routes and validation from intrada-core. Replace sqlx/Postgres with libsql for Turso compatibility. SQLite-compatible schema (JSON columns for tags instead of TEXT arrays). CORS support for cross-origin requests from Cloudflare Workers frontend. Health check endpoint. Database migrations via libsql_migration crate."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Library CRUD (Priority: P1)

As a user, I want to create, read, update, and delete pieces and exercises through a server-side API so that my music library is stored durably in a database rather than only in browser localStorage.

**Why this priority**: The library is the core data of the application. Without CRUD operations on pieces and exercises, no other API functionality (sessions, sync) has anything to operate on.

**Independent Test**: Can be fully tested with HTTP requests (e.g., curl) against the running server. Create a piece, list all pieces, get it by ID, update it, delete it — each operation verifiable independently.

**Acceptance Scenarios**:

1. **Given** an empty database, **When** a user creates a piece with a title and composer, **Then** the system returns the created piece with a generated ID and timestamps, and a subsequent list request includes that piece.
2. **Given** an existing piece, **When** a user updates the piece's title, **Then** the system returns the updated piece with a new updated-at timestamp and the original created-at timestamp preserved.
3. **Given** an existing piece, **When** a user deletes it, **Then** the system confirms deletion and a subsequent get request for that ID returns not found.
4. **Given** a create request with an empty title, **When** the system validates the input, **Then** the system returns a validation error indicating the title is required.
5. **Given** an existing exercise with tags, **When** a user updates only the notes field, **Then** all other fields (title, composer, category, tags, etc.) remain unchanged.

---

### User Story 2 - Practice Sessions (Priority: P2)

As a user, I want to store completed practice sessions on the server so that my practice history is preserved and accessible from any device.

**Why this priority**: Practice sessions are the second major data type. They are read-heavy (view history) with occasional writes (save completed session). They depend on pieces/exercises existing in the library.

**Independent Test**: Can be tested by saving a completed session via the API and then listing/retrieving it. Requires at least one piece or exercise to exist (from US1).

**Acceptance Scenarios**:

1. **Given** a completed practice session with setlist entries, **When** a user saves the session, **Then** the system stores it with all entries, notes, timestamps, durations, and completion status.
2. **Given** multiple saved sessions, **When** a user lists sessions, **Then** the system returns them in reverse chronological order (most recent first).
3. **Given** a saved session, **When** a user retrieves it by ID, **Then** the system returns the full session including all setlist entries.
4. **Given** a saved session, **When** a user deletes it, **Then** the system confirms deletion and it no longer appears in the list.

---

### User Story 3 - Cross-Origin Access (Priority: P3)

As a frontend application hosted on a different domain, I need to make API requests to the server without being blocked by browser security policies, so that the web app can communicate with the API.

**Why this priority**: Without cross-origin support, the frontend (hosted on Cloudflare Workers) cannot communicate with the API (hosted on a separate server). This is a prerequisite for the future Cloud Sync feature but must be built into the API from the start.

**Independent Test**: Can be tested by making a preflight OPTIONS request from a different origin and verifying the response includes the correct headers allowing the request.

**Acceptance Scenarios**:

1. **Given** a request from the frontend origin, **When** the browser sends a preflight OPTIONS request, **Then** the server responds with headers allowing the request to proceed.
2. **Given** a request from an unknown origin, **When** the browser sends a preflight request, **Then** the server rejects it with appropriate headers.

---

### User Story 4 - Health & Readiness (Priority: P4)

As a deployment platform, I need a health check endpoint to monitor whether the API server is running and can reach its database, so that unhealthy instances can be detected and replaced.

**Why this priority**: Essential for production operations but not directly user-facing. A simple endpoint that verifies connectivity.

**Independent Test**: Can be tested by hitting the health endpoint and verifying a success response.

**Acceptance Scenarios**:

1. **Given** a running server with database connectivity, **When** the health endpoint is requested, **Then** the system returns a success status.
2. **Given** a running server where the database is unreachable, **When** the health endpoint is requested, **Then** the system returns an error status indicating the database is unavailable.

---

### Edge Cases

- What happens when creating an item with tags containing special characters or Unicode? The system should accept and store them correctly (within length limits).
- What happens when updating with a field set to null vs omitting the field? Null clears the field; omitting it leaves it unchanged (three-state semantics).
- What happens when requesting an item that doesn't exist? The system should return a 404 with a clear error message.
- What happens when the request body is malformed or missing required fields? The system should return a 400 with a descriptive validation error.
- What happens when the database is unavailable during a write operation? The system should return a 500 with a generic error message (no internal details leaked).
- What happens when a session references a piece/exercise ID that no longer exists? The session should still be stored — the item title is denormalised in the setlist entry so the session remains readable.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide CRUD operations (create, list, get by ID, update, delete) for pieces.
- **FR-002**: The system MUST provide CRUD operations (create, list, get by ID, update, delete) for exercises.
- **FR-003**: The system MUST provide operations to save, list, get by ID, and delete practice sessions.
- **FR-004**: The system MUST validate all input using the same validation rules as the existing client application (title length, composer length, BPM range, tag length, notes length, etc.).
- **FR-005**: The system MUST return structured error responses for validation failures, including which field failed and why.
- **FR-006**: The system MUST generate unique IDs and timestamps server-side when creating items (clients do not provide IDs).
- **FR-007**: Update operations MUST support three-state semantics: omit a field to leave it unchanged, set it to null to clear it, or provide a value to update it.
- **FR-008**: The system MUST support cross-origin requests from the frontend application's domain.
- **FR-009**: The system MUST provide a health check endpoint that verifies both server and database availability.
- **FR-010**: List operations MUST return items in reverse chronological order (newest first).
- **FR-011**: The system MUST persist all data to a durable database that survives server restarts and redeployments.
- **FR-012**: The system MUST run database schema migrations automatically on startup.
- **FR-013**: Tags MUST be stored as a structured list (not a single string) so individual tags can be queried and displayed correctly.
- **FR-014**: Practice sessions MUST store denormalised item titles in setlist entries so sessions remain readable even if the referenced piece/exercise is later deleted.

### Key Entities

- **Piece**: A musical piece in the library. Has a title (required), composer (required), and optional key, tempo (marking + BPM), notes, and tags. Identified by a unique ID with created and updated timestamps.
- **Exercise**: A practice exercise in the library. Has a title (required), and optional composer, category, key, tempo, notes, and tags. Identified by a unique ID with created and updated timestamps.
- **Practice Session**: A completed practice session with a setlist of entries, session-level notes, start/completion timestamps, total duration, and completion status (completed or ended early).
- **Setlist Entry**: An individual item practiced within a session. References a piece or exercise by ID and title, with a position, duration, status (completed/skipped/not attempted), and optional notes.
- **Tempo**: A tempo descriptor with optional marking (e.g., "Allegro") and optional BPM (1-400). At least one must be present.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All CRUD operations for pieces and exercises complete successfully and return correct data when tested with HTTP requests.
- **SC-002**: Validation errors return descriptive messages identifying the failing field, matching the same rules enforced by the client application.
- **SC-003**: Data persists across server restarts — items created before a restart are retrievable after the restart.
- **SC-004**: The frontend application (on a different domain) can successfully make API requests without cross-origin errors.
- **SC-005**: The health check endpoint accurately reports server and database status.
- **SC-006**: Practice sessions store and return all setlist entries with correct positions, durations, statuses, and notes.
- **SC-007**: All existing client-side tests continue to pass with no changes (the core library is not modified).

### Assumptions

- No authentication is required for this feature — the API is single-user and unauthenticated. Auth will be added in a future feature.
- The frontend application's domain is known and can be configured as an allowed origin for cross-origin requests.
- Practice sessions are write-once from the API perspective — they are saved as completed sessions and cannot be edited after creation (only deleted).
- The API does not need to support pagination for list operations in this initial version. The expected dataset size (hundreds, not millions) does not require it.
- The API does not need to support search or filtering for list operations in this initial version. This can be added later.
