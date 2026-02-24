# Tasks: Tempo Progress Charts

**Input**: Design documents from `/specs/151-tempo-progress-charts/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

**Tests**: No test tasks — spec does not request TDD or explicit test generation. Existing `cargo test` suite covers regressions.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Workspace layout: `crates/intrada-web/` (web shell), `crates/intrada-core/` (Crux core — not modified).

---

## Phase 1: Setup

**Purpose**: Register the new component module and ensure design tokens are available

- [x] T001 Add `pub mod tempo_progress_chart;` and re-export `TempoProgressChart` in `crates/intrada-web/src/components/mod.rs`
- [x] T002 [P] Create empty `crates/intrada-web/src/components/tempo_progress_chart.rs` with module-level doc comment and placeholder `TempoProgressChart` component that returns an empty view

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Data preparation helpers that ALL user stories depend on — Y-axis scaling, date formatting, target BPM parsing

**⚠️ CRITICAL**: No user story rendering work can begin until these helpers exist

- [x] T003 Implement `parse_target_bpm(tempo: &Option<String>) -> Option<u16>` helper in `crates/intrada-web/src/components/tempo_progress_chart.rs` — extracts numeric BPM from strings like "120 BPM" or "120"
- [x] T004 Implement `compute_y_range(entries: &[TempoHistoryEntry], target: Option<u16>) -> (u16, u16)` helper in `crates/intrada-web/src/components/tempo_progress_chart.rs` — returns (min, max) rounded to nearest 10 BPM with padding, includes target in range calculation per R3
- [x] T005 [P] Implement `format_chart_date(rfc3339: &str) -> String` helper in `crates/intrada-web/src/components/tempo_progress_chart.rs` — formats session date for X-axis labels (abbreviated month + day)

**Checkpoint**: Data preparation helpers ready — chart rendering can now begin

---

## Phase 3: User Story 1 — Line Chart with Target Reference Line (Priority: P1) 🎯 MVP

**Goal**: Display a line chart of achieved tempo over time on the item detail page, with an optional dashed target reference line at the target BPM

**Independent Test**: Navigate to an item with 3+ tempo recordings and a target BPM set. Verify: (1) line chart appears with data points connected by lines in chronological order, (2) dashed target reference line at the correct BPM, (3) Y-axis shows BPM labels, (4) X-axis shows session dates. Also verify items with no tempo history show no chart.

### Implementation for User Story 1

- [x] T006 [US1] Implement SVG chart scaffold in `TempoProgressChart` component in `crates/intrada-web/src/components/tempo_progress_chart.rs` — viewBox (0 0 600 200), chart padding constants, grid lines using `var(--color-chart-grid)`, Y-axis BPM labels (min/mid/max) using auto-scaled range from T004
- [x] T007 [US1] Implement data point plotting in `crates/intrada-web/src/components/tempo_progress_chart.rs` — reverse tempo_history to chronological order, map entries to SVG coordinates, render `<polyline>` for connecting line using `var(--color-chart-line)` and `<circle>` elements for data points using `var(--color-chart-point-stroke)`, with density-adaptive circle radius per R6 (radius 3 for <50 points, 2 for 50+, hidden for 100+)
- [x] T008 [US1] Implement X-axis date labels in `crates/intrada-web/src/components/tempo_progress_chart.rs` — render date labels below the chart area at evenly spaced intervals, using `format_chart_date` from T005, showing a subset of dates to avoid overlap
- [x] T009 [US1] Implement target reference line in `crates/intrada-web/src/components/tempo_progress_chart.rs` — when target BPM is Some, render a dashed horizontal `<line>` at the target Y-position using `var(--color-chart-secondary)` with `stroke-dasharray`, and a small "Target" text label at the right end per R4
- [x] T010 [US1] Implement empty state handling in `crates/intrada-web/src/components/tempo_progress_chart.rs` — when tempo_history is empty, render nothing (hidden); when exactly 1 data point, render a single `<circle>` without a polyline
- [x] T011 [US1] Add `aria-label` to the SVG element in `crates/intrada-web/src/components/tempo_progress_chart.rs` describing the chart content (e.g., "Tempo progress chart showing X data points") for accessibility
- [x] T012 [US1] Integrate `TempoProgressChart` into the item detail view in `crates/intrada-web/src/views/detail.rs` — replace the current tempo history plain list (date/BPM badges) with the new chart component, passing `tempo_history: Vec<TempoHistoryEntry>` and `target_bpm: Option<u16>` (parsed from item.tempo) as props

**Checkpoint**: At this point, the tempo line chart renders on the item detail page with data points, connecting lines, target reference line, proper Y-axis scaling, and empty state handling. This is a shippable MVP.

---

## Phase 4: User Story 2 — Progress Percentage (Priority: P2)

**Goal**: Display a progress percentage showing how close the latest achieved tempo is to the target BPM

**Independent Test**: View an item with target BPM of 120 and latest achieved tempo of 90 — verify "75% of target" is displayed. View an item exceeding the target (130/120) — verify "108% of target". View an item with no target — verify no percentage shown.

### Implementation for User Story 2

- [x] T013 [US2] Implement progress percentage display in `crates/intrada-web/src/components/tempo_progress_chart.rs` — compute `(latest_tempo / target_bpm * 100)` rounded to nearest integer per R5, render as text above the chart (e.g., "♩ 75% of target (120 BPM)"), allow values above 100%, show nothing when target or latest_tempo is None
- [x] T014 [US2] Update `TempoProgressChart` component props in `crates/intrada-web/src/components/tempo_progress_chart.rs` to accept `latest_tempo: Option<u16>` and pass it from `crates/intrada-web/src/views/detail.rs` using `ItemPracticeSummary.latest_tempo`

**Checkpoint**: At this point, Users see both the chart AND the progress percentage. Stories 1 and 2 are both functional.

---

## Phase 5: User Story 3 — Tooltips on Data Points (Priority: P3)

**Goal**: Show date and BPM details when hovering over (desktop) or tapping (mobile) chart data points

**Independent Test**: On desktop, hover over a data point — verify tooltip shows session date and exact BPM. On mobile emulation, tap a point — verify same info is accessible.

### Implementation for User Story 3

- [x] T015 [US3] Add `<title>` tooltip elements inside each `<circle>` (or invisible hit-target `<circle>` for 100+ points) in `crates/intrada-web/src/components/tempo_progress_chart.rs` — tooltip text shows formatted date and BPM value (e.g., "Jan 15, 2026 — 95 BPM"), following the same pattern as the existing LineChart tooltips

**Checkpoint**: All user stories complete. Chart, progress percentage, and tooltips all functional.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Design catalogue entry, validation, and final checks

- [x] T016 [P] Add TempoProgressChart showcase entry to `crates/intrada-web/src/views/design_catalogue.rs` — show the chart with sample tempo data (5-10 data points with an upward trend) and a target reference line, demonstrating the full component visually
- [x] T017 Run quickstart.md verification steps (V1–V8) to validate all acceptance scenarios
- [x] T018 Run `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` to verify no regressions

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on T002 (file exists) — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Phase 2 completion (data helpers)
- **User Story 2 (Phase 4)**: Depends on Phase 3 (chart must exist to add percentage above it)
- **User Story 3 (Phase 5)**: Depends on Phase 3 (data point circles must exist to add tooltips)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 — no dependencies on other stories
- **User Story 2 (P2)**: Depends on US1 (progress text is rendered above the chart component)
- **User Story 3 (P3)**: Depends on US1 (tooltips attach to data point circles rendered in US1)
- **US2 and US3**: Can run in parallel once US1 is complete (they modify different parts of the same component)

### Within Each User Story

- Data preparation helpers (Phase 2) before rendering code
- SVG scaffold (T006) before data plotting (T007) before target line (T009)
- Core chart before detail.rs integration (T012)
- Story complete before moving to next priority

### Parallel Opportunities

- T001 and T002 can run in parallel (different files)
- T004 and T005 can run in parallel (independent helpers in same file, but no conflicts)
- T013 and T015 can run in parallel after US1 (different parts of the component)
- T016 can run in parallel with T017/T018 (different files)

---

## Parallel Example: User Story 1

```bash
# After Phase 2, launch US1 tasks sequentially (same file):
Task: T006 "SVG chart scaffold in tempo_progress_chart.rs"
Task: T007 "Data point plotting in tempo_progress_chart.rs"
Task: T008 "X-axis date labels in tempo_progress_chart.rs"
Task: T009 "Target reference line in tempo_progress_chart.rs"
Task: T010 "Empty state handling in tempo_progress_chart.rs"
Task: T011 "Aria-label accessibility in tempo_progress_chart.rs"
Task: T012 "Integrate chart into detail.rs"
```

## Parallel Example: After US1 Complete

```bash
# US2 and US3 can run in parallel (different parts of the component):
Task: T013+T014 "Progress percentage (US2)"
Task: T015 "Tooltips on data points (US3)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T005)
3. Complete Phase 3: User Story 1 (T006–T012)
4. **STOP and VALIDATE**: Run V1, V2, V4, V5, V7, V8 from quickstart.md
5. This delivers a fully functional tempo chart — shippable

### Incremental Delivery

1. Complete Setup + Foundational → Helpers ready
2. Add User Story 1 → Test with quickstart V1/V2/V4/V5/V7 → Chart works (MVP!)
3. Add User Story 2 → Test with quickstart V3 → Progress percentage works
4. Add User Story 3 → Test with quickstart V6 → Tooltips work
5. Polish → Design catalogue + full validation → Complete

---

## Notes

- All implementation is in `crates/intrada-web/` only — no core or API changes
- The chart reads from `ItemPracticeSummary.tempo_history` and `latest_tempo` already available via `build_practice_summaries()` from #52
- Target BPM comes from `LibraryItemView.tempo` (an `Option<String>` like "120 BPM") already on the detail page
- Follow the SVG patterns from the existing `LineChart` in `components/line_chart.rs`
- Use design system colour tokens (`--color-chart-line`, `--color-chart-secondary`, etc.) — never raw Tailwind colours
- Total: 18 tasks across 6 phases
