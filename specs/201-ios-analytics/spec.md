# Feature Specification: iOS Analytics Dashboard

**Feature Branch**: `201-ios-analytics`
**Created**: 2026-04-04
**Status**: Draft
**Input**: iOS analytics dashboard — practice insights and visualisation. Weekly summary with week-over-week comparison, practice streak, 28-day chart, most practised items, score trends, neglected items, score changes. iPhone and iPad layouts. Replaces Analytics tab placeholder.

## User Scenarios & Testing

### User Story 1 — Weekly Practice Overview (Priority: P1)

A musician opens the Analytics tab to see how their week is going. They see their practice streak, this week's stats (time, sessions, items) compared to last week, and any items that need attention.

**Why this priority**: The weekly overview is the entry point and primary value — it answers "how am I doing?" at a glance.

**Independent Test**: Practice for a few days → open Analytics → see streak, weekly stats with comparison arrows, neglected items.

**Acceptance Scenarios**:

1. **Given** the user has practiced this week, **When** they open Analytics, **Then** they see: practice streak (days), total practice time, session count, and items covered — each with a comparison arrow (up/down/same) versus last week.
2. **Given** items haven't been practiced in 14+ days, **When** viewing Analytics, **Then** a "Needs Attention" section shows those items.
3. **Given** scores have changed this week, **When** viewing Analytics, **Then** an "Improvements" section shows score transitions.
4. **Given** no practice data exists, **When** viewing Analytics, **Then** empty states explain what each section will show.

---

### User Story 2 — Practice History Chart (Priority: P2)

A musician sees a 28-day chart showing daily practice minutes, giving a visual sense of consistency.

**Why this priority**: The chart provides visual proof of effort — important for motivation and pattern recognition.

**Independent Test**: Practice over several days → see chart with bars/dots for each day.

**Acceptance Scenarios**:

1. **Given** practice data exists for the past 28 days, **When** viewing Analytics, **Then** a chart shows daily practice minutes.
2. **Given** no data for the past 28 days, **When** viewing the chart area, **Then** an empty state explains.

---

### User Story 3 — Most Practised & Score Trends (Priority: P3)

A musician sees which items they practice most and how their scores trend over time.

**Why this priority**: Rankings and trends provide insight into practice distribution and progress.

**Independent Test**: Practice several items with scores → see ranked list and score dots per item.

**Acceptance Scenarios**:

1. **Given** items have been practiced, **When** viewing Analytics, **Then** a ranked list shows items by total practice time with session counts.
2. **Given** items have scores, **When** viewing Analytics, **Then** score trends show progression with colour-coded dots.

---

## Requirements

### Functional Requirements

- **FR-001**: The Analytics tab MUST display the current practice streak (consecutive days).
- **FR-002**: Weekly summary MUST show: total minutes, session count, items covered — each with direction indicator versus previous week.
- **FR-003**: A 28-day practice history chart MUST be displayed.
- **FR-004**: Most practised items MUST be shown ranked by total practice time.
- **FR-005**: Score trends MUST show per-item score progression with colour-coded indicators.
- **FR-006**: Neglected items (14+ days since practice) MUST be highlighted.
- **FR-007**: Score changes this week MUST be shown with delta indicators.
- **FR-008**: Empty states MUST be shown when no data exists for each section.
- **FR-009**: The dashboard MUST support both iPhone and iPad layouts.

## Design

### Existing Components Used

- `CardView` — section containers
- `TypeBadge` — item type indicators
- `EmptyStateView` — no data states
- `StatCard` — metric display

### New Components Needed

- **AnalyticsDashboardView**: Tab root — scrollable dashboard with all sections
- **WeeklySummaryCard**: 3-stat comparison with direction arrows
- **PracticeChartView**: 28-day bar chart using SwiftUI Charts
- **ScoreTrendDots**: Colour-coded score history dots per item

## Success Criteria

- **SC-001**: Dashboard loads instantly from cached ViewModel data.
- **SC-002**: All sections display correctly with 50+ sessions of data.
- **SC-003**: Empty states are clear and encouraging, not discouraging.

## Assumptions

- `AnalyticsView` is already computed by the Crux core and available in `core.viewModel.analytics`.
- No events to dispatch — this is purely a read-only data display.
- SwiftUI Charts framework available on iOS 17+.
- The web's line chart translates to a bar chart on iOS (more native feel).
