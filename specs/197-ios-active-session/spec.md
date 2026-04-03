# Feature Specification: iOS Active Session — Focus Mode, Timer & Scoring

**Feature Branch**: `197-ios-active-session`
**Created**: 2026-03-21
**Status**: Draft
**Input**: iOS active session — focus mode with countdown timer, progress ring, per-item scoring (1-5), tempo input, rep counter, item transitions, pause/resume, end early, and crash recovery. State-driven rendering on Practice tab (Active status). Tab bar shows accent indicator during active session. iPhone and iPad layouts.

## User Scenarios & Testing

### User Story 1 — Play Through a Session (Priority: P1)

A musician starts a session from the builder, sees the current item with a running timer, and advances through each item until the session is complete. The active session view is a focused, minimal-UI experience — no navigation bars or tab bars visible. When the last item finishes, the app transitions to the session summary.

**Why this priority**: This is the core practice experience — without it, the app cannot deliver its primary value. Everything else builds on top of this foundation.

**Independent Test**: Start a 3-item session → see first item displayed with timer → tap Next → see second item → tap Next → see third item → tap Finish → land on session summary screen.

**Acceptance Scenarios**:

1. **Given** a session with 3 items has been started, **When** the active session view loads, **Then** the current item's title, type, and position ("1 of 3") are displayed alongside a running timer.
2. **Given** item 1 is active, **When** the user taps "Next", **Then** item 2 becomes active, the timer resets, and the position updates to "2 of 3".
3. **Given** the last item is active, **When** the user taps "Finish", **Then** the session completes and the app transitions to the session summary view.
4. **Given** the active session view is displayed, **When** looking at the screen, **Then** the navigation bar and tab bar are hidden (focus mode).
5. **Given** an active session, **When** the user views the tab bar from another tab, **Then** the Practice tab shows an accent-coloured indicator (dot or icon colour) signalling a live session.

---

### User Story 2 — Progress Ring Timer (Priority: P2)

Items with a planned duration show a circular progress ring that fills as time passes. When the timer reaches zero, a gentle prompt suggests moving to the next item. Items without a planned duration show a simple elapsed time display (MM:SS) without a ring.

**Why this priority**: The progress ring is the primary visual feedback mechanism during practice. It externalises time perception — critical for neurodivergent users who struggle with time blindness.

**Independent Test**: Start a session with a 5-minute item → see the progress ring animate over 5 minutes → when timer hits zero, "Up next..." prompt appears. Start a session with an item that has no planned duration → see a simple elapsed timer instead of a ring.

**Acceptance Scenarios**:

1. **Given** an item with a 5-minute planned duration, **When** the timer is running, **Then** a circular progress ring fills from 0% to 100% over 5 minutes, with remaining time displayed in the centre.
2. **Given** an item's timer has reached zero, **When** time expires, **Then** a transition prompt slides in showing "Up next: [next item title]" with a "Continue" button.
3. **Given** the last item's timer has reached zero, **When** time expires, **Then** the prompt shows "Session complete — ready to finish?" with a "Finish" button.
4. **Given** an item without a planned duration, **When** the timer is running, **Then** a simple MM:SS elapsed time display is shown (no ring, no countdown).
5. **Given** any active item, **When** the user looks at the timer, **Then** the elapsed time is always visible regardless of whether a ring is shown.

---

### User Story 3 — Per-Item Scoring & Feedback (Priority: P3)

After completing each item (advancing to the next or finishing the session), the user can optionally rate their confidence (1–5 stars/dots), record the tempo they achieved (BPM), and add notes. These inputs are optional — the user can skip them and move on immediately.

**Why this priority**: Scoring and tempo tracking are how the app learns about progress over time. Without them, the analytics and practice summaries have no data. But they're optional to keep friction low.

**Independent Test**: Complete an item → see score input (1–5) → tap a score → optionally enter tempo → tap Next to continue. Or skip scoring entirely and tap Next immediately.

**Acceptance Scenarios**:

