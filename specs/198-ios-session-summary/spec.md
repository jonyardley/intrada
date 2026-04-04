# Feature Specification: iOS Session Summary & History

**Feature Branch**: `198-ios-session-summary`
**Created**: 2026-04-03
**Status**: Draft
**Input**: iOS session summary — post-session review screen showing per-item scores, duration, tempo achieved, notes, session stats (total time, items completed). Save or discard session. View past session history. iPhone and iPad layouts. Replaces SummaryPlaceholderView in PracticeTabRouter.

## User Scenarios & Testing

### User Story 1 — Review & Save a Completed Session (Priority: P1)

After finishing (or ending early) a practice session, the musician sees a summary of what they just practised. They can review per-item results, optionally edit scores/tempo/notes, add overall session notes, and save the session to their history. This is the critical post-practice reflection moment.

**Why this priority**: Without this, completed sessions are lost — the user can only discard. Saving is the minimum viable action that makes the active session feature useful.

**Independent Test**: Finish a 3-item session → see summary with item list, scores, duration → add session notes → tap Save → session appears in history.

**Acceptance Scenarios**:

1. **Given** a session has just been completed, **When** the summary view loads, **Then** it displays: "Session Complete!" header, total duration, completion status (completed or ended early), and a list of all items with their status, duration, and any scores/tempo/notes recorded during the session.
2. **Given** the summary is showing, **When** the user taps a confidence score (1–5) on a completed item, **Then** the score is recorded (or updated if already set).
3. **Given** the summary is showing, **When** the user enters or updates a tempo (BPM) for a completed item, **Then** the tempo is recorded.
4. **Given** the summary is showing, **When** the user types in the session notes field, **Then** the overall session notes are saved.
5. **Given** the summary is showing with edits made, **When** the user taps "Save Session", **Then** the session is persisted to history and the Practice tab returns to the idle state.
6. **Given** the summary is showing, **When** the user taps "Discard", **Then** a confirmation appears. On confirm, the session is discarded and the Practice tab returns to idle.

---

### User Story 2 — View Past Session History (Priority: P2)

The musician can browse their practice history — a chronological list of completed sessions with key stats. They can tap into a session to see the full detail. This provides the "Track" pillar's foundational data view.

