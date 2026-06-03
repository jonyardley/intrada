# Tasks: Weekly Practice Summary

**Input**: Design documents from `/specs/153-weekly-practice-summary/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included — the spec defines independent test criteria per user story, and the plan calls for ~25 unit tests.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Core (pure logic)**: `crates/intrada-core/src/`
- **Web shell (UI)**: `crates/intrada-web/src/`
- **E2E tests**: `e2e/fixtures/`

---

## Phase 1: Setup (New Types)

**Purpose**: Add new types to the analytics module that all user stories depend on.

- [x] T001 Add `Direction` enum (Up, Down, Same) with Serialize/Deserialize and Default derives in `crates/intrada-core/src/analytics.rs`
- [x] T002 Add `NeglectedItem` struct (item_id, item_title, days_since_practice: Option<u32>) with Serialize/Deserialize derives in `crates/intrada-core/src/analytics.rs`
- [x] T003 Add `ScoreChange` struct (item_id, item_title, previous_score: Option<u8>, current_score, delta: i8, is_new: bool) with Serialize/Deserialize derives in `crates/intrada-core/src/analytics.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Extend core data structures and wiring that MUST be complete before any user story implementation.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T004 Extend `WeeklySummary` struct with comparison fields: `items_covered`, `prev_total_minutes`, `prev_session_count`, `prev_items_covered`, `time_direction`, `sessions_direction`, `items_direction`, `has_prev_week_data` in `crates/intrada-core/src/analytics.rs`. Update Default impl to include new fields.
- [x] T005 Add `neglected_items: Vec<NeglectedItem>` and `score_changes: Vec<ScoreChange>` fields to `AnalyticsView` struct in `crates/intrada-core/src/analytics.rs`. Update the Default derive or impl accordingly.
- [x] T006 Extend `compute_analytics` function signature from `(sessions: &[PracticeSession], today: NaiveDate)` to `(sessions: &[PracticeSession], items: &[Item], today: NaiveDate)` in `crates/intrada-core/src/analytics.rs`. Add `use crate::domain::item::Item;` import. Wire new `items` param to stub calls for `compute_neglected_items` (return empty Vec for now). Wire `compute_score_changes` (return empty Vec for now).
- [x] T007 [P] Update the `compute_analytics` call site in `crates/intrada-core/src/app.rs` (around line 220) to pass `&model.items` as the second argument: `compute_analytics(&model.sessions, &model.items, today)`.
- [x] T008 Fix all existing unit tests in `crates/intrada-core/src/analytics.rs` that call `compute_analytics` or `compute_weekly_summary` to match the new signatures. Pass `&[]` for items where not needed. Verify all existing tests still pass with `cargo test -p intrada-core -- analytics`.

**Checkpoint**: Foundation ready — `cargo test -p intrada-core` passes, `cargo clippy -- -D warnings` clean. User story implementation can now begin.

---

## Phase 3: User Story 1 — Week-over-week comparison (Priority: P1) 🎯 MVP

**Goal**: Musicians see this week's practice time, session count, and items covered compared to last week, with directional indicators (up/down/same).

**Independent Test**: Seed two weeks of sessions. Call `compute_weekly_summary`. Verify current and previous week values and direction indicators are correct. In the UI, the three comparison metrics render in a card with the old stat cards removed.

### Tests for User Story 1

- [x] T009 [US1] Write unit tests for extended `compute_weekly_summary` in `crates/intrada-core/src/analytics.rs`:
  - Test current + previous week with sessions in both weeks (verify all 3 metrics + directions)
  - Test sessions this week only, none last week (`has_prev_week_data: false`)
  - Test sessions last week only, none this week (Monday morning — all current values 0)
  - Test `items_covered` counts distinct item_ids from session entries
  - Test direction indicators: Up when current > prev, Down when current < prev, Same when equal
  - Test week boundary: Sunday 23:55 session belongs to ending week, Monday 00:05 to new week

### Implementation for User Story 1

- [x] T010 [US1] Extend `compute_weekly_summary` function in `crates/intrada-core/src/analytics.rs` to: (1) compute previous ISO week totals alongside current week, (2) count distinct `item_id` values from session entries for both weeks, (3) compute `Direction` for each metric, (4) set `has_prev_week_data` based on whether previous week had any sessions.
- [x] T011 [US1] Restructure `AnalyticsDashboard` component in `crates/intrada-web/src/views/analytics.rs`: remove the "This Week" and "Sessions" `StatCard` components from the 3-column grid. Keep only the "Streak" `StatCard` as a single card (no longer in a 3-column grid).
- [x] T012 [US1] Create `WeekComparisonRow` component in `crates/intrada-web/src/views/analytics.rs` that renders 3 metrics (time, sessions, items) in a `grid grid-cols-3 gap-3` layout. Each metric shows: the current value (large, `text-primary`), a direction arrow with comparison text (e.g. "↑ from 3" using `text-success-text` for up, `text-muted` for down/same), and a label (`field-label` utility). When `has_prev_week_data` is false, show "no data last week" instead of comparison.
- [x] T013 [US1] Integrate `WeekComparisonRow` into the analytics page in `crates/intrada-web/src/views/analytics.rs`: wrap it in a `Card` component with heading "This Week", positioned between the Streak stat card and the Practice History chart. Ensure the entire weekly summary `Card` is hidden when `ViewModel.analytics` is `None` (FR-010).

