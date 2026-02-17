# Feature Specification: Practice Analytics Dashboard

**Feature Branch**: `023-analytics-dashboard`
**Created**: 2026-02-17
**Status**: Draft
**Input**: User description: "Practice Analytics Dashboard - A read-only dashboard showing practice trends, streaks, weekly/monthly totals, most-practiced items, and confidence score trends over time. Leverages existing session data (durations, scores, timestamps) to give musicians motivation and insight into their practice habits. No new data persistence required - purely computed from existing sessions and library data via the API."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Practice Overview (Priority: P1)

A musician opens the analytics dashboard and immediately sees a summary of their practice activity: total practice time this week, number of sessions, and their current practice streak (consecutive days with at least one session). This gives them a quick motivational snapshot without needing to scroll or navigate further.

**Why this priority**: The overview is the entry point for all analytics. It delivers immediate value by answering "How much have I been practising?" — the most common question musicians have. Without this, there is no dashboard.

**Independent Test**: Can be fully tested by navigating to the dashboard route and verifying that summary statistics are visible. Delivers value as a standalone motivational snapshot even without the other stories.

**Acceptance Scenarios**:

1. **Given** a musician has completed 5 sessions over the past 7 days totalling 3 hours, **When** they open the dashboard, **Then** they see "5 sessions" and "3h 0m" for this week, plus their current streak length.
2. **Given** a musician has no completed sessions, **When** they open the dashboard, **Then** they see a friendly empty state encouraging them to start practising, with a link to begin a new session.
3. **Given** a musician practised yesterday and today, **When** they open the dashboard, **Then** their streak shows "2 days".
4. **Given** a musician last practised 3 days ago, **When** they open the dashboard, **Then** their streak shows "0 days" (streak broken).

---

### User Story 2 - View Practice History Chart (Priority: P2)

A musician wants to see how their practice time has trended over the past few weeks. The dashboard shows a line chart of daily practice minutes (connected points, x-axis = day, y-axis = minutes), making it easy to spot trends, patterns (e.g., weekday vs weekend practice), and gaps in practice habits.

**Why this priority**: Trend visualisation is the core differentiator of a dashboard over the existing session list. It transforms raw data into insight, helping musicians understand their habits at a glance.

**Independent Test**: Can be tested by creating sessions across multiple days and verifying the chart displays the correct daily totals. Delivers standalone value as a visual practice journal.

**Acceptance Scenarios**:

1. **Given** a musician has sessions spanning the past 4 weeks, **When** they view the practice history, **Then** they see daily practice minutes for the past 4 weeks (28 days).
2. **Given** a musician has no sessions on a particular day, **When** they view the chart, **Then** that day shows zero minutes.
3. **Given** a musician has multiple sessions on the same day, **When** they view the chart, **Then** that day shows the combined total of all sessions.

---

### User Story 3 - View Most Practised Items (Priority: P3)

A musician wants to know which pieces and exercises they've spent the most time on recently. The dashboard shows a ranked list of their most-practised library items, helping them ensure they're distributing practice time effectively.

**Why this priority**: Understanding practice distribution helps musicians avoid over-practising favourites while neglecting other repertoire. This builds on the overview by adding item-level detail.

**Independent Test**: Can be tested by creating sessions with entries for different library items and verifying the ranked list matches expected totals. Delivers standalone value as a "practice focus" report.

**Acceptance Scenarios**:

1. **Given** a musician has practised 5 different items this month, **When** they view the most-practised list, **Then** items are ranked by total practice time in descending order.
2. **Given** a musician has practised "Clair de Lune" for 120 minutes across 4 sessions, **When** they view the list, **Then** "Clair de Lune" shows "120 min" and "4 sessions".
3. **Given** a musician has no sessions, **When** they view the most-practised section, **Then** they see a message indicating no practice data is available yet.

---

### User Story 4 - View Confidence Score Trends (Priority: P4)

A musician wants to see how their confidence scores have changed over time for items they've been scoring. The dashboard shows score trends for their recently scored items, helping them see tangible progress (or identify items that need more attention).

**Why this priority**: Score trends close the feedback loop on the scoring feature (022). However, basic score history is already visible on individual item detail pages, making this an enhancement rather than a new capability.

**Independent Test**: Can be tested by creating sessions with scored entries across multiple dates and verifying score data is displayed chronologically. Delivers standalone value as a progress tracker.

