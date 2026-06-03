# Feature Specification: Rework Sessions (Setlist Model)

**Feature Branch**: `015-rework-sessions`
**Created**: 2026-02-15
**Status**: Draft
**Input**: User description: "Rework sessions to use a setlist model where users select multiple library items to practice in order, with a single timer that tracks time per item, skip and add-mid-session support, and end-of-session summary with per-item notes"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Build a Practice Setlist (Priority: P1)

As a musician, I want to select multiple pieces and exercises from my library and arrange them into an ordered setlist so that I can plan my practice session before I begin.

**Why this priority**: The setlist is the foundational concept of the reworked session model. Without the ability to build a setlist, no other session functionality works. This delivers the core planning value — a user can see what they intend to practice and in what order.

**Independent Test**: Can be verified by selecting items from the library, seeing them appear in an ordered setlist, reordering them, and removing items before starting.

**Acceptance Scenarios**:

1. **Given** the user has items in their library, **When** they start building a new session, **Then** they can browse their library and select items to add to the setlist.
2. **Given** the user has added items to the setlist, **When** they view the setlist, **Then** items appear in the order they were added with their title and type (piece/exercise) visible.
3. **Given** the user has added items to the setlist, **When** they reorder items, **Then** the setlist reflects the new order.
4. **Given** the user has added items to the setlist, **When** they remove an item, **Then** the item is removed from the setlist and the remaining items retain their relative order.
5. **Given** the user has an empty setlist, **When** they attempt to start the session, **Then** the system prevents starting and communicates that at least one item is required.

---

### User Story 2 - Practice Through a Setlist with Timed Tracking (Priority: P1)

As a musician, I want to work through my setlist one item at a time with a running timer that automatically tracks how long I spend on each item so that I can focus on playing rather than timekeeping.

**Why this priority**: This is the core practice experience — the session timer is the primary interaction during practice. Without it, the setlist is just a list. This is co-priority P1 with US1 because together they form the MVP.

**Independent Test**: Can be verified by starting a session with a setlist, seeing the first item displayed with a running timer, pressing "Next" to advance, and confirming time was recorded for the completed item.

**Acceptance Scenarios**:

1. **Given** the user has a setlist with at least one item, **When** they start the session, **Then** the first item in the setlist is displayed as the active item with a timer counting from zero.
2. **Given** the user is practising an item, **When** they press "Next", **Then** the time spent on the current item is recorded and the next item in the setlist becomes active with a fresh timer.
3. **Given** the user is practising the last item in the setlist, **When** they press "Finish", **Then** the time spent on the last item is recorded and the session ends, transitioning to the summary view.
4. **Given** the user is in an active session, **When** they view the session screen, **Then** they can see the current item name, elapsed time, and their progress through the setlist (e.g., "3 of 7").
5. **Given** the user has completed multiple items, **When** they advance to a new item, **Then** previously completed items are marked as done in the setlist overview.
6. **Given** the user is in an active session with some items completed, **When** they choose to end the session early, **Then** the time for the current item is recorded, remaining items are marked as not attempted, and the session transitions to the summary view with all progress captured.

---

### User Story 3 - Skip Items and Add Items Mid-Session (Priority: P2)

As a musician, I want to skip items I don't feel like practising and add items I forgot to include so that my session stays flexible and responsive to how I feel in the moment.

**Why this priority**: Flexibility during a session prevents frustration. Without skip and add, users would need to abandon and restart sessions, which is a poor experience. However, the core timer flow (US2) works without this.

**Independent Test**: Can be verified by starting a session, skipping an item, verifying it's marked as skipped, adding a new item from the library, and verifying the new item appears in the setlist.

**Acceptance Scenarios**:

1. **Given** the user is practising an item, **When** they press "Skip", **Then** the item is marked as skipped (with zero time), the timer resets, and the next item becomes active.
2. **Given** the user is in an active session, **When** they choose to add an item from their library, **Then** the selected item is added to the end of the setlist.
3. **Given** the user is in an active session, **When** they choose to add a brand new item (not in their library), **Then** the new item is created in the library and added to the end of the setlist.
4. **Given** the user has skipped an item, **When** the session summary is shown, **Then** the skipped item appears in the summary marked as skipped with no time recorded.

---

### User Story 4 - End-of-Session Summary with Notes (Priority: P2)

As a musician, I want to see a breakdown of my session when it ends and add notes about specific items or the session overall so that I can reflect on my practice and have a record for future reference.

