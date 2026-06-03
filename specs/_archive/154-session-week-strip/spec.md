# Feature Specification: Session Week Strip Navigator

**Feature Branch**: `154-session-week-strip`
**Created**: 2026-03-04
**Status**: Draft
**Input**: User description: "Issue #65 — Sessions page redesign with week strip navigator. Replace the current flat session list with a weekly calendar strip view."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse sessions by week (Priority: P1)

A musician opens the Sessions page and sees a horizontal week strip showing Monday through Sunday with date numbers. Days that had practice sessions display a small dot below the date. Today's date is selected by default and any sessions from that day are displayed below the strip. If today has no sessions, the most recent day with sessions in the current week is auto-selected instead.

**Why this priority**: This is the core navigation paradigm replacing the flat list. Without it, the feature has no value.

**Independent Test**: Can be fully tested by navigating to /sessions and verifying the week strip renders with correct dates, dot indicators, and day selection showing session cards.

**Acceptance Scenarios**:

1. **Given** the user has sessions on Monday, Wednesday, and Friday this week, **When** they open the Sessions page, **Then** they see a week strip with dots under Monday, Wednesday, and Friday, and today's sessions (or the most recent session day) are displayed below.
2. **Given** the user has no sessions this week, **When** they open the Sessions page, **Then** they see a week strip with no dots and an empty state message below.
3. **Given** the user has sessions today, **When** they open the Sessions page, **Then** today is selected by default and today's sessions are shown.
4. **Given** the user has no sessions today but has a session on Wednesday, **When** they open the Sessions page (on Thursday), **Then** Wednesday is auto-selected and Wednesday's sessions are shown.

---

### User Story 2 - Select a day to view its sessions (Priority: P1)

The musician taps a day in the week strip to see all sessions from that day. Each session card shows the start time, total duration, items practised with their confidence scores, and other session metadata. Tapping a session card navigates to the existing session detail page.

**Why this priority**: Day selection is the primary interaction — it makes the week strip useful beyond a static display.

**Independent Test**: Can be tested by clicking different days in the strip and verifying the correct sessions appear below, then tapping a session card to confirm navigation.

**Acceptance Scenarios**:

1. **Given** Monday has two sessions, **When** the user taps Monday in the week strip, **Then** two session cards appear below showing start time, duration, items, and scores for each session.
2. **Given** Tuesday has no sessions, **When** the user taps Tuesday, **Then** an empty state message appears (e.g. "No sessions on this day").
3. **Given** a session card is displayed, **When** the user taps the card, **Then** they navigate to the session detail page for that session.

---

### User Story 3 - Navigate between weeks (Priority: P2)

The musician can move to previous or future weeks using left/right arrow buttons or by swiping horizontally on mobile. When navigating to a new week, the strip updates to show that week's dates and session indicators. The first day with sessions in the new week is auto-selected.

**Why this priority**: Without week navigation, users can only see the current week. This unlocks the full practice history.

**Independent Test**: Can be tested by tapping the left arrow to go back one week, verifying the dates change, and confirming sessions for the previous week are accessible.

**Acceptance Scenarios**:

1. **Given** the user is viewing the current week, **When** they tap the left arrow, **Then** the strip shows the previous week's dates with correct session dot indicators.
2. **Given** the user is viewing a past week, **When** they tap the right arrow, **Then** the strip moves forward one week.
3. **Given** the user navigates to a week with sessions on Tuesday and Thursday, **When** the week loads, **Then** the first day with sessions (Tuesday) is auto-selected.
4. **Given** the user is on mobile, **When** they swipe left on the week strip, **Then** it navigates to the next week.
5. **Given** the user is on mobile, **When** they swipe right on the week strip, **Then** it navigates to the previous week.

---

### User Story 4 - Access full session list (Priority: P3)

A musician who wants to see all their sessions chronologically can tap a "Show all sessions" link below the week view. This reveals the existing flat list of all sessions, providing a fallback for users who prefer browsing without the week constraint.

**Why this priority**: Provides a familiar fallback for users who prefer the full list. Lower priority because the week view should satisfy most use cases.

**Independent Test**: Can be tested by tapping "Show all sessions" and verifying the full chronological list appears.

**Acceptance Scenarios**:

1. **Given** the user is on the Sessions page with the week strip visible, **When** they tap "Show all sessions", **Then** the full chronological session list is displayed (same as the current sessions page layout).
2. **Given** the user is viewing the full list, **When** they want to return to the week view, **Then** they can navigate back (e.g. via a "Week view" link or the back action).

---

### Edge Cases

