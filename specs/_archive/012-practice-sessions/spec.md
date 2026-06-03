# Feature Specification: Practice Sessions

**Feature Branch**: `012-practice-sessions`
**Created**: 2026-02-15
**Status**: Draft
**Input**: User description: "Practice Sessions: Log practice time against library items. New domain with session tracking - start time, end time, duration, optional notes, linked to a piece or exercise by ID. New sessions.json file for CLI persistence and intrada:sessions localStorage key for web. CLI commands: start/stop session, list sessions, show session detail. Web UI: session timer on detail view, session history list. Follows the segmented JSON pattern established in 011-json-persistence."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Log a completed practice session (Priority: P1)

A musician finishes practising a piece and wants to record that they practised it. They select the library item, enter how long they practised (e.g. "30 minutes"), optionally add notes about the session ("worked on the coda, still struggling with bar 24"), and save. The session is persisted and visible in the item's practice history.

**Why this priority**: This is the most fundamental action — recording that practice happened. Without this, no other session feature has value. Manual logging is the simplest path and doesn't require timer infrastructure.

**Independent Test**: Log a session against a library item, close and reopen the app, verify the session appears in the item's history with correct duration and notes.

**Acceptance Scenarios**:

1. **Given** a library with at least one piece, **When** the user logs a 30-minute session against that piece with notes, **Then** the session is saved with the item ID, duration, notes, and current timestamp
2. **Given** a library item with no sessions, **When** the user logs a session, **Then** the item's session count changes from 0 to 1
3. **Given** a library item, **When** the user logs a session without notes, **Then** the session is saved with an empty notes field
4. **Given** multiple library items, **When** the user logs sessions against different items, **Then** each item shows only its own sessions

---

### User Story 2 - View practice history for a library item (Priority: P2)

A musician opens a piece or exercise and wants to see when and how much they've practised it. They see a chronological list of past sessions showing date, duration, and any notes. The most recent session appears first.

**Why this priority**: Viewing history is the natural companion to logging — it turns raw data into useful context. A musician can see their practice frequency and read back their own notes.

**Independent Test**: Log three sessions on different dates against one item, view the item's history, verify all three appear in reverse chronological order with correct details.

**Acceptance Scenarios**:

1. **Given** a library item with 5 logged sessions, **When** the user views the item detail, **Then** they see all 5 sessions listed newest-first
2. **Given** a library item with no sessions, **When** the user views the item detail, **Then** they see a message indicating no practice sessions have been recorded
3. **Given** a session with notes, **When** the user views the session in the history list, **Then** the notes are visible

---

### User Story 3 - Use a practice timer (Priority: P3)

A musician is about to practise and wants to time their session. They start a timer from the item detail view, practise, then stop the timer. The elapsed time is automatically recorded as a session. They can optionally add notes before saving.

**Why this priority**: A live timer is more convenient than manual logging but is an enhancement over the core logging flow. It requires more UI complexity (running timer display, active session state) but doesn't change the underlying data model.

**Independent Test**: Start a timer on an item, wait 10 seconds, stop the timer, add notes, save — verify a session is created with approximately 10 seconds duration.

**Acceptance Scenarios**:

1. **Given** a library item, **When** the user starts a timer, **Then** a running clock is displayed showing elapsed time
2. **Given** an active timer, **When** the user stops it, **Then** they are prompted to optionally add notes before the session is saved
3. **Given** an active timer on one item, **When** the user tries to start a timer on another item, **Then** they are warned that a session is already in progress
4. **Given** an active timer, **When** the user navigates away from the item, **Then** the timer continues running (it is not cancelled)

---

### User Story 4 - View all recent sessions across the library (Priority: P4)

A musician wants to see a summary of their recent practice activity across all items. They view a list of all sessions, showing which item was practised, when, and for how long.

**Why this priority**: This gives a bird's-eye view of practice activity. Useful but not essential for the core logging workflow.

**Independent Test**: Log sessions against 3 different items, view the "all sessions" list, verify all sessions appear with their associated item names.

**Acceptance Scenarios**:

1. **Given** sessions logged against multiple items, **When** the user views the sessions list, **Then** all sessions appear newest-first with their associated item title and type
2. **Given** no sessions exist, **When** the user views the sessions list, **Then** they see a message indicating no practice has been recorded yet
3. **Given** a deleted library item with past sessions, **When** the user views the sessions list, **Then** those sessions still appear but indicate the item no longer exists

---

### User Story 5 - Edit a practice session (Priority: P5)

A musician logged a session with the wrong duration or wants to add notes after the fact. They can edit an existing session to correct the duration or update the notes.

**Why this priority**: Error correction is important but secondary to the core record/view flow. Editing is more user-friendly than deleting and re-creating.

**Independent Test**: Log a session with 30 minutes and no notes, edit it to 45 minutes with notes, verify the updated values appear in the session history.

**Acceptance Scenarios**:

1. **Given** an existing session, **When** the user edits the duration and notes, **Then** the session is updated with the new values
2. **Given** an existing session, **When** the user edits only the notes, **Then** the duration remains unchanged
3. **Given** an existing session, **When** the user submits an invalid duration (0 or > 1440), **Then** the edit is rejected with a validation error

---

### User Story 6 - Delete a practice session (Priority: P6)

