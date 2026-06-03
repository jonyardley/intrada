# Data Model: Session Week Strip Navigator

**Feature**: 154-session-week-strip
**Date**: 2026-03-04

## Overview

This feature introduces **no new persistent entities**. All data comes from the existing `PracticeSessionView` in the ViewModel. The "data model" for this feature consists of derived/computed structures used purely in the Leptos shell for rendering the week strip UI.

## Existing Entities (unchanged)

### PracticeSessionView (from intrada-core ViewModel)
| Field | Type | Notes |
|-------|------|-------|
| id | String | ULID |
| started_at | String | RFC3339 datetime (e.g. "2026-03-04T14:30:00+00:00") |
| finished_at | String | RFC3339 datetime |
| total_duration_display | String | e.g. "25 min" |
| completion_status | String | "completed" or "ended_early" |
| notes | Option\<String\> | Session notes |
| entries | Vec\<SetlistEntryView\> | Practice items with scores, tempo, reps |
| session_intention | Option\<String\> | Session intention |

### SetlistEntryView (from intrada-core ViewModel)
| Field | Type | Notes |
|-------|------|-------|
| id | String | ULID |
| item_id | String | Library item reference |
| item_title | String | Display name |
| item_type | String | "piece" or "exercise" |
| position | usize | Order in session |
| duration_display | String | e.g. "10 min" |
| status | String | "completed", "skipped", "not_attempted" |
| score | Option\<u8\> | 1-5 confidence score |
| notes | Option\<String\> | Entry-level notes |
| intention | Option\<String\> | Entry-level intention |
| rep_target | Option\<u8\> | Repetition target |
| rep_count | Option\<u8\> | Repetitions completed |
| rep_target_reached | Option\<bool\> | Whether target was met |
| achieved_tempo | Option\<u16\> | BPM achieved |

## Derived Structures (new, shell-only)

These structures exist only in the Leptos shell layer (`intrada-web`) as computed values for rendering. They are **not persisted** and **not part of the Crux model**.

### WeekData
Computed from `ViewModel.sessions` for a given week offset.

| Field | Type | Description |
|-------|------|-------------|
| week_start | NaiveDate | Monday of the displayed week |
| days | [DayData; 7] | Mon–Sun data for each day |
| month_label | String | e.g. "March 2026" or "Feb – Mar 2026" |

**Computation**: Given `week_offset: i32` relative to current week:
1. Get current ISO week via `today.iso_week()`
2. Calculate target Monday: `current_monday + Duration::days(week_offset * 7)`
3. Build 7 DayData entries for Mon–Sun
4. Determine month label from first and last day

### DayData
One day cell in the week strip.

| Field | Type | Description |
|-------|------|-------------|
| date | NaiveDate | The calendar date |
| day_abbrev | &'static str | Single letter: "M", "T", "W", "T", "F", "S", "S" |
| date_number | u32 | Day of month (1–31) |
| has_sessions | bool | Whether any sessions exist on this date |
| is_today | bool | Whether this is today's date |
| session_count | usize | Number of sessions on this date |

### SessionsByDate
Lookup structure for quick access to sessions grouped by date.

| Field | Type | Description |
|-------|------|-------------|
| map | HashMap\<NaiveDate, Vec\<PracticeSessionView\>\> | Sessions keyed by started_at date |

**Computation**: Parse each session's `started_at` RFC3339 string → extract `NaiveDate` → group into HashMap. Sessions within each day are sorted chronologically (earliest first, per spec FR-005).

## State Transitions

This feature has no domain state transitions. All state is ephemeral UI state:

| State | Type | Initial Value | Transitions |
|-------|------|---------------|-------------|
| week_offset | i32 | 0 (current week) | ← arrow: -1, → arrow: +1 |
| selected_date | Option\<NaiveDate\> | None (triggers auto-select) | Day tap: Some(date), Week nav: None (re-triggers auto-select) |

**Auto-select logic** (when `selected_date` is None):
1. If today is in the displayed week and has sessions → select today
2. Else if any day in the week has sessions → select the most recent such day
3. Else → select today (or Monday if today not in week)

## Validation Rules

No new validation rules. The feature reads existing validated data from the ViewModel.

## Database Changes

**None.** No new tables, columns, or migrations required.
