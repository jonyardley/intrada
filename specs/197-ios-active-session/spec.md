# Feature Specification: iOS Active Session — Focus Mode, Timer & Scoring

**Feature Branch**: `197-ios-active-session`
**Created**: 2026-03-18
**Status**: Draft
**Input**: User description: "iOS active session — focus mode, timer and scoring"

## User Scenarios & Testing

### User Story 1 — Play Through a Session (Priority: P1)

A musician taps "Start Session" in the session builder and enters a focused practice view. They see the current item's title, a timer counting elapsed time, and can advance to the next item when ready. When they reach the last item and finish, they transition to the session summary.

**Why this priority**: This is the core practice loop — without it, sessions can be built but never played. Every other feature depends on this flow working.

**Independent Test**: Start a session with 2+ items, advance through each, finish the last one, and land on the summary placeholder.

**Acceptance Scenarios**:

1. **Given** a session is started from the builder, **When** the active session view loads, **Then** the user sees the first item's title, type badge, and a running timer.
2. **Given** the user is on item 1 of 3, **When** they tap "Next", **Then** the elapsed time is recorded for item 1, the view transitions to item 2, and the timer resets.
3. **Given** the user is on the last item, **When** they tap "Finish Session", **Then** the session status transitions to Summary and the user sees the summary view.
4. **Given** a session is active, **When** the user taps "Skip", **Then** the current item is marked as Skipped (0 seconds) and the next item loads.

---

### User Story 2 — Timer with Progress Ring (Priority: P2)

When an item has a planned duration (set in the builder), the timer displays as a circular progress ring that fills over the planned time. When the planned time elapses, a non-blocking transition prompt appears suggesting the user move on. Items without a planned duration show a simple digital timer.

**Why this priority**: The progress ring is the visual heartbeat of the practice session — it gives the musician a sense of pacing without being intrusive.

**Independent Test**: Start a session with one item that has a planned duration and one without; verify the progress ring appears only for the first and the transition prompt fires at the right time.

**Acceptance Scenarios**:

1. **Given** the current item has a planned duration of 5 minutes, **When** the timer is running, **Then** a circular progress ring fills proportionally from 0% to 100% over 5 minutes.
2. **Given** the current item has no planned duration, **When** the timer is running, **Then** a large digital timer (MM:SS) counts up with no progress ring.
3. **Given** the current item's planned duration has elapsed, **When** the timer exceeds the planned time, **Then** a non-blocking banner appears: "Up next: [Next Item Title]" or "Session complete — ready to finish?" for the last item.
4. **Given** the transition prompt is visible, **When** the user ignores it, **Then** the timer continues counting and the user can keep practising — the prompt does not block.

---

### User Story 3 — Rep Counter (Priority: P3)

When an item has a rep target (set in the builder), a rep counter appears showing current reps vs target. The musician taps "Got it" or "Missed" after each attempt. When the target is reached, the counter shows a success state. The counter can be hidden and reshown without losing state.

**Why this priority**: Rep counting supports deliberate practice for exercises and technical passages — important but not every item uses it.

**Independent Test**: Start a session with an item that has a rep target of 3; tap "Got it" 3 times; verify the target-reached state appears.

**Acceptance Scenarios**:

1. **Given** the current item has a rep target of 5, **When** the active session loads, **Then** a rep counter displays "0 / 5" with "Got it" and "Missed" buttons.
2. **Given** the rep count is 2 / 5, **When** the user taps "Got it", **Then** the count updates to 3 / 5.
3. **Given** the rep count is 2 / 5, **When** the user taps "Missed", **Then** the count decrements to 1 / 5.
4. **Given** the rep count reaches the target, **When** count equals target, **Then** the counter shows "Target reached!" and the buttons are hidden.
5. **Given** the rep counter is visible, **When** the user hides it, **Then** it collapses. When reshown, the previous count and history are preserved.

---

### User Story 4 — End Early / Abandon (Priority: P4)

A musician can end a session early (remaining items marked as Not Attempted, session saved with "ended early" status) or abandon it entirely (session discarded, return to idle). Both options require confirmation.

**Why this priority**: Life happens — musicians need a graceful exit. But it's not the primary flow.