**Why this priority**: History is what makes saving sessions valuable. Without it, saved data has no visibility. This also feeds into the analytics dashboard (#201).

**Independent Test**: Save two sessions on different days → go to Practice tab (idle) → see session list with dates, durations, item counts → tap a session → see full detail.

**Acceptance Scenarios**:

1. **Given** the user is on the Practice tab in idle state, **When** sessions exist in history, **Then** a list of past sessions is displayed with: date, total duration, item count, completion status, and session intention (if set).
2. **Given** the session list is showing, **When** the user taps a session, **Then** the full session detail expands or navigates to show per-item results (scores, tempo, notes, status).
3. **Given** a session in the list was ended early, **When** looking at that session, **Then** an "Ended Early" indicator is visible.
4. **Given** the session list, **When** the user wants to delete a session, **Then** a swipe-to-delete or delete button with confirmation removes it from history.
5. **Given** no sessions exist in history, **When** the Practice tab shows idle state, **Then** an empty state encourages the user to start their first session.

---

### User Story 3 — Edit Scores & Notes After Saving (Priority: P3)

Sometimes a musician wants to revisit a past session and update scores or add notes they forgot. Tapping into a saved session's detail allows editing scores, tempo, and notes for individual items.

**Why this priority**: Nice-to-have refinement — most scoring happens in the summary immediately after practice, but late edits are occasionally needed.

**Independent Test**: View a past session → tap an item's score → change from 3 to 4 → score persists on return.

**Acceptance Scenarios**:

1. **Given** a past session's detail view, **When** the user taps a confidence score for an item, **Then** the score updates and is persisted.
2. **Given** a past session's detail view, **When** the user edits the tempo or notes for an item, **Then** the changes are persisted.
3. **Given** a past session's detail view, **When** the user edits overall session notes, **Then** the changes are persisted.

---

### Edge Cases

- What happens when a session has only skipped/not-attempted items? The summary shows all items with their skip/not-attempted status. Save is still available.
- What happens when the user force-quits during the summary? The session is in summary state and crash recovery restores it. The user can still save or discard.
- What happens when the user has many sessions (50+)? The history list should scroll smoothly without loading delay.
- What happens when a session has no scored items? The summary displays fine — scoring fields are optional and shown empty.

## Requirements

### Functional Requirements

- **FR-001**: The session summary view MUST display the total session duration, completion status (completed or ended early), and session intention (if set).
- **FR-002**: The summary MUST list all session entries showing: item title, item type, status (completed/skipped/not attempted), duration, confidence score (if set), achieved tempo (if set), rep count vs target (if applicable), and notes (if set).
- **FR-003**: The user MUST be able to edit confidence scores (1–5) for completed items directly in the summary.
- **FR-004**: The user MUST be able to edit achieved tempo (BPM) for completed items directly in the summary.
- **FR-005**: The user MUST be able to edit per-item notes directly in the summary.
- **FR-006**: The user MUST be able to add or edit overall session notes.
- **FR-007**: The user MUST be able to save the session, persisting all data to history.
- **FR-008**: The user MUST be able to discard the session with a confirmation dialog.
- **FR-009**: The Practice tab idle state MUST show a list of past sessions when history exists, or an empty state when no sessions have been saved.
- **FR-010**: Each session in the history list MUST display: date/time, total duration, item count, completion status, and session intention (if set).
- **FR-011**: The user MUST be able to tap a past session to view its full detail (same layout as summary but read-only or editable per US3).
- **FR-012**: The user MUST be able to delete a past session with confirmation.
- **FR-013**: The summary and history views MUST support both iPhone and iPad layouts.

### Key Entities

- **SummaryView**: Post-session review state — total duration, completion status, session notes, entries, session intention
- **PracticeSessionView**: Saved session in history — same fields as summary plus id, started/finished timestamps
- **SetlistEntryView**: Individual item result — title, type, status, duration, score, tempo, notes, rep data

## Design

### Existing Components Used

- `ButtonView` — Save/Discard actions
- `CardView` — Section containers
- `TypeBadge` — Item type indicators
- `ScoreSelectorView` — Confidence score dots (from #197)
- `EmptyStateView` — No sessions state
- `Toast` — Save confirmation feedback
- `PageHeading` — Screen titles

### New Components Needed

- **SessionSummaryView**: Post-session review screen — header stats, scrollable entry list with inline editing, session notes, Save/Discard actions.
- **SessionEntryResultRow**: Single entry result display — status icon, title, duration, score dots, tempo, notes field. Supports both read-only (history) and editable (summary) modes.
- **SessionHistoryList**: Chronological list of saved sessions — date grouping, session cards with stats, tap to expand/navigate.
- **SessionDetailView**: Full detail view for a past session — same layout as summary, with optional editing.

### Wireframe / Layout Description

Reference existing Pencil frames for the general iOS design language. New frames to create:
- Session Summary (iPhone) — scrollable card with header stats and entry list
- Session Summary (iPad) — wider layout, potentially two-column (stats left, entries right)
- Session History (iPhone) — scrollable list of session cards
- Session History (iPad) — master-detail with session list and detail pane

### Responsive Behaviour

- **iPhone**: Single column, scrollable. Summary is a full-screen view. History list fills the screen with tap-to-navigate detail.
- **iPad**: Summary can use a wider card layout. History uses split view — session list on left, detail on right.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can review and save a completed session in under 15 seconds of non-practice interaction time.
- **SC-002**: 100% of scores, tempo values, and notes entered in the summary appear correctly in the saved session history.
- **SC-003**: The session history list loads and scrolls smoothly with 50+ sessions.
- **SC-004**: The summary view displays immediately after session completion with no visible loading delay.
- **SC-005**: Discarding a session requires confirmation and permanently removes all session data.

## Assumptions

- All Crux core events for the summary phase already exist (`SaveSession`, `DiscardSession`, `UpdateEntryScore`, `UpdateEntryTempo`, `UpdateEntryNotes`, `UpdateSessionNotes`).
- `SummaryView` and `PracticeSessionView` are already in the ViewModel — no core changes needed.
- The `sessions` array in the ViewModel contains all saved sessions for the current user.
- Crash recovery handles the summary state — a force-quit during summary preserves the session for review on relaunch.
- Deleting a session uses the existing `DeleteSession` event.
- Save-as-routine functionality (web feature) is deferred — not included in this iOS feature.