**Checkpoint**: User Story 1 complete — analytics page shows streak card + weekly comparison card with 3 metrics. `cargo test -p intrada-core -- analytics` passes. Old stat cards removed.

---

## Phase 4: User Story 2 — Items covered & neglected (Priority: P2)

**Goal**: Musicians see which library items they haven't touched in 14+ days (or never), helping them plan their next session.

**Independent Test**: Seed a library with 10 items and sessions covering 4 of them. Call `compute_neglected_items`. Verify 5 or fewer neglected items returned, never-practised at top, ordered by gap length. In the UI, the "Needs attention" section appears inside the weekly summary card.

### Tests for User Story 2

- [x] T014 [US2] Write unit tests for `compute_neglected_items` in `crates/intrada-core/src/analytics.rs`:
  - Test 10 items, 4 practised this week → 6 neglected, capped at 5
  - Test never-practised items sort to top with `days_since_practice: None`
  - Test ordering by days since practice descending (longest gap first)
  - Test all items practised within 14 days → empty result
  - Test empty library → empty result
  - Test deleted item (in session but not in items list) → not in neglected list
  - Test item practised 13 days ago (within window) → not neglected
  - Test item practised exactly 14 days ago → neglected

### Implementation for User Story 2

- [x] T015 [US2] Implement `compute_neglected_items(sessions: &[PracticeSession], items: &[Item], today: NaiveDate) -> Vec<NeglectedItem>` in `crates/intrada-core/src/analytics.rs`. Logic: (1) find all item_ids with at least one session entry in the 14 days up to `today`, (2) for each item in `items` not in that set, create a `NeglectedItem`, (3) compute `days_since_practice` by scanning ALL sessions for the most recent entry for that item (None if never practised), (4) sort: None first, then descending by days, (5) truncate to 5.
- [x] T016 [US2] Update `compute_analytics` in `crates/intrada-core/src/analytics.rs` to replace the stub `compute_neglected_items` call with the real implementation.
- [x] T017 [US2] Create `NeglectedItemsList` component in `crates/intrada-web/src/views/analytics.rs` that renders a heading "Needs attention" (`card-title`) and a list of up to 5 items. Each item shows the title (`text-sm text-primary`) and a subtitle: "never practised" (`text-xs text-muted`) for items with `days_since_practice: None`, or "X days ago" for others. The entire section is hidden when the list is empty (FR-007).
- [x] T018 [US2] Integrate `NeglectedItemsList` into the weekly summary `Card` in `crates/intrada-web/src/views/analytics.rs`, positioned below the `WeekComparisonRow`. On desktop (≥640px) it sits in a 2-column grid alongside the score changes section; on mobile (<640px) it stacks vertically.

**Checkpoint**: User Story 2 complete — neglected items appear in the weekly summary card when applicable, hidden when all items are covered. Tests pass.

---

## Phase 5: User Story 3 — Score improvements (Priority: P3)

**Goal**: Musicians see which items had score changes this week, reinforcing that practice is working.

**Independent Test**: Seed sessions where one item went from score 2 to score 4 this week. Call `compute_score_changes`. Verify the item appears with previous=2, current=4, delta=+2. In the UI, the "Improvements" section appears in the weekly summary card with neutral language.

### Tests for User Story 3

- [x] T019 [US3] Write unit tests for `compute_score_changes` in `crates/intrada-core/src/analytics.rs`:
  - Test item scored 2 last week, 4 this week → delta +2
  - Test item scored 4 last week, 3 this week → delta -1 (shown neutrally, no negative language)
  - Test item scored for first time this week → `is_new: true`, `previous_score: None`, `delta: 0`
  - Test no items scored this week → empty result
  - Test more than 5 score changes → capped at 5, sorted by largest absolute delta
  - Test item scored multiple times this week → uses latest score this week
  - Test item scored same as last week → not included (no change)

### Implementation for User Story 3

- [x] T020 [US3] Implement `compute_score_changes(sessions: &[PracticeSession], today: NaiveDate) -> Vec<ScoreChange>` in `crates/intrada-core/src/analytics.rs`. Logic: (1) collect all scored entries, partition into this-week and pre-this-week by ISO week, (2) for each item scored this week, find latest score this week and latest score before this week, (3) if scores differ or item is newly scored, create a `ScoreChange`, (4) sort by absolute delta descending, (5) truncate to 5.
- [x] T021 [US3] Update `compute_analytics` in `crates/intrada-core/src/analytics.rs` to replace the stub `compute_score_changes` call with the real implementation.
- [x] T022 [US3] Create `ScoreChangesList` component in `crates/intrada-web/src/views/analytics.rs` that renders a heading "Improvements" (`card-title`) and a list of up to 5 items. Each item shows: title (`text-sm text-primary`), score transition "X → Y" (`text-sm text-secondary`), and signed delta "(+N)" or "(-N)" using `text-accent-text`. For newly scored items show "new" instead of delta. Neutral framing only — no words like "worse" or "declined" (FR-009). Hidden when empty (FR-007).
- [x] T023 [US3] Integrate `ScoreChangesList` into the weekly summary `Card` in `crates/intrada-web/src/views/analytics.rs`, positioned alongside `NeglectedItemsList` in the 2-column grid (desktop) or stacked below it (mobile).