A musician logged a session by mistake (wrong item entirely) and wants to remove it. They can delete individual sessions from the practice history.

**Why this priority**: Deletion is the last resort for error correction, after editing. Secondary to edit functionality.

**Independent Test**: Log a session, delete it, verify it no longer appears in the item's history or the all-sessions list.

**Acceptance Scenarios**:

1. **Given** a session in the practice history, **When** the user deletes it, **Then** it is removed from all views
2. **Given** a session to delete, **When** the user confirms deletion, **Then** the session is permanently removed from storage

---

### Edge Cases

- What happens when a user logs a session against a library item that is later deleted? The session is retained with its item ID. When displayed, the item title shows as "Deleted item" or similar.
- What happens when a user enters a duration of 0 minutes? Rejected — minimum session duration is 1 minute.
- What happens when a user enters an extremely long duration (e.g. 24 hours)? Accepted — maximum duration is 1440 minutes (24 hours). Durations beyond this are rejected.
- What happens when the browser tab is closed during an active timer? The timer state is lost. The session is not saved. This is acceptable for P3 — persistence of active timers is a future enhancement.
- What happens when two sessions overlap in time for the same item? Allowed — the system does not enforce non-overlapping sessions. Musicians may log retroactively or adjust times.
- What happens when the sessions file/key is missing? An empty sessions list is returned, matching the library data pattern.
- What happens when the timer is stopped before 30 seconds? The duration rounds to 0 minutes, which is below the minimum — the session is rejected with a validation error explaining the minimum is 1 minute.

## Clarifications

### Session 2026-02-15

- Q: Should the CLI support a live timer (blocking foreground process) or only manual session logging? A: Manual logging only. The CLI does not need a live timer — users enter duration after the fact. No background process or state file needed.
- Q: Should sessions be editable after creation (change duration, notes) or only deletable? A: Sessions are fully editable — users can update both duration and notes after creation.
- Q: How should the web timer's second-level precision be stored? A: Round to nearest minute (< 30s rounds down, >= 30s rounds up). Duration is always stored in whole minutes, keeping the data model consistent across CLI and web.
- Q: What timestamp semantics should sessions use? A: Store both "started-at" (when practice began) and "logged-at" (when the session was saved). For timer sessions, started-at is the actual start. For manual logs, started-at defaults to logged-at minus duration.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Users MUST be able to log a practice session by specifying a library item and duration
- **FR-002**: Each session MUST be linked to exactly one library item (piece or exercise) by its ID
- **FR-003**: Each session MUST record: a unique session ID, the linked item ID, duration in whole minutes, a "started-at" timestamp (when practice began), a "logged-at" timestamp (when the session was saved), and optional free-text notes. For manual logs, "started-at" defaults to "logged-at" minus duration
- **FR-004**: Users MUST be able to view all sessions for a specific library item, ordered newest-first
- **FR-005**: Users MUST be able to view all sessions across the entire library, ordered newest-first
- **FR-006**: Users MUST be able to delete individual sessions
- **FR-007**: The web shell MUST provide a start/stop practice timer that automatically creates a session with the elapsed duration
- **FR-008**: Only one timer session MAY be active at a time across the entire application
- **FR-009**: Session duration MUST be between 1 and 1440 whole minutes (inclusive). The web timer MUST round to the nearest minute (< 30 seconds rounds down, >= 30 seconds rounds up) before saving
- **FR-010**: Sessions MUST persist in `sessions.json` (CLI) and `intrada:sessions` localStorage key (web), following the segmented JSON pattern from 011-json-persistence
- **FR-011**: Deleting a library item MUST NOT delete its associated sessions — orphaned sessions are displayed with a placeholder title
- **FR-012**: The CLI MUST support logging sessions manually via a command specifying item ID and duration
- **FR-013**: The CLI MUST support listing sessions with optional filtering by item ID
- **FR-014**: The item detail view (both CLI and web) MUST display the total number of sessions and total practice time for that item
- **FR-015**: Users MUST be able to edit an existing session's duration and notes
- **FR-016**: The CLI MUST NOT include a live timer — session logging is manual only (user provides duration after the fact)
- **FR-017**: The CLI MUST support editing a session via a command specifying session ID and updated fields
- **FR-018**: The CLI MUST support showing detail for a single session by its ID

### Key Entities

- **Session**: A single record of practice activity. Attributes: unique ID (ULID), linked item ID, duration (whole minutes), started-at timestamp (when practice began), logged-at timestamp (when the session was saved), optional notes. For manual logs, started-at is computed as logged-at minus duration. A session belongs to one library item but exists independently (survives item deletion).
- **SessionsData**: Top-level serialisation unit for `sessions.json` / `intrada:sessions`. Contains `sessions: Vec<Session>`. Follows the same `#[serde(default)]` pattern as `LibraryData`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can log a practice session and see it in the item's history within the same interaction — no page refresh or app restart required
- **SC-002**: Session data persists across app restarts in both CLI and web shells
- **SC-003**: All existing library functionality (add, edit, delete, search pieces and exercises) continues to work without regression
- **SC-004**: The practice timer in the web shell accurately tracks elapsed time with no more than 1 second of drift per hour
- **SC-005**: Session history for an item with 100 sessions loads and displays without perceptible delay
- **SC-006**: All session validation (duration bounds, required fields) provides clear user-facing error messages
