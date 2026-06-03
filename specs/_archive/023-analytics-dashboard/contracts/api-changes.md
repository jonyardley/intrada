# API Changes: Practice Analytics Dashboard

**Feature**: 023-analytics-dashboard
**Date**: 2026-02-17

## No API Changes Required

This feature is entirely read-only and computes all analytics client-side from the existing `GET /api/sessions` response.

### Existing Endpoint Used

**`GET /api/sessions`** — Returns `Vec<PracticeSession>` with full entry details including:
- `started_at`, `completed_at`, `total_duration_secs` (session-level)
- `entries[].item_id`, `entries[].item_title`, `entries[].duration_secs`, `entries[].score` (entry-level)

This provides all data needed for:
- Weekly summary (filter by `started_at` within current week)
- Practice streak (extract unique dates from `started_at`)
- Daily totals (aggregate `total_duration_secs` by date)
- Item rankings (aggregate entry `duration_secs` by `item_id`)
- Score trends (collect `score` values by `item_id` across sessions)

### Future Consideration

If the session count grows large enough to cause performance issues with client-side computation, a dedicated `GET /api/analytics` endpoint with server-side aggregation could be added. This is not needed for the initial implementation (spec assumes <100 sessions is the typical case).