**Why this priority**: The summary completes the practice loop — without it, session data is captured but the user has no immediate feedback. Notes are essential for the future suggestions feature the user wants to build toward.

**Independent Test**: Can be verified by completing a session and confirming the summary shows per-item time, skipped items, and allows adding per-item notes and overall session notes before saving.

**Acceptance Scenarios**:

1. **Given** the user has finished a session, **When** the summary view appears, **Then** it shows: total session duration, each item with its individual time, and which items were skipped.
2. **Given** the user is viewing the session summary, **When** they add a note to a specific item, **Then** the note is saved and associated with that item's entry in the session.
3. **Given** the user is viewing the session summary, **When** they add an overall session note, **Then** the note is saved and associated with the session as a whole.
4. **Given** the user has reviewed the summary and added any desired notes, **When** they confirm/save the session, **Then** the complete session record (items, times, notes, skips) is persisted.
5. **Given** the user does not add any notes, **When** they save the session, **Then** the session is saved successfully with no notes.

---

### User Story 5 - Replace Existing Session Data and Code (Priority: P1)

As a developer, I want the old flat session model (one session = one item) completely replaced with the new setlist-based model so that the codebase has a single, consistent session concept with no legacy code remaining.

**Why this priority**: The old session model is fundamentally incompatible with the new setlist approach. Keeping both creates confusion and technical debt. This is P1 because it's a prerequisite for all other stories — the old model must be cleared to make way for the new one.

**Independent Test**: Can be verified by confirming the old `Session` struct, `SessionEvent` enum, and related storage effects no longer exist, the old session data in localStorage is wiped on first load, and the new data structures are in place.

**Acceptance Scenarios**:

1. **Given** the app has existing session data in localStorage, **When** the app loads after the update, **Then** the old session data is discarded and the new empty session store is initialised.
2. **Given** the old session code exists, **When** the rework is complete, **Then** the old `Session` struct, `SessionEvent` enum, `LogSession`, `UpdateSession`, `SessionsData`, and related storage effects are replaced with new equivalents.
3. **Given** the old session UI exists (manual log form, practice timer component, session history component, sessions view), **When** the rework is complete, **Then** these are replaced with new session UI components supporting the setlist flow.

---

### Edge Cases

- What happens if the user closes the browser mid-session? The session-in-progress should be recoverable on next load (persisted to localStorage periodically or on each "Next" action).
- What happens if a library item referenced in a session setlist is deleted from the library? The session record should retain the item title as a snapshot so historical sessions remain readable.
- What happens if the user tries to start a session with zero items? The system should prevent starting and display a clear message.
- What happens if the user adds the same library item to the setlist multiple times? This should be allowed — a musician may want to revisit an item (e.g., warm-up scales at start and end).
- What happens if the user skips every item in the setlist? The session should still be saveable with all items marked as skipped and zero total practice time.
- What happens if the user wants to go back to a previous item? The current scope uses a forward-only "Next" model. Revisiting items is deferred to a future iteration; the user can add the item again to the end of the setlist if needed.
- What happens if the user wants to end a session early without finishing all items? The session is saved as a partial session — completed items retain their recorded times, the current item's time is recorded up to the point of ending, and remaining items are marked as not attempted. The user sees the summary view with all progress captured.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST allow users to create a new session by selecting one or more items from their library into an ordered setlist.
- **FR-002**: The system MUST allow users to reorder items in the setlist before starting the session.
- **FR-003**: The system MUST allow users to remove items from the setlist before starting the session.
- **FR-004**: The system MUST prevent starting a session with an empty setlist.
- **FR-005**: The system MUST display the current item with a running timer (counting up from zero) when a session is active.
- **FR-006**: The system MUST allow the user to advance to the next item ("Next" button), which records the elapsed time for the current item and starts a fresh timer for the next item.
- **FR-007**: The system MUST allow the user to finish the session from the last item ("Finish" button), recording the final item's time and transitioning to the summary view.
- **FR-008**: The system MUST show the user's progress through the setlist during an active session (e.g., current position out of total items).
- **FR-009**: The system MUST allow users to skip the current item during an active session, marking it as skipped with zero time.
- **FR-010**: The system MUST allow users to add items from their library to the end of the setlist during an active session.
- **FR-011**: The system MUST allow users to create a brand new library item and add it to the end of the setlist during an active session.
- **FR-012**: The session summary MUST display: total session duration, each item with its individual elapsed time, and which items were skipped.
- **FR-013**: The session summary MUST allow users to add a text note to any individual item in the session.
- **FR-014**: The session summary MUST allow users to add an overall text note for the session.
- **FR-015**: The system MUST persist the complete session record (setlist order, per-item times, skip status, per-item notes, session notes) when the user saves from the summary view.
- **FR-016**: The system MUST replace the old flat session data model (`Session`, `LogSession`, `UpdateSession`, `SessionsData`, related storage effects and events) with the new setlist-based model.
- **FR-017**: The system MUST wipe any existing old-format session data from localStorage on first load after the update.
- **FR-018**: The system MUST store a snapshot of each item's title and type in the session record so that sessions remain readable even if the original library item is later deleted.
- **FR-019**: The system MUST allow the same library item to appear multiple times in a single setlist.
- **FR-020**: The system MUST persist session-in-progress state so that it can be recovered if the user closes the browser mid-session.
- **FR-021**: Per-item notes and session notes MUST follow existing validation rules (max 5,000 characters).
- **FR-022**: The system MUST replace the existing session-related UI (manual log form, practice timer component, session history list, sessions view) with new UI supporting the setlist flow.
- **FR-023**: The system MUST allow users to end a session early at any point during practice; completed items retain their times, the current item's elapsed time is recorded, and remaining items are marked as not attempted. The session transitions to the summary view with all progress captured.
- **FR-024**: The session history list MUST display each past session with: the date, total duration, number of items practised, and whether the session was completed fully or ended early.
- **FR-025**: The per-item practice summary on library detail views (total sessions, total minutes) MUST continue to work with the new session model, aggregating time and session count from setlist entries across all completed sessions for each library item.