**Independent Test**: Start a 3-item session, play through 1 item, tap "End Early", confirm, and verify the summary shows 1 completed + 2 not attempted.

**Acceptance Scenarios**:

1. **Given** a session is active, **When** the user taps "End Early", **Then** a confirmation prompt appears: "End session early? Remaining items will be marked as not attempted."
2. **Given** the user confirms "End Early", **Then** remaining items are marked NotAttempted, the session transitions to Summary with CompletionStatus::EndedEarly.
3. **Given** a session is active, **When** the user taps "Abandon", **Then** a confirmation prompt appears: "Abandon session? All progress will be lost."
4. **Given** the user confirms "Abandon", **Then** the session is discarded, the status returns to Idle, and crash recovery data is cleared.

---

### User Story 5 — Crash Recovery (Priority: P5)

If the app is terminated during an active session (killed, crash, or phone restart), the session state is recovered on next launch. The user returns to their active session at the item they were on, with all progress preserved.

**Why this priority**: Important for trust, but only matters in edge cases. The core already handles persistence — this story is about the iOS shell correctly restoring UI state.

**Independent Test**: Start a session, advance to item 2, force-quit the app, relaunch, and verify the active session resumes at item 2.

**Acceptance Scenarios**:

1. **Given** a session is active at item 2 of 3, **When** the app is force-quit and relaunched, **Then** the Practice tab shows the active session view at item 2.
2. **Given** the app relaunches with a recovered session, **When** the timer starts, **Then** it begins from 0:00 (the timer is UI-only; only recorded durations from completed items persist).
3. **Given** the user abandons the recovered session, **When** they tap Abandon and confirm, **Then** the crash recovery data is cleared and the session returns to Idle.

---

### Edge Cases

- What happens when the session has only 1 item? "Next" should not appear; only "Finish Session" is shown.
- What happens when all items are skipped? The session completes normally and transitions to Summary with all items marked Skipped.
- What happens when the user switches tabs during an active session? The timer continues counting (the tab bar shows an activity indicator). Returning to the Practice tab shows the active session.
- What happens on iPad? The active session view uses the full screen (no split view — focus mode means minimal distractions).

## Requirements

### Functional Requirements

- **FR-001**: System MUST display the current item's title, type (Piece/Exercise), position (e.g., "2 of 5"), and a running elapsed timer.
- **FR-002**: System MUST provide "Next" navigation to advance to the next item, recording the elapsed time for the current item.
- **FR-003**: System MUST provide "Skip" to skip the current item (0 seconds recorded, status = Skipped) and advance.
- **FR-004**: System MUST show "Finish Session" instead of "Next" on the last item, transitioning to Summary on tap.
- **FR-005**: System MUST display a circular progress ring when the current item has a planned duration, filling proportionally over the planned time.
- **FR-006**: System MUST display a digital timer (MM:SS) when the current item has no planned duration.
- **FR-007**: System MUST show a non-blocking transition prompt when the planned duration elapses ("Up next: [Title]" or "Session complete — ready to finish?").
- **FR-008**: System MUST display a rep counter (current / target) with "Got it" and "Missed" buttons when the current item has a rep target.
- **FR-009**: System MUST show a "Target reached!" state when the rep count equals or exceeds the target, hiding the action buttons.
- **FR-010**: System MUST allow the rep counter to be hidden and reshown without losing state.
- **FR-011**: System MUST provide "End Early" to finish the session with remaining items marked as NotAttempted and CompletionStatus::EndedEarly.
- **FR-012**: System MUST provide "Abandon" to discard the session entirely, returning to Idle and clearing crash recovery data.
- **FR-013**: Both "End Early" and "Abandon" MUST require user confirmation before executing.
- **FR-014**: System MUST display the session intention (if set) and the current item's intention (if set).
- **FR-015**: System MUST show an activity indicator on the Practice tab when a session is active (accent dot or icon colour change).
- **FR-016**: System MUST persist active session state to local storage after each navigation/rep action for crash recovery.
- **FR-017**: System MUST restore the active session on app relaunch, resuming at the correct item with all completed-item progress preserved.
- **FR-018**: Timer MUST reset to 0:00 on crash recovery (timer is UI-only; only recorded durations from completed items persist).
- **FR-019**: The active session view MUST use the full screen on both iPhone and iPad (no split view — focus mode).
- **FR-020**: System MUST display a list of completed items below the active item (collapsible for focus).

