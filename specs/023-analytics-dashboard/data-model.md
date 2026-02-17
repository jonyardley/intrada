# Data Model: Practice Analytics Dashboard

**Feature**: 023-analytics-dashboard
**Date**: 2026-02-17

## Overview

No new persisted entities. All structures below are computed view models derived from the existing `PracticeSession` and `SetlistEntry` data.

## New Types (in `intrada-core`)

### AnalyticsView

Top-level analytics container, added to the existing `ViewModel`.

```
AnalyticsView
├── weekly_summary: WeeklySummary
├── streak: PracticeStreak
├── daily_totals: Vec<DailyPracticeTotal>   # 28 entries, oldest first
├── top_items: Vec<ItemRanking>             # up to 10, sorted by total_minutes DESC
└── score_trends: Vec<ItemScoreTrend>       # up to 5 most recently scored items
```

### WeeklySummary

Aggregated stats for the current ISO week (Monday–Sunday).

| Field | Type | Description |
|-------|------|-------------|
| total_minutes | u32 | Sum of all session durations this week |
| session_count | usize | Number of sessions started this week |

### PracticeStreak

Consecutive-day practice count.

| Field | Type | Description |
|-------|------|-------------|
| current_days | u32 | Consecutive days with ≥1 session, counted backwards from today |

### DailyPracticeTotal

One entry per day for the 28-day history chart.

| Field | Type | Description |
|-------|------|-------------|
| date | String | ISO date (YYYY-MM-DD) |
| minutes | u32 | Total practice minutes for this day (0 if no sessions) |

### ItemRanking

Per-item aggregation for the "most practised" list.

| Field | Type | Description |
|-------|------|-------------|
| item_id | String | Library item ID |
| item_title | String | Title from session entry (survives item deletion) |
| item_type | String | "Piece" or "Exercise" |
| total_minutes | u32 | Sum of duration across all entries for this item |
| session_count | usize | Number of distinct sessions containing this item |

### ItemScoreTrend

Score progression for a single item.

| Field | Type | Description |
|-------|------|-------------|
| item_id | String | Library item ID |
| item_title | String | Title from session entry |
| scores | Vec<ScorePoint> | Chronological score points (oldest first) |
| latest_score | u8 | Most recent score value (1–5) |

### ScorePoint

Single data point in a score trend.

| Field | Type | Description |
|-------|------|-------------|
| date | String | ISO date (YYYY-MM-DD) from session started_at |
| score | u8 | Confidence score (1–5) |

## ViewModel Changes

The existing `ViewModel` struct gains one new field:

```
ViewModel (existing)
├── items: Vec<LibraryItemView>           # unchanged
├── sessions: Vec<PracticeSessionView>    # unchanged
├── active_session: Option<...>           # unchanged
├── building_setlist: Option<...>         # unchanged
├── summary: Option<...>                  # unchanged
├── session_status: String                # unchanged
├── error: Option<String>                 # unchanged
└── analytics: Option<AnalyticsView>      # NEW — None until sessions loaded
```

## Computation Functions

All in `intrada-core/src/analytics.rs`:

| Function | Input | Output |
|----------|-------|--------|
| `compute_analytics` | `&[PracticeSession], today: NaiveDate` | `AnalyticsView` |
| `compute_weekly_summary` | `&[PracticeSession], today: NaiveDate` | `WeeklySummary` |
| `compute_streak` | `&[PracticeSession], today: NaiveDate` | `PracticeStreak` |
| `compute_daily_totals` | `&[PracticeSession], today: NaiveDate` | `Vec<DailyPracticeTotal>` |
| `compute_top_items` | `&[PracticeSession]` | `Vec<ItemRanking>` |
| `compute_score_trends` | `&[PracticeSession]` | `Vec<ItemScoreTrend>` |

## Existing Types Used (unchanged)

- `PracticeSession` — source data with `started_at: DateTime<Utc>`, `total_duration_secs: u64`, `entries: Vec<SetlistEntry>`
- `SetlistEntry` — per-item data with `item_id`, `item_title`, `duration_secs`, `score: Option<u8>`
- `ScoreHistoryEntry` — existing type in model.rs (reused conceptually but analytics uses its own `ScorePoint` for simpler serialisation)
