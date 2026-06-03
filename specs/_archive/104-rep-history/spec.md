# Feature Specification: Rep History Tracking

**Feature Branch**: `104-rep-history`
**Created**: 2026-02-21
**Status**: Draft
**Input**: User description: "Extend the repetition counter with full rep history tracking, preserve count on hide/show, and add icon to enable button"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Full Rep History Capture (Priority: P1)

A musician practises a passage using the rep counter. Each time they tap "Got it" or "Missed", the action is recorded in sequence. After the session, the summary shows the total number of attempts alongside the final count. In the future, analytics can use this sequence to surface patterns — a clean 5/5 run in 5 taps vs. a volatile journey of 15 taps tells a very different story about mastery.

**Why this priority**: This is the core insight the feature unlocks. Without the history, we only know the destination, not the journey. The attempt sequence is the foundation for future practice-quality analytics.

**Independent Test**: Can be tested by running a session with a rep counter, tapping Got it and Missed in various orders, completing the session, and verifying the attempt history is persisted and displayed in the summary.

**Acceptance Scenarios**:

1. **Given** an active session with a rep counter enabled on an item, **When** the user taps "Got it", **Then** a "got_it" action is appended to the item's rep history sequence.
2. **Given** an active session with a rep counter enabled on an item, **When** the user taps "Missed", **Then** a "missed" action is appended to the item's rep history sequence.
3. **Given** an item with rep history [got_it, got_it, missed, got_it, got_it, got_it], **When** the user completes the session and views the summary, **Then** the summary displays the attempt count (e.g. "6 attempts") alongside the final rep count (3/5).
4. **Given** an item with rep history, **When** the session is saved via the API, **Then** the full ordered sequence is persisted to the database and retrievable on subsequent loads.
5. **Given** an item with no rep counter enabled, **When** the session is saved, **Then** the rep history is absent (null/None) for that item.

---

### User Story 2 - Preserve Rep State on Hide/Show (Priority: P2)

A musician enables the rep counter, records several reps, then hides the counter (perhaps to focus on the timer or reduce visual clutter). When they re-enable it, their progress is still there — the count, target, and history are preserved. This lets users toggle the counter UI without fear of losing work.

**Why this priority**: Currently, disabling the counter destroys all rep state. This is a usability flaw that blocks users from freely toggling the counter. It must be fixed before the history feature ships, otherwise hiding the counter would also destroy the history.

**Independent Test**: Can be tested by enabling a counter, tapping Got it several times, disabling the counter, re-enabling it, and verifying the count and target are unchanged.

**Acceptance Scenarios**:

1. **Given** an active counter with count 3/5 and history [got_it, got_it, missed, got_it, got_it, got_it], **When** the user hides the counter, **Then** all rep state (target, count, reached, history) is preserved on the entry.
2. **Given** a hidden counter with preserved state, **When** the user shows the counter again, **Then** the counter resumes at the previous count and target with history intact.
3. **Given** an item with no prior rep state, **When** the user enables the counter for the first time, **Then** defaults are applied (target=5, count=0, reached=false, history=[]).
4. **Given** a hidden counter with count 5/5 (target reached), **When** the user shows the counter, **Then** the counter displays the reached state and the history is intact.

---

### User Story 3 - Discoverable Enable Button with Icon (Priority: P3)

The "Rep Counter" button in the active session view includes a visual icon to make it easier to recognise at a glance. This is a minor polish item that improves discoverability.

**Why this priority**: Small UX improvement. The button already exists as a proper Button component; adding an icon is incremental polish.

**Independent Test**: Can be tested by starting an active session without a rep counter and verifying the enable button renders with a visible icon alongside the label text.

**Acceptance Scenarios**:

1. **Given** an active session on an item without a rep counter, **When** the session timer view renders, **Then** the "Rep Counter" enable button displays with a recognisable icon (e.g. a repeat/cycle symbol) alongside the label.

---

### Edge Cases

