# Tasks: Practice Analytics Dashboard

**Input**: Design documents from `/specs/023-analytics-dashboard/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/api-changes.md, quickstart.md

**Tests**: Included — core analytics functions are pure and testable with synthetic data. Tests cover unit tests in `intrada-core` only (no new E2E tests).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Exact file paths included in descriptions

## Path Conventions

- **Core crate**: `crates/intrada-core/src/`
- **Web crate**: `crates/intrada-web/src/`
- **Tests**: `crates/intrada-core/src/analytics.rs` (inline `#[cfg(test)]` module)

---

## Phase 1: Setup

**Purpose**: Create new files and module structure for the analytics feature

- [x] T001 Create analytics module file at `crates/intrada-core/src/analytics.rs` with module-level doc comment and all type definitions: `AnalyticsView`, `WeeklySummary`, `PracticeStreak`, `DailyPracticeTotal`, `ItemRanking`, `ItemScoreTrend`, `ScorePoint` per data-model.md — all with `#[derive(Clone, Debug, Default, Serialize, Deserialize)]`
- [x] T002 Register the analytics module in `crates/intrada-core/src/lib.rs` with `pub mod analytics;` and re-export key types
- [x] T003 [P] Create stat card component file at `crates/intrada-web/src/components/stat_card.rs` with a `StatCard` Leptos component accepting `title: &'static str`, `value: String`, `subtitle: Option<String>` props — renders a glassmorphism card matching existing Card style
- [x] T004 [P] Create line chart component file at `crates/intrada-web/src/components/line_chart.rs` with placeholder `LineChart` Leptos component accepting `data: Vec<DailyPracticeTotal>` prop — renders empty SVG container with viewBox `0 0 600 200`
- [x] T005 Register new components in `crates/intrada-web/src/components/mod.rs` — add `pub mod stat_card;` and `pub mod line_chart;`
- [x] T006 Create analytics view file at `crates/intrada-web/src/views/analytics.rs` with placeholder `AnalyticsPage` Leptos component that renders a `PageHeading` with "Analytics" title
- [x] T007 Register analytics view in `crates/intrada-web/src/views/mod.rs` — add `pub mod analytics;`

**Checkpoint**: All new files exist, modules compile. `cargo check -p intrada-core && cargo check -p intrada-web` succeeds.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add `analytics` field to ViewModel and wire the route so all user stories can render on the dashboard

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T008 Add `analytics: Option<AnalyticsView>` field to the existing `ViewModel` struct in `crates/intrada-core/src/model.rs` — initialise as `None` in Default impl
- [x] T009 Add the `/analytics` route to the Leptos router in `crates/intrada-web/src/app.rs` — route renders the `AnalyticsPage` component
- [x] T010 Add "Analytics" tab to the bottom navigation bar in `crates/intrada-web/src/components/bottom_tab_bar.rs` — use chart/graph icon SVG, link to `/analytics`, highlight when active
- [x] T011 Implement the top-level `compute_analytics` function in `crates/intrada-core/src/analytics.rs` that takes `&[PracticeSession]` and `today: NaiveDate`, calls all sub-computation functions, and returns `AnalyticsView`
- [x] T012 Wire `compute_analytics` into the core app update logic in `crates/intrada-core/src/app.rs` — when sessions are loaded (existing `SessionsLoaded` or `DataLoaded` event), call `compute_analytics` with the loaded sessions and `Utc::now().date_naive()` (passed from shell), store result in `ViewModel.analytics`

**Checkpoint**: Foundation ready — navigating to `/analytics` shows the page heading, analytics tab is visible in nav. `cargo check` succeeds. `ViewModel.analytics` is populated when sessions load.

---

## Phase 3: User Story 1 — View Practice Overview (Priority: P1)

**Goal**: Musicians see weekly summary (total time + session count) and current practice streak on the dashboard.

**Independent Test**: Navigate to `/analytics` after creating sessions — verify weekly stats and streak display correctly. Empty state shown when no sessions exist.

### Tests for User Story 1

