# Feature Specification: Weekly Practice Summary

**Feature Branch**: `153-weekly-practice-summary`
**Created**: 2026-02-24
**Status**: Draft
**Input**: User description: "Issue #68: Weekly practice summary with insights"

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Week-over-week comparison (Priority: P1)

A musician opens the analytics page and sees a summary comparing this week's
practice to last week. At a glance they can tell whether they practised more or
fewer sessions, spent more or less total time, and covered more or fewer distinct
items. The comparison uses neutral language — "up from 3" / "down from 5" —
without shaming or guilt.

**Why this priority**: The core value proposition. Without the comparison, this
is just a repeat of the existing stat cards. Comparison turns raw numbers into
insight: "Am I practising more or less than last week?"

**Independent Test**: Seed two weeks of sessions. Navigate to analytics. Verify
the summary section shows this-week and last-week values with directional
indicators (up/down/same).

**Acceptance Scenarios**:

1. **Given** the user has sessions in both the current and previous ISO week,
   **When** they visit the analytics page,
   **Then** they see this week's totals alongside last week's totals with
   directional arrows or text indicating the change.

2. **Given** the user has sessions this week but none last week,
   **When** they visit the analytics page,
   **Then** the summary shows this week's totals with "no data last week" rather
   than comparing to zero.

3. **Given** the user has sessions last week but none this week (e.g. it's Monday
   morning),
   **When** they visit the analytics page,
   **Then** the summary shows "0 sessions so far this week" compared to last
   week's totals.

---

### User Story 2 — Items covered & neglected (Priority: P2)

A musician sees which library items they practised this week and — more
importantly — which items they haven't touched recently. This helps them plan
their next session by surfacing items that might be falling behind.

