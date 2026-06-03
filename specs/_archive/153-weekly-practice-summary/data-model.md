# Data Model: Weekly Practice Summary

**Feature**: 153-weekly-practice-summary
**Date**: 2026-02-24

## Overview

No new persistence is required. All data is computed from existing `PracticeSession` and
`Item` data already loaded in the Crux `Model`. The new types below are **view model
extensions** — serialisable structs added to `AnalyticsView` and consumed by the web shell.

## Entity Definitions

### WeeklySummary (extended)

Replaces the existing `WeeklySummary` struct in `analytics.rs`.

| Field | Type | Description |
|-------|------|-------------|
| `total_minutes` | `u32` | Total practice time this week (minutes) |
| `session_count` | `usize` | Number of sessions this week |
| `items_covered` | `usize` | Distinct items practised this week |
| `prev_total_minutes` | `u32` | Total practice time last week |
| `prev_session_count` | `usize` | Number of sessions last week |
| `prev_items_covered` | `usize` | Distinct items practised last week |
| `time_direction` | `Direction` | Up, Down, or Same for total minutes |
| `sessions_direction` | `Direction` | Up, Down, or Same for session count |
| `items_direction` | `Direction` | Up, Down, or Same for items covered |
| `has_prev_week_data` | `bool` | Whether previous week had any sessions |

**Derivation**: Computed from `&[PracticeSession]` by bucketing sessions into current
and previous ISO weeks (using `NaiveDate::iso_week()`).

### Direction

Enum for directional comparison indicators.

| Variant | Description |
|---------|-------------|
| `Up` | This week's value > last week's |
| `Down` | This week's value < last week's |
| `Same` | Values are equal |

Serialised as `"up"`, `"down"`, `"same"` via `#[serde(rename_all = "lowercase")]`.

### NeglectedItem

A library item not practised within the 14-day lookback window.

| Field | Type | Description |
|-------|------|-------------|
| `item_id` | `String` | Library item ID |
| `item_title` | `String` | Display title |
| `days_since_practice` | `Option<u32>` | Days since last practised; `None` = never practised |

**Derivation**: Computed from `&[Item]` (current library) and `&[PracticeSession]`
(all sessions). Scans sessions within the past 14 days to find practised item IDs.
Any current library item not in that set is a neglected item.

**Ordering**: Never-practised items first (`days_since_practice: None`), then by
`days_since_practice` descending (longest gap first). Capped at 5 items.

### ScoreChange

An item whose score changed during the current week.

| Field | Type | Description |
|-------|------|-------------|
| `item_id` | `String` | Library item ID |
| `item_title` | `String` | Display title |
| `previous_score` | `Option<u8>` | Latest score before this week; `None` = newly scored |
| `current_score` | `u8` | Latest score this week |
| `delta` | `i8` | Signed change (current - previous); 0 for newly scored |
| `is_new` | `bool` | True if scored for the first time this week |

**Derivation**: Computed from `&[PracticeSession]` by collecting all scored entries,
partitioning into this-week and pre-this-week, finding items where the latest score
differs. Items scored for the first time this week have `is_new: true` and `delta: 0`.

**Ordering**: Largest absolute delta first. Capped at 5 items.

## AnalyticsView Extension

The existing `AnalyticsView` struct gains two new fields:

```
AnalyticsView {
    weekly_summary: WeeklySummary,   // ← extended with comparison fields
    streak: PracticeStreak,          // unchanged
    daily_totals: Vec<DailyPracticeTotal>,  // unchanged
    top_items: Vec<ItemRanking>,     // unchanged
    score_trends: Vec<ItemScoreTrend>,  // unchanged
    neglected_items: Vec<NeglectedItem>,  // NEW
    score_changes: Vec<ScoreChange>,      // NEW
}
```

## Relationships

```
PracticeSession 1──* SetlistEntry
                         │
                         ├── item_id ──→ Item.id
                         └── score: Option<u8>

Item (library)
  │
  ├── referenced by SetlistEntry.item_id (sessions)
  ├── referenced by NeglectedItem.item_id (computed)
  └── referenced by ScoreChange.item_id (computed)
```

## Validation Rules

No user input validation needed — all data is computed from existing persisted data.
The computation functions enforce:

- ISO week boundaries (Monday–Sunday) for week bucketing
- 14-day lookback window for neglected items
- Cap of 5 for neglected items and score changes
- Never-practised items sort to top of neglected list
- Neutral language for score changes (no "declined", "dropped", etc.)

## State Transitions

None — these are stateless computations. Every call to `compute_analytics()` produces
a fresh result from the current `Model` state. No caching, no incremental updates.
