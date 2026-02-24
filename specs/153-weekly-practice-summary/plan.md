# Implementation Plan: Weekly Practice Summary

**Branch**: `153-weekly-practice-summary` | **Date**: 2026-02-24 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/153-weekly-practice-summary/spec.md`

## Summary

Add a weekly practice summary to the analytics page that compares this week's practice
to last week across three metrics (time, sessions, items covered), surfaces neglected
library items (not practised in 14+ days), and highlights score changes. The summary
replaces the existing "This Week" and "Sessions" stat cards while retaining the streak
card. All computation is pure (no I/O, no new API endpoints) — extending the existing
`compute_analytics()` function in `intrada-core/src/analytics.rs`.

## Technical Context

**Language/Version**: Rust stable (1.89.0), 2021 edition
**Primary Dependencies**: crux_core 0.17.0-rc2, serde 1, chrono 0.4, leptos 0.8.x (CSR), Tailwind CSS v4
**Storage**: N/A — no new persistence; reads existing `PracticeSession` and `Item` data
**Testing**: cargo test (unit tests in intrada-core, existing E2E via Playwright)
**Target Platform**: WASM (browser) via Leptos CSR
**Project Type**: Web application (three-crate workspace)
**Performance Goals**: Analytics computation completes in <10ms for 1000 sessions
**Constraints**: Pure core (no I/O), all computation accepts `today` parameter for determinism
**Scale/Scope**: Single page modification (analytics), ~4 files changed, ~3 new view components

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | Pure functions with single responsibility; clear type names; no dead code |
| II. Testing Standards | ✅ Pass | All computation functions will have unit tests; test helpers already exist in analytics.rs |
| III. UX Consistency | ✅ Pass | Uses existing Card component, design tokens, consistent layout patterns; WCAG 2.1 AA with ARIA attributes |
| IV. Performance | ✅ Pass | Pure computation from in-memory data; no new API calls; no new storage; no WASM bundle impact beyond minimal code |
| V. Architecture Integrity | ✅ Pass | All logic in intrada-core (pure, no I/O); `today` parameter for testability; shell only renders |
| VI. Inclusive Design | ✅ Pass | Neutral language for score changes (FR-009); "so far this week" framing; no shaming; positive framing for neglected items ("Needs attention" not "You missed") |

No violations. Gate passed.

## Project Structure

### Documentation (this feature)

```text
specs/153-weekly-practice-summary/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (no new API contracts)
│   └── README.md
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
  intrada-core/
  └── src/
      └── analytics.rs           # MODIFY — extend WeeklySummary, add Direction enum,
                                 #   add NeglectedItem, ScoreChange structs,
                                 #   add compute_neglected_items(), compute_score_changes(),
                                 #   extend compute_analytics() signature & body,
                                 #   add ~25 unit tests

  intrada-web/
  └── src/
      └── views/
          └── analytics.rs       # MODIFY — replace stat cards with weekly summary card,
                                 #   add WeekComparisonRow, NeglectedItemsList,
                                 #   ScoreChangesList components,
                                 #   update AnalyticsDashboard destructuring

  intrada-core/
  └── src/
      └── app.rs                 # MODIFY — pass model.items to compute_analytics()

e2e/
  └── fixtures/
      └── api-mock.ts            # MODIFY — update analytics mock to include new fields
```

**Structure Decision**: Existing three-crate workspace. Changes are confined to the
core analytics module (computation), the web analytics view (rendering), and the core
app.rs (wiring). No new crates, no new modules, no new API routes.

## Implementation Details

### 1. Core: Extend analytics types (`analytics.rs`)

**New types:**

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
    Same,
}
```

**Extended `WeeklySummary`:**

```rust
pub struct WeeklySummary {
    pub total_minutes: u32,
    pub session_count: usize,
    pub items_covered: usize,           // NEW
    pub prev_total_minutes: u32,        // NEW
    pub prev_session_count: usize,      // NEW
    pub prev_items_covered: usize,      // NEW
    pub time_direction: Direction,      // NEW
    pub sessions_direction: Direction,  // NEW
    pub items_direction: Direction,     // NEW
    pub has_prev_week_data: bool,       // NEW
}
```