- [x] T013 [P] [US1] Write unit test `test_weekly_summary_basic` in `crates/intrada-core/src/analytics.rs` — create 3 sessions within the current ISO week, verify `WeeklySummary` returns correct `total_minutes` and `session_count`
- [x] T014 [P] [US1] Write unit test `test_weekly_summary_excludes_previous_week` in `crates/intrada-core/src/analytics.rs` — create sessions in previous week and current week, verify only current week sessions are counted
- [x] T015 [P] [US1] Write unit test `test_weekly_summary_empty` in `crates/intrada-core/src/analytics.rs` — pass empty session list, verify `WeeklySummary { total_minutes: 0, session_count: 0 }`
- [x] T016 [P] [US1] Write unit test `test_streak_consecutive_days` in `crates/intrada-core/src/analytics.rs` — create sessions on 3 consecutive days ending today, verify `PracticeStreak { current_days: 3 }`
- [x] T017 [P] [US1] Write unit test `test_streak_broken` in `crates/intrada-core/src/analytics.rs` — create sessions with a gap day, verify streak resets at the gap
- [x] T018 [P] [US1] Write unit test `test_streak_no_sessions_today` in `crates/intrada-core/src/analytics.rs` — create sessions on yesterday and day before, verify streak is 2 (counts from yesterday when today has no session)
- [x] T019 [P] [US1] Write unit test `test_streak_empty` in `crates/intrada-core/src/analytics.rs` — pass empty session list, verify `PracticeStreak { current_days: 0 }`

### Implementation for User Story 1

- [x] T020 [US1] Implement `compute_weekly_summary` function in `crates/intrada-core/src/analytics.rs` — filter sessions by ISO week of `today`, sum `total_duration_secs` (convert to minutes), count sessions
- [x] T021 [US1] Implement `compute_streak` function in `crates/intrada-core/src/analytics.rs` — extract unique dates from `started_at`, sort descending, count consecutive days backwards from today (or yesterday if today has no session)
- [x] T022 [US1] Render weekly summary in `crates/intrada-web/src/views/analytics.rs` — use two `StatCard` components showing "X sessions" and "Xh Ym" for the current week, read from `ViewModel.analytics.weekly_summary`
- [x] T023 [US1] Render practice streak in `crates/intrada-web/src/views/analytics.rs` — use `StatCard` component showing "X days" streak, read from `ViewModel.analytics.streak`
- [x] T024 [US1] Implement empty state in `crates/intrada-web/src/views/analytics.rs` — when `ViewModel.analytics` is `None` or sessions are empty, show encouraging message with a link to start a new session (FR-007, FR-011)
- [x] T025 [US1] Implement skeleton/placeholder loading UI in `crates/intrada-web/src/views/analytics.rs` — show animated placeholder cards while `ViewModel.analytics` is `None` and data is still loading (FR-012)

**Checkpoint**: User Story 1 complete — dashboard shows weekly summary, streak, and empty state. All US1 unit tests pass.

---

## Phase 4: User Story 2 — View Practice History Chart (Priority: P2)

**Goal**: Musicians see a 28-day line chart showing daily practice minutes as connected data points.

**Independent Test**: Create sessions across multiple days, navigate to `/analytics`, verify line chart displays correct daily totals with 28 data points.

### Tests for User Story 2

- [x] T026 [P] [US2] Write unit test `test_daily_totals_28_days` in `crates/intrada-core/src/analytics.rs` — create sessions across 5 different days within past 28 days, verify output has exactly 28 `DailyPracticeTotal` entries (oldest first), with correct minutes for session days and 0 for empty days
- [x] T027 [P] [US2] Write unit test `test_daily_totals_multiple_sessions_same_day` in `crates/intrada-core/src/analytics.rs` — create 3 sessions on the same day, verify that day's `minutes` equals the sum of all three
- [x] T028 [P] [US2] Write unit test `test_daily_totals_empty` in `crates/intrada-core/src/analytics.rs` — pass empty session list, verify 28 entries all with `minutes: 0`

### Implementation for User Story 2

- [x] T029 [US2] Implement `compute_daily_totals` function in `crates/intrada-core/src/analytics.rs` — generate 28 days (today minus 27 days through today), aggregate `total_duration_secs` per day (converted to minutes), fill gaps with 0
- [x] T030 [US2] Implement the `LineChart` SVG rendering in `crates/intrada-web/src/components/line_chart.rs` — calculate x/y coordinates from `Vec<DailyPracticeTotal>` within viewBox `0 0 600 200`, render `<polyline>` for the line and `<circle>` elements for data points, style with Tailwind-compatible CSS classes
- [x] T031 [US2] Add x-axis day labels and y-axis minute scale to the `LineChart` in `crates/intrada-web/src/components/line_chart.rs` — show abbreviated day labels (e.g., every 7th day), auto-scale y-axis based on max value
- [x] T032 [US2] Render the line chart section in `crates/intrada-web/src/views/analytics.rs` — add a Card containing the `LineChart` component with section heading "Practice History (28 days)", read data from `ViewModel.analytics.daily_totals`
- [x] T033 [US2] Add empty state for chart section in `crates/intrada-web/src/views/analytics.rs` — when all daily totals are 0, show message "No practice data for the past 28 days" with link to start a session (FR-007, FR-011)

