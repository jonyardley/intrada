# Feature Specification: Tempo Tracking

**Feature Branch**: `105-tempo-tracking`
**Created**: 2026-02-24
**Status**: Draft
**Input**: User description: "Issue #52: Tempo tracking — log achieved tempo per item per session, working toward a target BPM"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Log achieved tempo during practice (Priority: P1)

A musician finishes practising a piece or exercise and wants to record the tempo (BPM) they comfortably achieved. During the session summary phase, they enter the achieved BPM for each completed entry. This data is saved alongside the existing score and notes for that entry.

**Why this priority**: Without the ability to log tempo, no other tempo feature works. This is the foundational data capture that everything else builds on.

**Independent Test**: Can be fully tested by completing a practice session, entering a BPM value on the summary screen, and verifying it persists in session history.

**Acceptance Scenarios**:

1. **Given** a completed practice session entry in the summary phase, **When** the musician enters an achieved BPM value, **Then** the value is saved with the session entry and visible in session history.
2. **Given** a session entry in the summary phase, **When** the musician leaves the achieved BPM field empty, **Then** no tempo is recorded and the entry saves normally (tempo logging is optional).
3. **Given** a session entry with an achieved BPM recorded, **When** the musician views that session in practice history, **Then** the achieved BPM is displayed alongside other entry details (score, notes, duration).

---

### User Story 2 - View tempo history for a library item (Priority: P2)

A musician navigates to a library item's detail view and wants to see how their comfortable tempo has changed over time. They see a list of their recent achieved tempos alongside dates, giving them a clear sense of whether their tempo is improving, plateauing, or regressing.

**Why this priority**: Seeing tempo progress over time is the core value proposition — it transforms a raw number into evidence that practice is working. Depends on P1 data capture being in place.

**Independent Test**: Can be tested by logging achieved tempos across multiple sessions for the same item, then viewing the item detail to confirm the tempo history is displayed chronologically.

**Acceptance Scenarios**:

1. **Given** a library item with achieved tempos logged across multiple sessions, **When** the musician views the item detail, **Then** a tempo history is displayed showing date and BPM for each recorded tempo, ordered most recent first.
2. **Given** a library item with a target BPM set and achieved tempos logged, **When** the musician views the tempo history, **Then** the target BPM is shown as a reference point alongside the history.
3. **Given** a library item with no achieved tempos logged, **When** the musician views the item detail, **Then** no tempo history section is shown (or it gracefully indicates no data yet).

---

### User Story 3 - See latest tempo on item in library list (Priority: P3)

A musician browses their library and wants a quick sense of where each item stands tempo-wise. The most recently achieved tempo is shown alongside each item in the library list, next to the target tempo if one is set.

**Why this priority**: Provides at-a-glance tempo context without navigating into each item. Lower priority because the detail view (P2) already shows this information — this is a convenience enhancement.

**Independent Test**: Can be tested by logging an achieved tempo for an item, then viewing the library list and confirming the latest achieved BPM appears next to that item.

**Acceptance Scenarios**:

1. **Given** a library item with at least one achieved tempo logged, **When** the musician views the library list, **Then** the most recently achieved BPM is displayed for that item.
2. **Given** a library item with both a target BPM and achieved tempos, **When** the musician views the library list, **Then** both the latest achieved BPM and target BPM are visible, making the gap clear.
3. **Given** a library item with no achieved tempos, **When** the musician views the library list, **Then** only the target BPM is shown (if set), with no achieved tempo displayed.

---

### Edge Cases