### Key Entities

- **ActiveSession**: The in-progress session with entries, current index, timestamps, and optional intention.
- **SetlistEntry**: Individual item in the setlist with status (Completed/Skipped/NotAttempted), recorded duration, rep state, and optional score.
- **RepAction**: A single rep event (Success or Missed) forming a history sequence.
- **TransitionPrompt**: A non-blocking UI overlay triggered when planned duration elapses.

## Design

### Existing Components Used

- `CardView` — Container for session info, rep counter, completed items
- `ButtonView` — Primary (Next/Finish), Secondary (Skip), Danger (End Early), DangerOutline (Abandon)
- `TypeBadge` — Piece/Exercise indicator on current item
- `Toast` — Confirmation/error feedback
- `PageHeading` — Current item title (serif heading)

### New Components Needed

- **ProgressRingView**: Circular progress indicator that fills over the planned duration. Shows elapsed time in the centre (MM:SS). Animates smoothly. Falls back to a static ring at 100% when time exceeds plan.
- **TransitionPromptView**: Non-blocking banner at the bottom of the screen showing the next item's title or "Session complete" message. Dismissible but auto-appears. Translucent background, slide-up animation.
- **RepCounterView**: Displays "X / Y" with "Got it" (success) and "Missed" (secondary) buttons. Shows a small progress bar or fill. Collapses to a single line when hidden. Shows "Target reached!" celebratory state.
- **SessionProgressBar**: Compact indicator showing position in the session (e.g., dots or segments for each item, current highlighted).
- **CompletedItemsList**: Collapsible list of items already practised, showing title, duration, and status badge (Completed/Skipped).

### Wireframe / Layout Description

**iPhone layout** (full screen, no tab bar during active session):
1. **Top area**: Back/exit button (top-left), session progress indicator (dots/segments, top-centre)
2. **Main content** (centred, dominant):
   - Current item title (large, serif heading)
   - Type badge + position ("2 of 5")
   - Timer / Progress Ring (large, centred)
   - Entry intention (if set, below timer)
3. **Rep counter** (below timer, if applicable): Count display + Got it / Missed buttons
4. **Transition prompt** (bottom overlay, non-blocking): Slides up when planned time elapses
5. **Action buttons** (bottom): Next/Finish Session (primary), Skip (secondary)
6. **Completed items** (scrollable below, collapsible): List of done items with duration

**iPad layout**: Same as iPhone but with more generous spacing. Full screen — no split view. The session is a focused, distraction-free experience.

Pencil mockups to be created: "iOS / Active Session", "iOS / Active Session (Transition Prompt)"

### Responsive Behaviour

- **iPhone**: Full-screen focus mode. Tab bar hidden during active session. All controls within thumb reach at bottom.
- **iPad**: Full-screen focus mode. Same layout as iPhone but with more whitespace and larger timer/progress ring. No split view.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A musician can complete a 3-item session (start, advance through items, finish) in under 5 taps beyond the practice time itself.
- **SC-002**: The transition prompt appears within 1 second of the planned duration elapsing.
- **SC-003**: Crash recovery restores the active session within 2 seconds of app relaunch, with all completed-item data intact.
- **SC-004**: The rep counter correctly tracks consecutive successes and decrements on misses, reaching the target state at the correct count 100% of the time.
- **SC-005**: The Practice tab activity indicator is visible from all other tabs whenever a session is active.

## Assumptions

- The Crux core already implements all session events (NextItem, SkipItem, FinishSession, EndSessionEarly, AbandonSession, RepGotIt, RepMissed, InitRepCounter) — this feature is a pure iOS shell implementation.
- Timer is client-side — elapsed time is not part of the ViewModel. Duration is calculated and sent with NextItem/SkipItem events.
- Crash recovery is handled by the core via KeyValue capability mapped to UserDefaults — the shell just needs to call startApp() on launch which triggers recoverSession.
- The Summary view (US1 "Finish Session" transition) will be a placeholder until #198 is implemented.
- Scoring (per-item scores) happens in the Summary phase (#198), not during the active session.
