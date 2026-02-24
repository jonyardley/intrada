# Feature Specification: Tempo Progress Charts

**Feature Branch**: `151-tempo-progress-charts`
**Created**: 2026-02-24
**Status**: Draft
**Input**: GitHub issue #66 — "Tempo progress charts — line charts showing tempo increase over time toward target BPM for each item"

## User Scenarios & Testing *(mandatory)*

### User Story 1 — View Tempo Progress for a Single Item (Priority: P1)

A musician navigates to an item's detail page and sees a line chart showing how their achieved tempo has changed across practice sessions. A horizontal reference line marks the target BPM (if set), making it immediately clear how close they are. This replaces the current plain date/BPM list with a visual that tells the story of progress at a glance.

**Why this priority**: This is the core value — seeing tempo improvement over time is what the feature exists for. Without this, nothing else matters.

**Independent Test**: Can be fully tested by creating an item with a target tempo, logging several sessions with different achieved tempos, then viewing the item detail page and confirming the chart renders with correct data points and target line.

**Acceptance Scenarios**:

1. **Given** an item with 3+ tempo history entries and a target BPM, **When** the musician views the item detail page, **Then** they see a line chart with data points for each recorded tempo and a dashed horizontal line at the target BPM.
2. **Given** an item with tempo history but no target BPM, **When** the musician views the item detail, **Then** they see the line chart without a target reference line.
3. **Given** an item with no tempo history, **When** the musician views the item detail, **Then** they see a message like "No tempo data recorded yet" instead of an empty chart.
4. **Given** an item with exactly 1 tempo data point, **When** the musician views the item detail, **Then** they see a single data point plotted on the chart (not a line, since there's nothing to connect to).

---

### User Story 2 — See Progress Percentage Toward Target (Priority: P2)

When viewing an item that has both a target BPM and at least one achieved tempo, the musician sees a progress indicator showing how close their most recent tempo is to the target. This gives a quick at-a-glance answer to "how far am I?"

**Why this priority**: Adds motivational context to the chart. Simple to compute (latest tempo ÷ target × 100), high emotional value.

**Independent Test**: Can be tested by checking the progress display updates correctly when new tempo data is logged, and handles edge cases (achieved > target, no target set).

**Acceptance Scenarios**:

1. **Given** an item with target BPM of 120 and latest achieved tempo of 90, **When** the musician views the detail page, **Then** they see "75% of target" (or similar wording).
2. **Given** an item with target BPM of 120 and latest achieved tempo of 130, **When** the musician views the detail page, **Then** they see "108% of target" — achievement beyond the target is celebrated, not capped.
3. **Given** an item with no target BPM, **When** the musician views the detail page, **Then** no progress percentage is shown (nothing to measure against).
4. **Given** an item with a target but no achieved tempo, **When** the musician views the detail page, **Then** no progress percentage is shown (no data to compute from).

---

### User Story 3 — Interact with Chart Data Points (Priority: P3)

When hovering over (desktop) or tapping (mobile) a data point on the tempo chart, the musician sees a tooltip with the exact BPM value and the session date. This lets them inspect specific sessions without leaving the page.

**Why this priority**: Polish that improves usability but the chart is fully functional without it. Follows the pattern already established by the existing practice history chart.

**Independent Test**: Can be tested by hovering/tapping data points and verifying the tooltip shows the correct date and BPM value.

**Acceptance Scenarios**:

1. **Given** a tempo chart with multiple data points, **When** the musician hovers over a data point (desktop), **Then** a tooltip shows the session date and exact BPM.
2. **Given** a tempo chart on a touch device, **When** the musician taps a data point, **Then** the same date/BPM information is accessible.

---

### Edge Cases

- What happens when multiple sessions on the same day record different tempos for the same item? Each data point is plotted individually — the chart shows all recordings, not daily averages.
- What happens when tempo values vary wildly (e.g., 40 BPM then 200 BPM)? The Y-axis scales dynamically to fit the data range, with the target line included in the range calculation if present.
- What happens when the target BPM is lower than all achieved tempos? The target line still displays; the chart simply shows the musician has exceeded their goal.
- What happens if the target BPM changes after tempo data is recorded? The chart always shows the current target. Historical target changes are not tracked.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display a line chart of achieved tempo over time on the item detail page when tempo history data exists.
- **FR-002**: System MUST show a horizontal reference line at the target BPM when the item has a target tempo set.
- **FR-003**: System MUST scale the Y-axis to accommodate all data points and the target reference line (if present), with appropriate padding above and below.
- **FR-004**: System MUST label the X-axis with session dates and the Y-axis with BPM values.
- **FR-005**: System MUST show each tempo recording as a distinct data point on the chart, connected by lines in chronological order.
- **FR-006**: System MUST display a progress percentage (latest tempo ÷ target × 100) when both a target BPM and at least one achieved tempo exist.
- **FR-007**: System MUST show an empty state message when no tempo data exists for the item.
- **FR-008**: System MUST provide date and BPM details for each data point via tooltip (hover on desktop, accessible via tap on mobile).
- **FR-009**: The chart MUST render correctly with any number of data points from 1 to hundreds.
- **FR-010**: The tempo chart MUST visually distinguish the target reference line from the data line (e.g., dashed vs solid, different colour).

### Key Entities

- **TempoHistoryEntry**: A single tempo measurement — session date, achieved BPM, and session reference. Already exists in the data model from tempo tracking (#52).
- **ItemPracticeSummary**: Contains the `tempo_history` collection and `latest_tempo` for an item. Already exists.
- **Target BPM**: The item's tempo target, stored as part of the item's tempo field. Already exists.

## Design

### Existing Components Used

- **Card** — container for the tempo chart section on the item detail page
- **LineChart** — existing SVG line chart component (currently used for 28-day practice history). Provides the rendering pattern: SVG viewBox, polyline, data points with tooltips, grid lines, axis labels.

### New Components Needed

- **TempoProgressChart**: A line chart specialised for tempo data. Displays achieved BPM over time with an optional target reference line. Follows the same SVG approach as the existing LineChart but adapted for tempo-specific data (BPM Y-axis, session dates X-axis, target reference line).
- **TempoProgressSummary**: A small display showing the progress percentage toward target (e.g., "75% of target"). Placed above or alongside the chart.

### Wireframe / Layout Description

The tempo progress section replaces the current plain tempo history list on the item detail page. Layout within the practice summary Card:

```
┌─────────────────────────────────────┐
│ Practice Summary                    │
│                                     │
│  Sessions: 12    Time: 3h 45m       │
│                                     │
│  ── Score History ──────────────    │
│  [existing score history section]   │
│                                     │
│  ── Tempo Progress ─────────────    │
│  ♩ 75% of target (120 BPM)         │
│                                     │
│  BPM                                │
│  140 ┤                              │
│  120 ┤- - - - - - - - - target - -  │
│  100 ┤         ●───●                │
│   80 ┤     ●──●                     │
│   60 ┤ ●──●                         │
│      └──┬──┬──┬──┬──┬──┬──→ Date    │
│        Jan  Feb  Mar                │
│                                     │
└─────────────────────────────────────┘
```

### Responsive Behaviour

- **Mobile**: Chart fills full card width. Axis labels may be abbreviated (e.g., "Jan" instead of "2026-01-15"). Tooltips appear as overlays. Fewer X-axis date labels are shown to avoid overlap.
- **Desktop**: Chart fills card width with comfortable padding. Full date labels where space allows. Tooltips appear on hover near the data point.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Musicians can see their tempo progress for any item that has tempo data within 1 second of navigating to the item detail page.
- **SC-002**: The chart correctly plots all recorded tempo data points in chronological order with the target reference line at the correct BPM.
- **SC-003**: The progress percentage accurately reflects the ratio of latest achieved tempo to target BPM.
- **SC-004**: The chart is legible and usable on both mobile (≥320px width) and desktop screens.
- **SC-005**: All existing tests continue to pass — the chart is additive and does not break existing functionality.

## Assumptions

- Tempo history data is already available via `ItemPracticeSummary.tempo_history` from the tempo tracking feature (#52). No new data collection is needed.
- The existing SVG charting approach (used in the 28-day practice chart) is sufficient for this feature. No external charting library is required.
- The chart displays raw data points per session, not aggregated daily/weekly averages.
- Target BPM is the item's current target — historical target changes are not tracked or displayed.
- The chart appears on the item detail page only. A cross-item tempo dashboard is a separate future feature.