**Checkpoint**: User Story 3 complete — score changes appear in the weekly summary card. All three user stories work together. Tests pass.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: E2E compatibility, responsive verification, and final validation.

- [x] T024 Update E2E analytics mock data in `e2e/fixtures/api-mock.ts` to include the new `AnalyticsView` fields: extended `weekly_summary` with comparison fields (`prev_total_minutes`, `prev_session_count`, `items_covered`, `prev_items_covered`, direction fields, `has_prev_week_data`), `neglected_items: []`, and `score_changes: []`.
- [x] T025 Verify responsive layout in `crates/intrada-web/src/views/analytics.rs`: confirm mobile (<640px) stacks neglected items and score changes vertically, desktop (≥640px) shows them side-by-side. Adjust Tailwind classes if needed.
- [x] T026 Run full verification per `specs/153-weekly-practice-summary/quickstart.md`: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test -p intrada-core`, `cargo test`, `cd e2e && npx playwright test`, visual check in browser.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion — BLOCKS all user stories
- **User Stories (Phase 3–5)**: All depend on Phase 2 completion
  - US1, US2, US3 share the same files so should be executed sequentially in priority order
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2). No dependencies on other stories. Removes old stat cards and creates the weekly summary Card container that US2 and US3 will add sections to.
- **User Story 2 (P2)**: Can start after US1 (Phase 3). Adds a section into the Card created by US1. Core computation is independent but UI integration depends on the Card structure from US1.
- **User Story 3 (P3)**: Can start after US2 (Phase 4). Adds a section alongside US2's section in the 2-column grid. Core computation is independent but UI integration depends on the layout from US1+US2.

### Within Each User Story

- Tests FIRST — write tests, verify they compile (they'll fail until implementation)
- Core computation functions before UI components
- UI components before integration into the page
- All tasks within a story share the same two files (`analytics.rs` in core and web)

### Parallel Opportunities

- **Phase 2**: T007 (app.rs) can run in parallel with T004–T006 (analytics.rs) since they are different files
- **Cross-story core computation**: T010/T015/T020 (computation functions) could theoretically be written in parallel since they're independent functions, but they're in the same file — use sequential execution
- **E2E mock (T024)**: Can be done in parallel with any Phase 3–5 work since it's a different file

---

## Parallel Example: Phase 2

```bash
# These can run in parallel (different files):
Task T006: "Extend compute_analytics signature in crates/intrada-core/src/analytics.rs"
Task T007: "Update compute_analytics call site in crates/intrada-core/src/app.rs"
# (T007 needs T006's signature to exist, so in practice run T006 first then T007 immediately after)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003) — add new types
2. Complete Phase 2: Foundational (T004–T008) — wire everything, fix existing tests
3. Complete Phase 3: User Story 1 (T009–T013) — week-over-week comparison
4. **STOP and VALIDATE**: `cargo test -p intrada-core`, visual check of analytics page
5. The analytics page now shows streak + weekly comparison — already more useful than before

### Incremental Delivery

1. Setup + Foundational → Types and wiring ready
2. Add User Story 1 → Test → analytics page has comparison metrics (MVP!)
3. Add User Story 2 → Test → "Needs attention" section surfaces neglected items
4. Add User Story 3 → Test → "Improvements" section shows score changes
5. Polish → E2E mock update, responsive check, full verification

### Single-Developer Strategy (Recommended)

Since all core work is in `analytics.rs` and all UI work is in `views/analytics.rs`:

1. Complete all core computation (Phases 1–2, then T009–T010, T014–T016, T019–T021)
2. Complete all UI work (T011–T013, T017–T018, T022–T023)
3. Polish (T024–T026)

This minimises file-switching overhead and allows natural flow from types → computation → tests → UI.

---

## Notes

- All computation functions are pure (no I/O) and accept `today: NaiveDate` for determinism
- The same two files (`analytics.rs` in core and web) are modified throughout — sequential execution within stories is safest
- Score changes use neutral language only — no "declined", "dropped", "worse" (FR-009)
- Sections hidden when empty — never show empty lists (FR-007)
- ISO week numbering (Monday–Sunday) consistent with existing `compute_weekly_summary`
- Never-practised items use `days_since_practice: None`, not a sentinel value like 9999
- Commit after each phase checkpoint for safe rollback points