- What happens when a musician enters a BPM of 0 or a negative number? The system rejects it with a validation message.
- What happens when a musician enters an extremely high BPM (e.g. 999)? The system accepts values up to 500 BPM. Values above 500 are rejected with a validation message.
- What happens when a musician enters a non-numeric value? The input field only accepts numeric values.
- What happens when multiple entries in the same session reference the same item with different achieved tempos? Each entry's achieved tempo is recorded independently — they may have practised the item at different tempos during the session.
- What happens when a musician deletes a session that contained tempo data? The tempo data is removed with the session, and the item's tempo history updates accordingly.
- What happens when a musician edits an item's target tempo? The target changes going forward; historical achieved tempos remain unchanged.
- What happens when a skipped entry has an achieved tempo? Skipped entries cannot have achieved tempos — the field is only available for completed entries (same pattern as scoring).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow musicians to record an achieved BPM (integer, 1–500) per completed session entry during the summary phase.
- **FR-002**: Achieved tempo MUST be optional — entries save normally without one.
- **FR-003**: Achieved tempo MUST be persisted with the session entry data and retrievable in session history.
- **FR-004**: System MUST display a tempo history for each library item, showing all achieved tempos with session dates, ordered most recent first.
- **FR-005**: Tempo history MUST show the item's target BPM (if set) as a reference alongside achieved tempos.
- **FR-006**: System MUST display the most recently achieved BPM for an item in the library list view.
- **FR-007**: Achieved tempo input MUST only be available for completed entries (not skipped or not-attempted).
- **FR-008**: System MUST validate achieved BPM is an integer between 1 and 500 inclusive.
- **FR-009**: When a session is deleted, all associated achieved tempo data MUST be removed.
- **FR-010**: The existing target tempo on library items (the BPM field in the item's tempo metadata) serves as the target — no new target tempo concept is introduced.

### Key Entities

- **Achieved Tempo**: A BPM value (integer, 1–500) recorded per session entry, representing the comfortable tempo a musician reached during that practice block. Associated with a specific session entry and by extension a specific item.
- **Tempo History**: A derived collection of all achieved tempos for a given item across sessions, used for displaying progress. Analogous to the existing score history.
- **Target Tempo**: The existing BPM field on a library item's tempo metadata, representing the goal BPM the musician is working toward.

## Design *(include if feature has UI)*

### Existing Components Used

- **TextField** — for the achieved BPM input field in the summary phase
- **StatCard** — for displaying latest achieved tempo on item detail
- **Card** — for the tempo history section on item detail
- **TypeBadge** — already used in library list, tempo display sits alongside it

### New Components Needed

- **Tempo History List**: A chronological list within the item detail view showing date, achieved BPM, and visual relationship to target BPM for each recorded tempo. Follows the same pattern as the existing score history display.
- **Tempo Badge**: A compact display element for the library list showing achieved BPM and/or target BPM (e.g. "108 / 120 BPM" meaning achieved 108 of target 120).

### Wireframe / Layout Description

**Summary phase (achieved tempo input)**:
Below the existing score selector for each completed entry, an optional BPM input field. Label: "Achieved tempo (BPM)". Compact inline number input. Only shown for completed entries.

**Item detail (tempo history)**:
A new section within the item detail view, below or alongside the existing practice summary. Shows a list of tempo data points (date + BPM), with the target BPM shown as a reference. Follows the same visual pattern as score history.

**Library list (tempo badge)**:
Next to each item in the library list, a small tempo indicator showing the latest achieved BPM and target BPM if both exist. Uses muted styling consistent with other metadata.

### Responsive Behaviour

- **Mobile**: Tempo input field is full-width below the score selector. Tempo history list stacks vertically. Tempo badge in library list uses abbreviated format.
- **Desktop**: Tempo input field sits inline. Tempo history has more horizontal space for date and BPM columns. Tempo badge shows full format.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Musicians can record an achieved tempo for a session entry in under 5 seconds (single field, numeric input).
- **SC-002**: Musicians can see their tempo progress for any item within one tap/click from the library.
- **SC-003**: Tempo history accurately reflects all recorded tempos — no data loss on session save, delete, or app restart.
- **SC-004**: The feature adds no perceptible delay to session save or library rendering (maintains existing performance thresholds).

## Assumptions

- The existing BPM field on library items serves as the target tempo. No separate "target tempo" concept is introduced.
- Achieved tempo follows the same UX pattern as scoring: available during the summary phase for completed entries only.
- Tempo history is precomputed alongside practice summaries (using the same caching pattern established in #150) rather than computed on every render.
- The BPM range of 1–500 is generous enough for all instruments and exercise types (metronome markings rarely exceed 300, but technique exercises on some instruments can go higher).
- Full tempo progress charts (line graphs over time) are out of scope — that is #66. This feature provides data capture and list-based history display.
