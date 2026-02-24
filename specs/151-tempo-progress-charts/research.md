# Research: Tempo Progress Charts

**Feature Branch**: `151-tempo-progress-charts`
**Date**: 2026-02-24

## R1: Chart Component Strategy — New vs Extend Existing

**Decision**: Create a new `TempoProgressChart` component rather than extending the existing `LineChart`.

**Rationale**: The existing `LineChart` is tightly coupled to `DailyPracticeTotal` (fixed 28-day window, minutes Y-axis, day-only X-axis labels). The tempo chart requires different data shapes (`TempoHistoryEntry`), a variable number of data points, BPM Y-axis, session-date X-axis, and a target reference line. Extending `LineChart` would require genericising it significantly, risking regressions on the analytics page. A new component reuses the same SVG patterns (viewBox, polyline, circles, axis labels) but is independently maintainable.

**Alternatives considered**:
- Generic `LineChart<T>` with trait-based data adapters — over-engineered for two use cases; Leptos component generics add complexity.
- External charting library (e.g., Plotly via WASM bindings) — adds significant bundle size, not justified for one chart.

## R2: Data Availability

**Decision**: No new API endpoints or database queries needed. All data is available client-side.

**Rationale**: `ItemPracticeSummary` already contains `tempo_history: Vec<TempoHistoryEntry>` and `latest_tempo: Option<u16>`, precomputed by `build_practice_summaries()` in a single O(M×E) pass. The target BPM comes from the item's `tempo` field already available on the detail page. The chart component receives data as props — no new effects, no new HTTP calls.

**Alternatives considered**:
- Server-side chart data endpoint — unnecessary when all data is already on the client.

## R3: Y-Axis Scaling Strategy

**Decision**: Auto-scale to data range with padding, including the target BPM in range calculation. Round to nearest 10 BPM for clean axis labels.

**Rationale**: Starting from 0 would compress tempo differences (e.g., 100-120 BPM would appear flat on a 0-500 scale). Auto-scaling to the data range makes small improvements visible, which is the motivational purpose of the chart. Including the target BPM ensures the reference line is always visible within the chart area.

**Alternatives considered**:
- Always start Y-axis at 0 — hides small tempo gains that musicians care about.
- Fixed 0-500 range — wastes most of the chart area; most data sits in a narrow band.

## R4: Target Reference Line Styling

**Decision**: Use `--color-chart-secondary` (warm gold) for the target line with dashed stroke to visually distinguish from the data line (`--color-chart-line`, warm indigo).

**Rationale**: The design system already provides `--color-chart-secondary` as a second series colour. Using dashed stroke plus a different colour provides dual differentiation (pattern + hue) for accessibility. A small text label at the right end of the line identifies it as "Target".

**Alternatives considered**:
- Same colour as data line, dashed only — insufficient contrast for colour-blind users.
- Red/green target line — carries achievement/failure connotation that conflicts with the positive framing in the constitution's inclusive design principle.

## R5: Progress Percentage Computation

**Decision**: Compute as `(latest_tempo / target_bpm * 100)` rounded to nearest integer. Display as "X% of target" text above the chart. Allow values above 100%.

**Rationale**: Simple ratio is immediately understandable. Not capping at 100% celebrates exceeding the target rather than making it invisible. Computed in the view from existing data — no new model fields needed.

**Alternatives considered**:
- Progress bar/ring instead of text — adds visual complexity to an already chart-heavy section.
- Only show when < 100% — hides the rewarding moment of exceeding the target.

## R6: Large Dataset Handling

**Decision**: Render all data points. At high density (50+ points), reduce circle radius to prevent overlap. No pagination or windowing.

**Rationale**: SVG handles hundreds of elements efficiently. The polyline itself remains legible at any density — it's only the circles that risk overlap. Reducing circle radius from 3 to 2 at 50+ points and hiding circles entirely at 100+ points (keeping only the polyline with tooltips on invisible hit targets) keeps the chart clean. Pagination would break the "progress over time" narrative.

**Alternatives considered**:
- Show only last N sessions — loses the long-term progress story.
- Aggregate to weekly averages — masks individual session detail.