- What happens when the user has never recorded a session? The week strip shows the current week with no dots and an empty state message encourages them to start practising.
- What happens when a day has more sessions than fit on screen? Sessions stack vertically and the user scrolls the session area (the week strip stays in place).
- What happens at week boundaries with timezones? Dates are based on the session's `started_at` timestamp in the user's local browser timezone.
- What happens when navigating far into the past? Week navigation continues to work with no hard limit on how far back the user can go.
- What happens with the right arrow when viewing the current week? The right arrow remains enabled, allowing navigation to future (empty) weeks for simplicity.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Sessions page MUST display a week strip showing seven days (Monday through Sunday) with date numbers.
- **FR-002**: Each day in the week strip MUST show a small dot indicator if one or more sessions were recorded on that day.
- **FR-003**: The week strip MUST default to the current week when the page loads.
- **FR-004**: The system MUST auto-select today if today has sessions, otherwise the most recent day with sessions in the current week, otherwise today.
- **FR-005**: Tapping a day in the week strip MUST display all sessions from that day below the strip, ordered chronologically (earliest session first).
- **FR-006**: Each session card MUST display: start time, total duration, items practised with status icons, confidence scores, tempo, rep targets, intention, and notes (matching the current session row detail level).
- **FR-007**: Tapping a session card MUST navigate to the existing session detail/review page.
- **FR-008**: The week strip MUST have left and right arrow buttons to navigate to the previous and next week respectively.
- **FR-009**: On mobile devices, the week strip MUST support swipe gestures (swipe left for next week, swipe right for previous week).
- **FR-010**: When navigating to a new week, the week strip MUST update immediately with the new dates, and session cards MUST show a brief loading skeleton before displaying the filtered sessions. The system MUST auto-select the first day in that week that has sessions, or default to Monday if no sessions exist.
- **FR-011**: A "Show all sessions" link MUST be displayed below the week view, navigating to the full chronological session list.
- **FR-012**: The selected day MUST be visually distinguished from unselected days in the week strip.
- **FR-013**: When a selected day has no sessions, the system MUST display an appropriate empty state message.
- **FR-014**: Session cards MUST include the existing delete functionality with confirmation.
- **FR-015**: The "New Session" button MUST remain accessible at the top of the page.

### Key Entities

- **Week**: A seven-day period (Monday–Sunday) used as the navigation unit. Identified by the date of its Monday.
- **Day Cell**: A single day within the week strip, displaying the day name, date number, selection state, and session indicator dot.
- **Session Card**: A card displaying a single practice session's summary data (reuses the existing session row content).

## Design

### Existing Components Used

- **PageHeading** — page title and subtitle at the top
- **Card** — container for each session card (reuses existing session row content)
- **Button** — delete confirmation actions within session cards
- **Icon** — status icons for session entries (check, ban, minus)
- **SkeletonBlock** / **SkeletonLine** — loading states for the week strip and session cards

### New Components Needed

- **WeekStrip**: A horizontal strip displaying seven day cells (Mon–Sun) with left/right navigation arrows. Supports tap selection on individual days and swipe gestures on mobile. Shows the month/year label for context. When a week spans two months, the label shows both (e.g. "Feb – Mar 2026").
- **DayCell**: A single cell within the WeekStrip showing: abbreviated day name (M, T, W, T, F, S, S), date number, selected state highlight, and a session indicator dot.

### Wireframe / Layout Description

```
┌─────────────────────────────────────────────────┐
│  Practice Sessions          [New Session]       │
│  Review your practice history...                │
├─────────────────────────────────────────────────┤
│              ◄   March 2026   ►                 │
│  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┐   │
│  │  M  │  T  │  W  │  T  │  F  │  S  │  S  │   │
│  │  2  │  3  │ [4] │  5  │  6  │  7  │  8  │   │
│  │     │  ·  │     │  ·  │     │     │     │   │
│  └─────┴─────┴─────┴─────┴─────┴─────┴─────┘   │
├─────────────────────────────────────────────────┤
│  ┌─ Session Card ───────────────────────────┐   │
│  │ 25 min · 3 items       4 Mar 2026, 14:30│   │
│  │ ✓ Bach Prelude No. 1   10 min      4/5  │   │
│  │ ✓ Scales in C Major     8 min      3/5  │   │
│  │ ✗ Sight Reading         7 min            │   │
│  │                                [Delete]  │   │
│  └──────────────────────────────────────────┘   │
│                                                 │
│  ┌─ Session Card ───────────────────────────┐   │
│  │ 15 min · 2 items       4 Mar 2026, 09:00│   │
│  │ ...                                      │   │
│  └──────────────────────────────────────────┘   │
│                                                 │
│            Show all sessions →                  │
└─────────────────────────────────────────────────┘
```

- The week strip sits directly below the page heading.
- The month/year label is centred between the left and right navigation arrows.
- The selected day ([4] in the wireframe) is visually highlighted.
- Session cards stack vertically below the strip for the selected day.
- "Show all sessions" link appears at the bottom.

### Responsive Behaviour

- **Mobile**: Week strip spans full width. Day cells are evenly distributed. Navigation uses both arrows and swipe gestures. Session cards are full-width.
- **Desktop**: Week strip sits within the content area with comfortable spacing. Arrows are always visible. Swipe is supported but not the primary interaction.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can identify which days they practised in a given week within 2 seconds of the page loading.
- **SC-002**: Users can navigate to any session from the past 4 weeks in 3 taps or fewer (week arrows + day select + card tap).
- **SC-003**: The week strip renders correctly for weeks with 0, 1, and 7 sessions spread across different days.
- **SC-004**: All existing session data (duration, items, scores, statuses, notes, intentions) remains accessible through the new view.
- **SC-005**: The full session list remains accessible via the "Show all sessions" link for users who prefer chronological browsing.

## Clarifications

### Session 2026-03-04

- Q: What order should sessions appear within a selected day? → A: Chronological order, earliest session first (oldest at top).
- Q: What should the month label show when a week spans two months? → A: Show both months (e.g. "Feb – Mar 2026").
- Q: How should the view behave during week navigation transitions? → A: Strip updates immediately; session cards area shows brief loading skeleton.

## Assumptions

- Sessions are grouped by the date portion of their `started_at` timestamp in the user's local browser timezone.
- The week starts on Monday (ISO 8601 convention), consistent with European/international standards.
- The "Show all sessions" link navigates to a separate route (e.g. `/sessions/all`) or toggles the view in-place. The exact mechanism will be decided during implementation.
- Mobile swipe detection uses basic touch event handling rather than a third-party gesture library, keeping dependencies minimal.
- The right arrow is enabled even when viewing the current week (allowing navigation to empty future weeks) for simplicity. This can be refined based on user feedback.
