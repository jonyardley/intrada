# Data Model: Tempo Progress Charts

**Feature Branch**: `151-tempo-progress-charts`
**Date**: 2026-02-24

## Overview

No new entities or database changes. This feature is purely a visualisation layer built on data structures introduced by tempo tracking (#52).

## Existing Entities Used

### TempoHistoryEntry

Already defined in `intrada-core/src/model.rs`.

| Field | Type | Description |
|-------|------|-------------|
| session_date | String (RFC3339) | When the session occurred |
| tempo | u16 | Achieved BPM (1–500) |
| session_id | String | Reference to the parent session |

**Sort order**: Descending by `session_date` (most recent first). The chart will reverse this to chronological (ascending) for left-to-right plotting.

### ItemPracticeSummary

Already defined in `intrada-core/src/model.rs`.

| Field | Type | Used by chart |
|-------|------|---------------|
| tempo_history | Vec\<TempoHistoryEntry\> | All data points for the chart |
| latest_tempo | Option\<u16\> | Progress percentage numerator |

### LibraryItemView (target BPM source)

The `tempo` field (`Option<String>`, e.g., "120 BPM") on `LibraryItemView` provides the target reference line value. Already available on the detail page.

## Data Flow

```
Crux Model (sessions + items)
  → build_practice_summaries()      [O(M×E) single pass, already runs]
  → ItemPracticeSummary             [tempo_history + latest_tempo]
  → detail.rs view()                [already receives practice summary]
  → TempoProgressChart component    [new — receives data as props]
```

No new effects, no new API calls, no new events. The chart component is a pure function of data already available in the view.

## Validation

No new validation. The tempo range (1–500 BPM) is enforced at input time by `validate_achieved_tempo()` from #52.

## State Changes

None. This feature adds no state transitions, no new events, and no model mutations. It is read-only visualisation of existing data.