**New structs:**

```rust
pub struct NeglectedItem {
    pub item_id: String,
    pub item_title: String,
    pub days_since_practice: Option<u32>,  // None = never practised
}

pub struct ScoreChange {
    pub item_id: String,
    pub item_title: String,
    pub previous_score: Option<u8>,  // None = newly scored
    pub current_score: u8,
    pub delta: i8,
    pub is_new: bool,
}
```

**New fields on `AnalyticsView`:**

```rust
pub neglected_items: Vec<NeglectedItem>,
pub score_changes: Vec<ScoreChange>,
```

### 2. Core: New computation functions (`analytics.rs`)

**`compute_weekly_summary`** — extend to compute both current and previous ISO week
totals, plus items-covered count (distinct `item_id` values from session entries).

**`compute_neglected_items(sessions, items, today)`** — new function:
1. Collect all item_ids practised in the 14 days before `today`
2. For each item in the current library not in that set, create a `NeglectedItem`
3. For never-practised items: `days_since_practice = None`
4. For previously practised items: compute days since their most recent session entry
5. Sort: `None` first, then descending by `days_since_practice`
6. Truncate to 5

**`compute_score_changes(sessions, today)`** — new function:
1. Partition scored entries into this-week and pre-this-week
2. For each item scored this week, find its latest score this week and latest score before this week
3. If scores differ (or item is newly scored), create a `ScoreChange`
4. Sort by absolute delta descending
5. Truncate to 5

**`compute_analytics`** — signature changes from `(sessions, today)` to `(sessions, items, today)`.
Add calls to `compute_neglected_items` and `compute_score_changes`.

### 3. Core: Wire items into analytics (`app.rs`)

Change the `compute_analytics` call at lines 216–221:

```rust
// Before:
Some(compute_analytics(&model.sessions, today))

// After:
Some(compute_analytics(&model.sessions, &model.items, today))
```

### 4. Web: Update analytics view (`views/analytics.rs`)

**Replace stat card grid:**

```rust
// Before: 3-column grid with This Week, Sessions, Streak
// After: Single Streak StatCard, then the weekly summary Card

<StatCard title="Streak" value=streak_display subtitle="days" />
```

**New weekly summary card** (inside `Card`):

```
┌─ This Week ───────────────────────────┐
│  [WeekComparisonRow: 3 metrics]       │
│  [NeglectedItemsList] [ScoreChanges]  │
└───────────────────────────────────────┘
```

**New sub-components** (private to `analytics.rs` or extracted to components):

- `WeekComparisonRow` — renders 3 metrics in a grid, each with value + direction arrow + comparison text
- `NeglectedItemsList` — renders up to 5 items with "X days ago" or "never practised" labels
- `ScoreChangesList` — renders up to 5 items with "before → after (+delta)" format

All sections hidden when their data is empty (FR-007).

### 5. E2E: Update mock data (`api-mock.ts`)

Add the new fields to the analytics mock response:
- `neglected_items: []` (empty by default)
- `score_changes: []` (empty by default)
- Extended `weekly_summary` with comparison fields

## Implementation Order

1. **Core types** — Add `Direction` enum, extend `WeeklySummary`, add `NeglectedItem` and `ScoreChange` structs, update `AnalyticsView`
2. **Core computation** — Implement `compute_neglected_items()`, `compute_score_changes()`, extend `compute_weekly_summary()` with comparison logic
3. **Core wiring** — Update `compute_analytics()` signature, pass items from `app.rs`
4. **Core tests** — ~25 unit tests covering all computation functions and edge cases
5. **Web view** — Restructure analytics page, remove old stat cards, add weekly summary card with sub-components
6. **E2E mock update** — Update analytics mock data shape
7. **Verification** — Run full test suite, visual check, responsive check

## Verification

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p intrada-core    # Analytics computation + edge case tests
cargo test                    # Full workspace
cd e2e && npx playwright test # E2E tests
```

Manual: Navigate to analytics page with session data, verify weekly summary displays
correctly with comparison indicators. Check empty state. Check responsive layout.

## Complexity Tracking

> No Constitution Check violations. No complexity justifications needed.