1. **Given** the user taps "Next" on an item, **When** the transition prompt appears, **Then** it includes an optional 1–5 confidence score selector.
2. **Given** the transition prompt is showing, **When** the user taps a score (e.g., 4), **Then** the score is recorded for that item and visually confirmed.
3. **Given** the transition prompt is showing, **When** the user taps "Continue" without selecting a score, **Then** the session advances without recording a score (no error, no nag).
4. **Given** the transition prompt is showing, **When** the user optionally enters a tempo (BPM number), **Then** the tempo is recorded for that item.
5. **Given** the transition prompt is showing, **When** the user optionally adds notes, **Then** the notes are saved for that item.

---

### User Story 4 — Rep Counter (Priority: P4)

Items with a rep target show a counter (e.g., "0 / 5") with "Got it" and "Missed" buttons. Each tap increments or decrements the count. When the target is reached, a brief celebration state is shown. The counter can be hidden and reshown without losing progress.

**Why this priority**: Reps are essential for technical exercises but not needed for repertoire pieces. This is an enhancement that makes exercise practice more structured.

**Independent Test**: Start a session with an exercise that has a rep target of 5 → see "0 / 5" counter → tap "Got it" 3 times → counter shows "3 / 5" → tap "Missed" once → counter shows "2 / 5" → tap "Got it" 3 more times → counter shows "5 / 5" with celebration.

**Acceptance Scenarios**:

1. **Given** an item with a rep target of 5, **When** the item becomes active, **Then** a rep counter displays "0 / 5" with "Got it" and "Missed" buttons.
2. **Given** the rep counter shows "2 / 5", **When** the user taps "Got it", **Then** the counter updates to "3 / 5".
3. **Given** the rep counter shows "3 / 5", **When** the user taps "Missed", **Then** the counter updates to "2 / 5" (minimum 0).
4. **Given** the rep counter reaches the target (e.g., "5 / 5"), **When** the target is met, **Then** a brief celebration state is shown (checkmark, colour change, or subtle animation).
5. **Given** an item without a rep target, **When** the item becomes active, **Then** no rep counter is displayed.

---

### User Story 5 — End Early & Abandon (Priority: P5)

The user can end a session before completing all items. "End Early" saves the session with remaining items marked as not attempted. "Abandon" discards the entire session without saving. Both require confirmation to prevent accidental data loss.

**Why this priority**: Users need an escape hatch — practice doesn't always go as planned. But saving partial progress is more valuable than losing everything.

**Independent Test**: During item 2 of 4 → tap "End Early" → confirmation appears → confirm → session summary shows 2 completed, 2 not attempted. Alternatively: tap "Abandon" → confirmation → session discarded, return to idle.

**Acceptance Scenarios**:

1. **Given** an active session, **When** the user taps "End Early", **Then** a confirmation dialog appears asking "End this session? Completed items will be saved."
2. **Given** the "End Early" confirmation is showing, **When** the user confirms, **Then** the session is saved with remaining items marked as not attempted, and the app transitions to session summary.
3. **Given** an active session, **When** the user taps "Abandon", **Then** a confirmation dialog appears warning "Discard this session? All progress will be lost."
4. **Given** the "Abandon" confirmation is showing, **When** the user confirms, **Then** the session is discarded and the Practice tab returns to the idle state.
5. **Given** either confirmation dialog, **When** the user cancels, **Then** the dialog dismisses and the session continues uninterrupted.

---

### Edge Cases

- What happens when the app is force-quit during an active session? The session state is recovered on relaunch via crash recovery. The timer resets to zero but all completed items are preserved.
- What happens when the user switches to another tab during an active session? The tab bar indicator shows the session is still active. Returning to the Practice tab resumes the session exactly where they left it.
- What happens when a session has only one item? The "Next" button is replaced with "Finish" from the start.
- What happens when the user's phone locks during a session? The timer continues counting in the background (or resumes from the persisted state on return).
- What happens when scoring a 0 or leaving the score empty? Both are valid — no score is recorded, which is different from a score of 1.

## Requirements

### Functional Requirements

