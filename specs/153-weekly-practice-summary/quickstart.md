# Quickstart: Weekly Practice Summary

**Feature**: 153-weekly-practice-summary
**Date**: 2026-02-24

## Prerequisites

- Rust stable toolchain (1.89.0+)
- trunk 0.21.x for web dev server
- Node.js (for Playwright E2E tests)

## Build & Test

```bash
# Run all workspace tests (includes new analytics unit tests)
cargo test

# Run core tests only (fastest feedback loop during development)
cargo test -p intrada-core

# Lint and format checks
cargo fmt --check
cargo clippy -- -D warnings
```

## Verification Steps

### Step 1: Unit tests pass (core computation)

```bash
cargo test -p intrada-core -- analytics
```

**Expected**: All analytics tests pass, including new tests for:
- `compute_weekly_summary` with previous-week comparison
- `compute_neglected_items` with 14-day lookback
- `compute_score_changes` with week-over-week deltas
- Edge cases: no previous week data, no items, never-practised items, Monday morning

### Step 2: Full workspace compiles and tests pass

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

**Expected**: Zero warnings, zero failures.

### Step 3: Visual verification in browser

```bash
# Terminal 1: Start API server
cd crates/intrada-api && cargo run

# Terminal 2: Start web dev server
cd crates/intrada-web && trunk serve
```

Navigate to `http://localhost:8080` → sign in → go to Analytics page.

**Verify with session data**:
1. ✅ Streak stat card appears at the top (single card, not 3-column grid)
2. ✅ "This Week" summary card appears below streak
3. ✅ Three comparison metrics visible: time, sessions, items covered
4. ✅ Each metric shows current value + directional indicator + previous week value
5. ✅ "Needs attention" section shows items not practised in 14+ days (if any)
6. ✅ "Improvements" section shows score changes this week (if any)
7. ✅ Sections with no data are hidden (not empty lists)
8. ✅ Practice History (28 days) chart appears below the summary card

**Verify without session data**:
1. ✅ The analytics empty state renders normally (no weekly summary card)
2. ✅ No errors in browser console

**Verify responsive layout**:
1. ✅ Mobile (<640px): metrics in 3-column grid, neglected + scores stacked vertically
2. ✅ Desktop (≥640px): metrics in 3-column grid, neglected + scores side-by-side

### Step 4: E2E tests pass

```bash
cd e2e && npx playwright test
```

**Expected**: All existing E2E tests pass. The analytics mock data in `api-mock.ts`
should include the new `AnalyticsView` fields so the analytics page renders correctly.

## Constitution Compliance

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ | Pure functions, clear types, no dead code |
| II. Testing | ✅ | Unit tests for all computation paths + edge cases |
| III. UX Consistency | ✅ | Uses Card, StatCard, design tokens; accessible |
| IV. Performance | ✅ | Pure computation, no new API calls, no new storage |
| V. Architecture | ✅ | All logic in intrada-core, zero I/O, today param |
| VI. Inclusive Design | ✅ | Neutral language, no shaming, positive framing |