**Why this priority**: Actionable insight. Knowing *what* was practised (and
what wasn't) is more valuable than knowing *how much* was practised. Connects
the Track pillar back to Plan.

**Independent Test**: Seed a library with 10 items and sessions covering only 4
of them this week. Verify the summary lists items covered and flags neglected
items.

**Acceptance Scenarios**:

1. **Given** the user has 10 items in their library and practised 4 of them this
   week,
   **When** they view the weekly summary,
   **Then** they see a count of items covered (4) and a list of up to 5 items
   not practised in the last 14 days, ordered by days since last practice
   (longest gap first).

2. **Given** all library items were practised this week,
   **When** they view the weekly summary,
   **Then** the neglected items section is hidden (not shown as an empty list).

3. **Given** the user has no items in their library,
   **When** they view the weekly summary,
   **Then** the items-covered section is hidden.

---

### User Story 3 — Score improvements (Priority: P3)

A musician sees which items had score changes this week — specifically which
improved and by how much. This reinforces the feeling that practice is working.

**Why this priority**: Motivational but not essential. Improvements provide
emotional reward. However, many users won't score every session, so this section
may often be empty.

**Independent Test**: Seed sessions where one item went from score 2 to score 4
this week. Verify the summary highlights the +2 improvement.

**Acceptance Scenarios**:

1. **Given** the user scored item "Clair de Lune" at 2 last week and 4 this week,
   **When** they view the weekly summary,
   **Then** they see "Clair de Lune: 2 → 4 (+2)" in the improvements section.

2. **Given** no items were scored this week,
   **When** they view the weekly summary,
   **Then** the score improvements section is hidden.

3. **Given** an item's score went down this week,
   **When** they view the weekly summary,
   **Then** it is still shown, with neutral framing (e.g. "3 → 2") — no
   negative language.

---

### Edge Cases

- **New user with zero sessions**: The weekly summary section is not rendered.
  The existing empty state handles this case.
- **Mid-week (e.g. Monday)**: The summary still renders with partial data.
  "So far this week" language makes this clear.
- **Very long session (e.g. 8 hours)**: Time formatting handles hours correctly
  (existing format utility already does this).
- **User practises the same item 50 times in a week**: The items-covered count
  is distinct items, not total entries.
- **Deleted items**: If a library item was practised then deleted, it still
  appears in session data with its title. It should appear in the summary but
  not in the neglected-items list (which only considers current library items).
- **Week boundary on Sunday night**: A session at 23:55 on Sunday belongs to
  that week; a session at 00:05 on Monday belongs to the next week. Uses ISO
  week numbering consistently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST compute a weekly summary from existing session data,
  comparing the current ISO week (Monday–Sunday) to the previous ISO week.
- **FR-002**: The weekly summary MUST include: total practice time, session
  count, and distinct items practised — for both current and previous weeks.
- **FR-003**: The summary MUST show directional comparison indicators (up, down,
  or unchanged) for each metric.
- **FR-004**: System MUST identify neglected items — library items not practised
  in the last 14 days, including items never practised — and display up to 5.
  Never-practised items sort to the top with a "never practised" label; remaining
  items are ordered by days since last practice (longest gap first).
- **FR-005**: System MUST identify score changes this week — items whose
  latest score this week differs from their latest score before this week —
  and display up to 5, ordered by largest absolute delta first.
- **FR-006**: The weekly summary MUST replace the existing "This Week" and
  "Sessions" stat cards on the analytics page. Only the streak stat card is
  retained. The summary is positioned between the streak card and the practice
  history chart.
- **FR-007**: Sections with no data (no neglected items, no score changes) MUST
  be hidden rather than shown as empty.
- **FR-008**: All computation MUST be pure (no I/O, no system clock) and accept
  a `today` parameter, consistent with the existing analytics module pattern.
- **FR-009**: Score changes MUST use neutral language. No words like "worse",
  "declined", or "dropped".
- **FR-010**: The entire weekly summary section MUST be hidden when the user
  has no session data, preserving the existing empty state.

### Key Entities

- **WeekComparison**: Holds this-week and last-week values for a single metric
  (e.g. total minutes, session count, items covered), plus a direction indicator
  (up, down, same).
- **NeglectedItem**: An item from the current library that hasn't been practised
  within a 14-day lookback window, or has never been practised at all. Includes
  the number of days since last practice (or a "never practised" indicator).
- **ScoreChange**: An item whose score changed this week, showing the previous
  score, current score, and signed delta.

## Design *(include if feature has UI)*

### Existing Components Used

- `Card` — container for the weekly summary section
- `PageHeading` — already on the analytics page
- Design tokens: `text-primary`, `text-secondary`, `text-muted`, `text-faint`,
  `text-accent-text`, `text-success-text`, `bg-surface-secondary`,
  `border-border-default`

### New Components Needed

- **WeekComparisonRow**: Displays three metrics in a grid — each showing a
  this-week value, the comparison to last week (e.g. "↑ from 3"), and a label.
  Reusable for time, sessions, and items-covered.
- **NeglectedItemsList**: A compact list of up to 5 library items not recently
  practised, each showing item title and "X days ago" timestamp.
- **ScoreChangesList**: A compact list of items with score changes, showing
  the before → after values and delta.

### Wireframe / Layout Description

```
┌─────────────────────────────────────────────┐
│  Analytics (PageHeading)                     │
├─────────────────────────────────────────────┤
│  [Streak StatCard]                           │  ← only streak remains
├─────────────────────────────────────────────┤
│  ┌─ This Week (Card) ─────────────────────┐ │
│  │                                         │ │
│  │  ┌──────────┐ ┌──────────┐ ┌─────────┐ │ │
│  │  │ 2h 30m   │ │ 5        │ │ 6       │ │ │
│  │  │ ↑ 1h 15m │ │ ↑ from 3 │ │ ↓ from 8│ │ │
│  │  │ time     │ │ sessions │ │ items   │ │ │
│  │  └──────────┘ └──────────┘ └─────────┘ │ │
│  │                                         │ │
│  │  Needs attention          Improvements  │ │
│  │  • Scales (12 days ago)   Clair: 2 → 4 │ │
│  │  • Hanon No. 3 (9 days)   Etude: 3 → 4 │ │
│  │  • Arpeggios (8 days)                   │ │
│  │                                         │ │
│  └─────────────────────────────────────────┘ │
├─────────────────────────────────────────────┤
│  Practice History (28 days)                  │  ← existing
```

### Responsive Behaviour

- **Mobile (<640px)**: The three comparison metrics stack in a 3-column grid
  (same as existing stat cards). Neglected items and score changes stack
  vertically below.
- **Desktop (≥640px)**: Comparison metrics sit in a 3-column grid. Neglected
  items and score changes sit side-by-side in a 2-column grid below.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users with session data see the weekly summary section on the
  analytics page within 1 second of page load.
- **SC-002**: All three comparison metrics (time, sessions, items) display
  accurate current-week and previous-week values with correct directional
  indicators.
- **SC-003**: Neglected items list surfaces items not practised in 14+ days,
  correctly ordered by gap length, limited to 5 items.
- **SC-004**: Score changes accurately reflect this-week vs pre-this-week
  score deltas for all scored items.
- **SC-005**: The summary section is hidden entirely when the user has no
  session data, preserving the existing empty state experience.
- **SC-006**: All computation is deterministic — given the same session data
  and date, the same summary is produced every time.

## Clarifications

### Session 2026-02-24

- Q: How should the new weekly summary relate to the existing stat cards (which already show this-week time and sessions)? → A: Replace — remove the time and sessions stat cards, keep only the streak card. The new weekly summary card handles time, sessions, and items with comparisons.
- Q: Should never-practised library items appear in the neglected items list? → A: Yes — include them at the top of the list with a "never practised" label.
- Q: Should score changes have a display limit? → A: Cap at 5, showing the largest absolute deltas first.

## Assumptions

- The "week" boundary follows ISO 8601 (Monday–Sunday), consistent with the
  existing weekly summary computation.
- The neglected-items lookback window is 14 days. This is a reasonable default
  that catches items falling out of regular rotation without being so short that
  everything appears neglected.
- Score changes compare the latest score this week vs the latest score in all
  prior sessions. Items scored for the first time this week are treated as new
  (shown with "new" indicator rather than a delta from 0).
- No new API endpoints or database tables are needed. All data is computed from
  existing session and item data already available in the application model.
- No email, push notification, or "shown on next open" delivery mechanism for
  v1. The summary lives on the analytics page.