### Key Entities

- **PracticeSession**: A single practice event containing an ordered list of setlist entries, overall timing information, overall notes, completion status (completed fully or ended early), and metadata (ID, timestamps). This is the top-level record that gets persisted.
- **SetlistEntry**: An individual item within a session's setlist, capturing: a reference to the library item, a snapshot of the item's title and type, the position in the setlist, the time spent practising, the entry's completion status (completed, skipped, or not attempted), and optional per-item notes.
- **SessionStatus**: The lifecycle state of a session — building (setlist being assembled), active (timer running), summary (reviewing results), or completed (saved to storage).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can build a setlist of 1 or more items and begin a practice session in under 30 seconds.
- **SC-002**: Time tracking per item is accurate to within 1 second of real elapsed time.
- **SC-003**: Session data persists correctly — a completed session can be viewed after the app is closed and reopened, with all items, times, notes, and skip statuses intact.
- **SC-004**: Mid-session modifications (skip, add from library, add new item) do not interrupt or reset the timer for the current item.
- **SC-005**: Session-in-progress state survives a browser refresh without data loss.
- **SC-006**: All existing tests pass after the rework, with new tests covering the setlist session model.
- **SC-007**: Zero references to the old flat session model remain in the codebase after the rework.
- **SC-008**: The session data model captures sufficient detail (per-item times, notes, skip status, item snapshots) to enable future practice analytics and suggestions features.

## Clarifications

### Session 2026-02-15

- Q: How should abandoning/ending an active session early work? → A: Save partial session — completed items retain their times, current item's time is recorded, remaining items marked as not attempted. User sees summary with all progress captured.
- Q: What information should the session history list show for each past session? → A: Moderate detail — date, total duration, number of items practised, and whether the session was completed fully or ended early.
- Q: Should per-item practice summary on library detail views still be computed from session data? → A: Yes — aggregate time and session count from setlist entries across all completed sessions for each library item.

### Assumptions

- The forward-only "Next" model is sufficient for this iteration; revisiting earlier items in the setlist is deferred.
- The setlist is ordered but not timed in aggregate up front — there is no pre-set time limit per item or per session.
- The timer runs client-side in the shell (web) and is driven by events sent to the core; the core records timestamps, not real-time tick counts.
- Duplicate items in a setlist are treated as independent entries (separate time tracking, separate notes).
- Session history (viewing past completed sessions) will use the existing sessions view location, reworked for the new model.
- The new session data will be stored under the same localStorage key (`intrada:sessions`) but with a new schema; old data is wiped, not migrated.
- Items created mid-session via "add new item" follow the same creation rules (validation, ULID generation) as items created from the library view.