- What happens when a user has an extremely long rep history (e.g. 200+ actions on one item)? The system stores all actions without truncation; display may summarise.
- What happens when the counter is hidden and the user moves to the next item? The rep state (including history) is frozen as normal when transitioning items, regardless of whether the counter is visible.
- What happens to crash recovery (localStorage) with the new history field? The history is serialised as part of the SetlistEntry and included in the crash-recovery payload.
- What happens to sessions saved before this feature? They have null/None rep_history, which is handled gracefully (displayed as no history available).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST record each "Got it" and "Missed" action as an ordered entry in the item's rep history sequence when a rep counter is active.
- **FR-002**: System MUST preserve all rep state (target, count, reached status, and history) when the user hides the rep counter during an active session.
- **FR-003**: System MUST restore preserved rep state when the user shows the rep counter again, resuming from where they left off.
- **FR-004**: System MUST only apply default rep values (target=5, count=0, history=[]) when enabling the counter on an item that has no prior rep state.
- **FR-005**: System MUST persist the full rep history sequence to the database when saving a session.
- **FR-006**: System MUST display the total attempt count from the rep history in the session summary view for each item that had an active counter.
- **FR-007**: System MUST freeze the rep history (stop accepting new actions) when transitioning away from an item (next, skip, finish, end early), consistent with existing rep state freezing.
- **FR-008**: System MUST include the rep history in the crash-recovery localStorage payload.
- **FR-009**: The "Rep Counter" enable button MUST display a recognisable icon alongside the label text.
- **FR-010**: System MUST validate rep history consistency on the API: history should only be present when rep_target is present.

### Key Entities

- **RepAction**: A single action in the rep history sequence. Either "got_it" or "missed", recorded in the order they occurred.
- **Rep History**: An ordered list of RepActions on a SetlistEntry. Absent (null) when no counter was active. Empty list when counter was enabled but no actions taken.
- **SetlistEntry** (extended): Gains a `rep_history` field alongside the existing `rep_target`, `rep_count`, and `rep_target_reached` fields.

## Design

### Existing Components Used

- **Button** (Secondary variant) — "Rep Counter" enable button, now with icon
- **Button** (Success variant) — "Got it" button
- **Button** (Secondary variant) — "Missed" button
- **Card** — Rep counter container in active session

### New Components Needed

- No new components required. The attempt count display in the session summary uses existing markup patterns (inline text alongside the existing rep count display).

### Wireframe / Layout Description

**Active session timer** — No layout changes. The "Rep Counter" enable button gains an icon prefix. The counter Card, Got it / Missed buttons, and progress bar remain unchanged.

**Session summary** — The existing rep count line (e.g. "Reps: 3 / 5 ✓") gains an attempt count suffix (e.g. "Reps: 3 / 5 ✓ · 8 attempts"). For items where the total attempts equals the target (a clean run), no attempt count is shown since it adds no information.

### Responsive Behaviour

- **Mobile**: No change from existing rep counter layout. Attempt count text wraps naturally.
- **Desktop**: No change from existing rep counter layout.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every "Got it" and "Missed" tap during an active counter is captured in the rep history with zero data loss, verifiable by comparing the history length to the total number of taps.
- **SC-002**: Hiding and re-showing the rep counter preserves all state — count, target, reached status, and history remain identical before and after the toggle.
- **SC-003**: Rep history is persisted through the full lifecycle: active session → crash recovery → API save → database → session detail retrieval.
- **SC-004**: Session summary displays attempt count for items where the attempt count differs from the target count, helping users distinguish clean runs from volatile ones.

## Assumptions

- The rep history sequence is small enough per item (typically under 50 actions) that storing it as a serialised list in a single database column is appropriate. No separate table is needed.
- The existing `freeze_rep_state()` helper is the correct place to also freeze the history.
- The hide/show toggle replaces the current enable/disable semantics. "Disable" no longer clears state; it only hides the counter UI. There is no way to truly reset the counter mid-item (users can move to the next item and come back if needed).
- The icon for the enable button is a Unicode character (e.g. 🔄 or similar) rather than an SVG icon, consistent with other inline icons in the app.