**Acceptance Scenarios**:

1. **Given** a musician has scored "Hanon No. 1" as 2, 3, and 4 across three sessions, **When** they view score trends, **Then** they see the progression from 2 to 3 to 4 with session dates.
2. **Given** a musician has scored items but all scores are the same, **When** they view trends, **Then** the flat trend is displayed accurately.
3. **Given** a musician has never used confidence scoring, **When** they view the score trends section, **Then** they see a message explaining the feature and how to start scoring.

---

### Edge Cases

- What happens when a musician has only one session ever? The dashboard should still render meaningfully with a single data point rather than appearing broken.
- How does the dashboard handle sessions that were ended early? These count toward practice totals using their actual recorded duration, same as completed sessions.
- What happens when a musician has sessions spanning different time zones? All calculations use the session's stored timestamp without time zone conversion (consistent with existing session display).
- How does the streak calculation handle days with only very short sessions (e.g., 1 minute)? Any completed session counts toward the streak regardless of duration.
- What happens when a library item referenced in a session has been deleted? The dashboard uses the item title stored in the session entry, so deleted items still appear in analytics.
- What happens when the API request to fetch sessions fails or times out? The dashboard shows an inline error message with a retry button, allowing the user to re-attempt without navigating away.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display a dashboard accessible from the main navigation.
- **FR-002**: System MUST show total practice time and session count for the current week (Monday to Sunday).
- **FR-003**: System MUST calculate and display a practice streak as the number of consecutive days (up to and including today) with at least one completed session.
- **FR-004**: System MUST display daily practice minutes for the most recent 28 days as a line chart (connected data points, x-axis = day, y-axis = minutes).
- **FR-005**: System MUST show a ranked list of the top 10 most-practised items by total practice time, including session count and total minutes per item.
- **FR-006**: System MUST display confidence score history for items that have been scored, showing the most recently scored items first.
- **FR-007**: System MUST show appropriate empty states for each section when no relevant data exists.
- **FR-008**: System MUST compute all analytics from existing session data without requiring new data persistence.
- **FR-009**: System MUST handle sessions with "ended early" status identically to completed sessions for analytics purposes.
- **FR-010**: System MUST be read-only — no data creation, modification, or deletion from the dashboard.
- **FR-011**: The dashboard MUST link to the "new session" flow from empty states to encourage first-time users.
- **FR-012**: System MUST display a skeleton/placeholder UI while data is loading.
- **FR-013**: System MUST display an inline error message with a retry button if data fetching fails.

### Key Entities

- **Weekly Summary**: Aggregated view of the current week's practice — total minutes, session count, and comparison context.
- **Practice Streak**: Consecutive-day count derived from session timestamps, representing sustained practice commitment.
- **Daily Practice Total**: Per-day aggregation of practice minutes across all sessions, used for trend display.
- **Item Practice Ranking**: Per-item aggregation of total practice time and session count, ranked by time spent.
- **Score Trend**: Chronological sequence of confidence scores for a single library item across sessions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can view their complete practice overview (weekly stats, streak, trends) within 2 seconds of navigating to the dashboard.
- **SC-002**: The dashboard correctly aggregates data from 100+ sessions without visible delay or incorrect totals.
- **SC-003**: Users with no practice history see clear, friendly guidance rather than empty or broken-looking screens.
- **SC-004**: All analytics sections display accurately for edge cases: single session, same-day multiple sessions, ended-early sessions, and deleted library items.
- **SC-005**: The dashboard is fully usable on mobile devices (responsive layout consistent with existing app design).

## Clarifications

### Session 2026-02-17

- Q: What type of visualisation for the 28-day practice history? → A: Line chart (connected points, x=day, y=minutes) — best for showing trends, future-proof for longer windows.
- Q: How should the dashboard handle API fetch failures or slow loading? → A: Skeleton/placeholder UI while loading; on failure show inline error message with retry button.

## Assumptions

- The "current week" follows ISO 8601 convention (Monday through Sunday).
- Practice streak is calculated from today backwards — if today has no session yet, the streak counts from yesterday.
- The 28-day practice history window is fixed (not user-configurable) to keep the initial implementation simple.
- The "top 10 most-practised items" ranking covers all time, not just the current week/month.
- Score trends display the 5 most recently scored items to avoid overwhelming the view.
- All analytics are computed on each dashboard load from existing API data (no pre-aggregation or caching layer).