**Checkpoint**: User Story 2 complete — 28-day line chart renders with correct daily totals. All US2 unit tests pass.

---

## Phase 5: User Story 3 — View Most Practised Items (Priority: P3)

**Goal**: Musicians see a ranked list of their top 10 most-practised items by total time, with session counts.

**Independent Test**: Create sessions with entries for different items, navigate to `/analytics`, verify items are ranked correctly by total practice time.

### Tests for User Story 3

- [x] T034 [P] [US3] Write unit test `test_top_items_ranking` in `crates/intrada-core/src/analytics.rs` — create sessions with entries for 5 different items with varying durations, verify `Vec<ItemRanking>` is sorted by `total_minutes` descending
- [x] T035 [P] [US3] Write unit test `test_top_items_max_10` in `crates/intrada-core/src/analytics.rs` — create entries for 15 items, verify only top 10 are returned
- [x] T036 [P] [US3] Write unit test `test_top_items_session_count` in `crates/intrada-core/src/analytics.rs` — create 3 sessions containing the same item, verify `session_count` is 3 and `total_minutes` sums correctly
- [x] T037 [P] [US3] Write unit test `test_top_items_empty` in `crates/intrada-core/src/analytics.rs` — pass empty session list, verify empty `Vec<ItemRanking>`

### Implementation for User Story 3

- [x] T038 [US3] Implement `compute_top_items` function in `crates/intrada-core/src/analytics.rs` — aggregate entries by `item_id` across all sessions, sum `duration_secs` (convert to minutes), count distinct sessions per item, sort by total_minutes descending, take top 10
- [x] T039 [US3] Render the most-practised items list in `crates/intrada-web/src/views/analytics.rs` — add a Card with section heading "Most Practised", iterate `ViewModel.analytics.top_items`, display each item's title, total minutes, and session count in a styled list
- [x] T040 [US3] Add empty state for most-practised section in `crates/intrada-web/src/views/analytics.rs` — when `top_items` is empty, show "No practice data yet" message with link to start a session (FR-007, FR-011)

**Checkpoint**: User Story 3 complete — ranked list of top 10 items displays correctly. All US3 unit tests pass.

---

## Phase 6: User Story 4 — View Confidence Score Trends (Priority: P4)

**Goal**: Musicians see score progression over time for their recently scored items (up to 5 items).

**Independent Test**: Create sessions with scored entries across multiple dates, navigate to `/analytics`, verify score trend data is displayed chronologically per item.

### Tests for User Story 4

- [x] T041 [P] [US4] Write unit test `test_score_trends_basic` in `crates/intrada-core/src/analytics.rs` — create 3 sessions scoring the same item with values 2, 3, 4, verify `ItemScoreTrend` has 3 `ScorePoint` entries in chronological order with correct `latest_score: 4`
- [x] T042 [P] [US4] Write unit test `test_score_trends_max_5_items` in `crates/intrada-core/src/analytics.rs` — create scored entries for 8 items, verify only 5 most recently scored items are returned
- [x] T043 [P] [US4] Write unit test `test_score_trends_excludes_unscored` in `crates/intrada-core/src/analytics.rs` — create sessions with some entries scored and some unscored, verify only scored items appear in trends
- [x] T044 [P] [US4] Write unit test `test_score_trends_empty` in `crates/intrada-core/src/analytics.rs` — pass sessions with no scored entries, verify empty `Vec<ItemScoreTrend>`

### Implementation for User Story 4

- [x] T045 [US4] Implement `compute_score_trends` function in `crates/intrada-core/src/analytics.rs` — collect all entries with `score: Some(n)`, group by `item_id`, build chronological `ScorePoint` lists, sort items by most recent score date, take top 5
- [x] T046 [US4] Render score trends section in `crates/intrada-web/src/views/analytics.rs` — add a Card with section heading "Score Trends", iterate `ViewModel.analytics.score_trends`, display each item's title and its score progression (e.g., score dots or small inline visualisation)
- [x] T047 [US4] Add empty state for score trends section in `crates/intrada-web/src/views/analytics.rs` — when `score_trends` is empty, show message explaining the scoring feature and how to start scoring items (FR-007)

