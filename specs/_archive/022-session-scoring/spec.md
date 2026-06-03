# Feature Specification: Session Item Scoring

**Feature Branch**: `022-session-scoring`
**Created**: 2026-02-17
**Status**: Draft
**Input**: User description: "A user should be able to score each item at the end of a session. This data will be used to track progress of items over time"

## Clarifications

### Session 2026-02-17

- Q: What does the score represent — session quality, confidence/mastery, or difficulty? → A: Confidence/mastery — "How confident do I feel with this piece?" (cumulative self-assessment)
- Q: How should the progress summary present scores visually? → A: Chronological list with the most recent score highlighted prominently
- Q: Should the latest confidence score surface on the library list view or only the item detail page? → A: Detail page only — library list stays clean

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Score Items During Session Review (Priority: P1)

After completing a practice session, during the summary review screen, a user assigns a confidence score to each item they practised. The score reflects how confident they feel with that item — a cumulative self-assessment of mastery (e.g., "How confident do I feel with this piece?"). The user then saves the session with scores included.

**Why this priority**: This is the core interaction — without the ability to assign scores, no progress data can be collected. Everything else builds on this.

**Independent Test**: Can be fully tested by completing a practice session with multiple items, assigning different scores to each item on the summary screen, saving the session, and verifying scores are persisted when the session is viewed again.

**Acceptance Scenarios**:

1. **Given** a user is on the session summary screen after practising, **When** they view the list of completed entries, **Then** each entry with status "completed" displays a scoring control defaulting to no score selected.
2. **Given** a user is on the session summary screen, **When** they assign a score to a completed entry, **Then** the selected score is visually indicated and associated with that entry.
3. **Given** a user has assigned scores to some or all entries, **When** they save the session, **Then** the scores are persisted alongside each entry.
4. **Given** a user has entries with status "skipped" or "not attempted", **When** they view the summary screen, **Then** those entries do not display a scoring control (scores only apply to completed entries).
5. **Given** a user chooses not to score a completed entry, **When** they save the session, **Then** the entry is saved with no score (scoring is optional per entry).

---

### User Story 2 - View Scores on Past Sessions (Priority: P2)

A user views a previously completed session and sees the scores they assigned to each item. This provides a historical record of how they felt about each practice.

**Why this priority**: Users need to recall past self-assessments to understand their journey. This is the basic read-back of stored data.

**Independent Test**: Can be fully tested by viewing a saved session that has scores and confirming each entry displays its recorded score alongside duration and status.

**Acceptance Scenarios**:

1. **Given** a user views a completed session that has scored entries, **When** the session detail loads, **Then** each entry displays its score alongside existing information (duration, status, notes).
2. **Given** a user views a completed session where some entries were not scored, **When** the session detail loads, **Then** unscored entries show no score indicator (not a zero or default).
3. **Given** a user views a session completed before scoring was available, **When** the session detail loads, **Then** entries display without scores (backward compatible).

---

### User Story 3 - Track Item Progress Over Time (Priority: P3)

A user views a library item (piece or exercise) and sees a summary of their scoring history — how their scores have trended across sessions. This helps them understand whether they are improving, plateauing, or struggling with a particular item.

**Why this priority**: This is the downstream value of collecting scores — turning data into insight. It depends on scores being collected (P1) and readable (P2).

**Independent Test**: Can be fully tested by practising the same item across multiple sessions with varying scores, then viewing that item's detail page and confirming the progress summary reflects all recorded scores in chronological order.

**Acceptance Scenarios**:

1. **Given** a library item has been scored in multiple sessions, **When** the user views that item's detail page, **Then** they see the most recent confidence score displayed prominently, followed by a chronological list of all past scores with session dates.
2. **Given** a library item has been practised but never scored, **When** the user views that item's detail page, **Then** the progress section indicates no scores are available yet.
3. **Given** a library item appears multiple times in a single session (e.g., warm-up and main practice), **When** the user views progress, **Then** each scored occurrence is included as a separate data point.
4. **Given** a library item has only one scored session, **When** the user views the progress summary, **Then** the single score is displayed prominently as the latest score, with no additional history rows.

---

### Edge Cases

- What happens when a user saves a session with zero entries scored? The session saves normally; scoring is entirely optional.
- What happens when the same library item appears multiple times in one session with different scores? Each entry is independent — both scores are stored and both appear in the item's progress history.
- What happens when a library item is deleted after sessions with scores exist? Scores remain on the session entries (item data is already snapshotted). The item's progress view is no longer accessible, but session history retains the scores.
- What happens when viewing progress for an item that has only "skipped" or "not attempted" entries across sessions? The progress section shows no scores available, since only completed entries can be scored.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow users to assign a score to each session entry with status "completed" during the session summary review.
- **FR-002**: System MUST use a 1–5 numeric scale for confidence scores, where 1 represents the lowest confidence ("not confident at all") and 5 represents the highest ("fully confident / mastered").
- **FR-003**: Scoring MUST be optional — users may save a session without scoring any or all completed entries.
- **FR-004**: System MUST persist the score as part of the session entry when the session is saved.
- **FR-005**: System MUST display stored scores when viewing a previously completed session.
- **FR-006**: System MUST remain backward compatible — sessions saved before scoring was introduced display normally without score information.
- **FR-007**: System MUST display a progress summary on each library item's detail page, showing the most recent confidence score prominently, followed by a chronological history of all past scores.
- **FR-008**: System MUST NOT allow scoring of entries with status "skipped" or "not attempted".
- **FR-009**: When a library item appears multiple times in a single session, each occurrence MUST be independently scoreable.
- **FR-010**: The progress summary for a library item MUST include the date of each session and the score assigned, ordered chronologically.
- **FR-011**: Confidence scores MUST NOT appear on the library list view — progress information is only visible on the individual item detail page.

### Key Entities

- **Confidence Score**: A numeric value (1–5) representing a user's self-assessed confidence or mastery of a specific item, recorded at the end of a session. Attached to a single setlist entry. Optional (may be absent).
- **Item Progress**: An aggregated, chronological view of all scores for a given library item across sessions. Derived from scored setlist entries that reference that item.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can score all completed items in a session and save within 30 seconds of additional interaction beyond the current summary flow.
- **SC-002**: 100% of saved scores are accurately displayed when the session is viewed again.
- **SC-003**: Progress history for a library item correctly reflects all scored sessions, with no data loss or duplication.
- **SC-004**: Sessions saved before this feature was introduced continue to display correctly with no visual artefacts or errors.

## Assumptions

- The 1–5 numeric scale is appropriate for a quick confidence self-assessment during session review. This is a common pattern in practice journals and spaced-repetition tools. Labels for the scale endpoints (e.g., 1 = "Not confident", 5 = "Mastered") can be determined during design/planning.
- Progress tracking is read-only — users cannot edit scores on past sessions after saving. If retroactive editing is needed, it would be a separate feature.
- The progress summary on the item detail page shows the latest score prominently plus a chronological list — not a chart or graph. Visual enhancements can be layered on later.
- Scores are personal and not shared or compared between users (single-user context, consistent with the existing app model).
