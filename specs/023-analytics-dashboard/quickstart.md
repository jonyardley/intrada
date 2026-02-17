# Quickstart: Practice Analytics Dashboard

**Feature**: 023-analytics-dashboard
**Date**: 2026-02-17

## Prerequisites

- Rust stable toolchain (1.75+)
- Trunk (`cargo install trunk`)
- Tailwind CSS v4 standalone CLI

## Verification Steps

### 1. Core tests pass

```bash
cargo test -p intrada-core
```

Expected: All existing tests pass + new analytics unit tests pass (streak, weekly summary, daily totals, item rankings, score trends).

### 2. Web crate compiles

```bash
cargo check -p intrada-web
```

Expected: Clean compilation with no errors.

### 3. Clippy clean

```bash
cargo clippy -- -D warnings
```

Expected: 0 warnings across all crates.

### 4. Manual verification

Start the development server:

```bash
trunk serve --open
```

Then verify:

1. **Navigation**: Click "Analytics" in the bottom tab bar → dashboard loads at `/analytics`
2. **Empty state**: If no sessions exist, see encouraging message with link to start a session
3. **Weekly summary**: Create a practice session → dashboard shows "1 session" and correct duration for this week
4. **Streak**: After completing a session today, streak shows "1 day"
5. **Line chart**: With sessions on multiple days, the 28-day chart renders as a line with connected points
6. **Most practised items**: Items appear ranked by total practice time with session counts
7. **Score trends**: Score an item during a session → score trend appears on dashboard
8. **Loading state**: Skeleton UI visible briefly while data loads
9. **Responsive**: Dashboard layout adapts to mobile viewport widths

### 5. Acceptance criteria mapping

| Requirement | Verification |
|-------------|-------------|
| FR-001 | Dashboard accessible from bottom nav |
| FR-002 | Weekly stats card shows correct totals |
| FR-003 | Streak displays consecutive days accurately |
| FR-004 | Line chart renders 28 days of data |
| FR-005 | Top 10 items listed with time and session count |
| FR-006 | Score trends show per-item progression |
| FR-007 | Empty states for each section when no data |
| FR-008 | No new database tables or API endpoints |
| FR-009 | Ended-early sessions included in calculations |
| FR-010 | No edit/delete/create actions on dashboard |
| FR-011 | Empty states link to new session flow |
| FR-012 | Skeleton UI during loading |
| FR-013 | Error with retry button on fetch failure |