**Checkpoint**: User Story 4 complete — score trends display for recently scored items. All US4 unit tests pass.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Error handling, responsiveness, and final validation across all user stories

- [x] T048 Implement error state with retry button in `crates/intrada-web/src/views/analytics.rs` — when session fetch fails, display inline error message with a retry button that re-triggers data loading (FR-013)
- [x] T049 [P] Ensure responsive layout in `crates/intrada-web/src/views/analytics.rs` — verify dashboard adapts to mobile viewport widths using Tailwind responsive classes, stat cards stack vertically on narrow screens, chart scales via SVG viewBox
- [x] T050 [P] Ensure ended-early sessions are included in all analytics calculations in `crates/intrada-core/src/analytics.rs` — verify no filtering by session status exists in any compute function (FR-009)
- [x] T051 Run `cargo test -p intrada-core` to verify all analytics unit tests pass
- [x] T052 Run `cargo clippy -- -D warnings` to verify zero warnings across all crates
- [x] T053 Run `cargo check -p intrada-web` to verify clean WASM compilation
- [x] T054 Run quickstart.md verification steps — manual check of navigation, empty state, weekly summary, streak, line chart, most practised items, score trends, loading state, and responsive layout

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion — BLOCKS all user stories
- **User Stories (Phases 3–6)**: All depend on Foundational phase completion
  - US1 (P1), US2 (P2), US3 (P3), US4 (P4) can proceed in parallel after Phase 2
  - Within each story: tests first (fail), then implementation (pass)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 — no dependencies on other stories
- **User Story 2 (P2)**: Can start after Phase 2 — no dependencies on other stories
- **User Story 3 (P3)**: Can start after Phase 2 — no dependencies on other stories
- **User Story 4 (P4)**: Can start after Phase 2 — no dependencies on other stories

### Within Each User Story

- Tests written FIRST (should fail before implementation)
- Core computation functions before view rendering
- Implementation before empty states
- All tests must pass before story is considered complete

### Parallel Opportunities

- **Phase 1**: T003 and T004 can run in parallel (different files); T006 can run in parallel with T003/T004
- **Phase 3–6**: All four user story phases can run in parallel after Phase 2
- **Within each story**: All test tasks marked [P] can run in parallel
- **Phase 7**: T049 and T050 can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Write test_weekly_summary_basic in crates/intrada-core/src/analytics.rs"      # T013
Task: "Write test_weekly_summary_excludes_previous_week in same file"                 # T014
Task: "Write test_weekly_summary_empty in same file"                                  # T015
Task: "Write test_streak_consecutive_days in same file"                               # T016
Task: "Write test_streak_broken in same file"                                         # T017
Task: "Write test_streak_no_sessions_today in same file"                              # T018
Task: "Write test_streak_empty in same file"                                          # T019
# Note: All tests are in the same file but different test functions — parallelizable
```

## Parallel Example: All User Stories

```bash
# After Phase 2 completes, launch all stories in parallel:
Task: "User Story 1 — Overview (Phase 3)"     # T013–T025
Task: "User Story 2 — History Chart (Phase 4)" # T026–T033
Task: "User Story 3 — Most Practised (Phase 5)" # T034–T040
Task: "User Story 4 — Score Trends (Phase 6)"  # T041–T047
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T007)
2. Complete Phase 2: Foundational (T008–T012)
3. Complete Phase 3: User Story 1 (T013–T025)
4. **STOP and VALIDATE**: Dashboard shows weekly summary, streak, empty state
5. Deploy/demo if ready — delivers immediate motivational value

### Incremental Delivery

1. Setup + Foundational → Foundation ready (T001–T012)
2. Add User Story 1 → Weekly stats + streak visible (MVP!)
3. Add User Story 2 → 28-day trend line chart
4. Add User Story 3 → Most-practised items list
5. Add User Story 4 → Score trend progression
6. Polish → Error handling, responsiveness, final validation
7. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files or independent functions, no dependencies
- [Story] label maps task to specific user story for traceability
- All analytics computation is pure (no I/O) in `intrada-core` — testable without browser
- No new API endpoints or database changes — reads from existing `GET /api/sessions`
- SVG chart is rendered inline via Leptos `view!` macro — zero new dependencies
- `today` parameter passed to all date-based functions for deterministic testing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