- **FR-001**: The active session view MUST display the current item's title, type badge, position ("X of Y"), and a running timer.
- **FR-002**: The active session view MUST hide the navigation bar and tab bar (focus mode). A minimal control bar provides session actions.
- **FR-003**: Items with a planned duration MUST display a circular progress ring that fills over time, with remaining time in the centre.
- **FR-004**: Items without a planned duration MUST display a simple elapsed time (MM:SS) without a progress ring.
- **FR-005**: When an item's planned time expires, a transition prompt MUST appear showing the next item title and optional scoring inputs.
- **FR-006**: The user MUST be able to advance to the next item at any time by tapping "Next" (not blocked by timer).
- **FR-007**: The user MUST be able to optionally rate confidence (1–5), enter achieved tempo (BPM), and add notes during item transitions.
- **FR-008**: Items with a rep target MUST display a counter with "Got it" / "Missed" buttons. The counter MUST not go below 0.
- **FR-009**: When the rep target is reached, the UI MUST show a brief celebration state.
- **FR-010**: The user MUST be able to end the session early (saving completed items) with confirmation.
- **FR-011**: The user MUST be able to abandon the session (discarding all data) with confirmation.
- **FR-012**: The Practice tab MUST show a visual indicator (accent dot or icon colour) when a session is active.
- **FR-013**: Session state MUST be persisted for crash recovery. On relaunch, the active session resumes at the correct item with completed items preserved.
- **FR-014**: The active session view MUST support both iPhone and iPad layouts. On iPad, the larger screen can show more detail (e.g., session intention, upcoming items) alongside the current item.
- **FR-015**: When the last item is completed, the app MUST transition to the session summary view.

## Design

### Existing Components Used

- `ProgressRing` — circular progress indicator (may need adaptation for countdown mode)
- `TypeBadge` — item type pill (Piece/Exercise)
- `ButtonView` — action buttons (Next, Finish, Got it, Missed, End Early)
- `CardView` — container for transition prompt / scoring inputs
- `Toast` — feedback notifications (e.g., "Session saved")

### New Components Needed

- **ActiveSessionView**: Main focus-mode screen — current item display, timer/ring, session progress, minimal controls. Hides nav/tab bars.
- **TransitionPromptView**: Overlay/sheet between items — next item preview, optional score selector (1–5), optional tempo input, optional notes, "Continue" button.
- **RepCounterView**: "0 / 5" display with "Got it" / "Missed" buttons. Shows celebration state when target reached.
- **SessionProgressBar**: Compact indicator showing position in session (e.g., segmented bar, dots, or "2 of 5").
- **ConfirmationSheet**: Reusable confirmation dialog for End Early / Abandon actions.

### Wireframe / Layout Description

Reference existing Pencil frames in `design/intrada.pen` for the general iOS design language. New frames should be created for:
- Active session (iPhone) — focus mode with timer/ring
- Active session (iPad) — wider layout with session context
- Transition prompt — between-item scoring overlay
- Rep counter — inline or floating counter display

### Responsive Behaviour

- **iPhone**: Full-screen focus mode. Timer/ring dominates the upper portion. Item title and controls below. Transition prompt slides up as a sheet.
- **iPad**: Same focus mode but with optional sidebar showing session overview (upcoming items, session intention, elapsed time). Transition prompt can be a centered card rather than a bottom sheet.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can complete a 5-item session from start to summary in under 30 seconds of non-practice interaction time (tapping Next/Finish between items).
- **SC-002**: The timer display updates smoothly (at least once per second) with no visible jank or freezing.
- **SC-003**: Crash recovery restores the session to the correct item within 2 seconds of app relaunch.
- **SC-004**: 100% of scored items appear correctly in the session summary and analytics views.
- **SC-005**: The focus mode hides all non-essential UI — the user sees only the current item, timer, and session controls during practice.

## Assumptions

- The Crux core already implements all active session events (NextItem, SkipItem, FinishSession, EndSessionEarly, AbandonSession, RepGotIt, RepMissed, UpdateEntryScore, UpdateEntryTempo, UpdateEntryNotes). No core changes are needed.
- Crash recovery uses the existing `UserDefaults` persistence via `IntradaCore.swift` effect processing.
- The timer is a shell-local concern (SwiftUI `Timer` publisher) — the Crux core provides `current_planned_duration_secs` but the shell tracks elapsed time locally.
- Scoring, tempo, and notes are dispatched to the core immediately on input (not batched).
- The transition prompt appears automatically when a timed item expires, but the user can also manually trigger "Next" at any time.
- iPad layout is an enhancement — the core experience (iPhone) is the MVP.
