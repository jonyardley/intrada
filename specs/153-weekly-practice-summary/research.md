# Research: Weekly Practice Summary

**Feature**: 153-weekly-practice-summary
**Date**: 2026-02-24

## Decision 1: Extend existing analytics vs new module

**Decision**: Extend the existing `analytics.rs` module with new pure computation functions.

**Rationale**: The existing `compute_analytics()` in `intrada-core/src/analytics.rs` already follows the exact pattern needed — pure functions accepting `&[PracticeSession]` and `today: NaiveDate`, returning serialisable view structs. Adding new functions here maintains consistency and keeps all analytics computation co-located. The function is called from `view()` in `app.rs` at lines 216–221, which already has access to both `model.sessions` and `model.items`.

**Alternatives considered**:
- Separate `weekly_comparison.rs` module: Rejected — would fragment analytics logic across two files for no architectural benefit.
- Compute in the web shell: Rejected — violates Architecture Integrity principle (pure core).

## Decision 2: Signature change to compute_analytics

**Decision**: Extend `compute_analytics` to accept `&[Item]` in addition to `&[PracticeSession]` and `today`.

**Rationale**: The neglected-items computation (FR-004) requires the current library items list to determine which items haven't been practised recently. The existing function signature `compute_analytics(sessions, today)` doesn't have access to items. The `view()` method in `app.rs` already has `model.items` available at the call site, so passing it through is a minimal change.

**Alternatives considered**:
- Compute neglected items separately in `view()`: Rejected — splits analytics logic across two locations.
- Add items to AnalyticsView as a post-processing step: Rejected — more complex, same result.

## Decision 3: Week comparison data structure

**Decision**: Replace the existing `WeeklySummary` struct with an expanded version that includes both current-week and previous-week values plus a directional indicator.

**Rationale**: The current `WeeklySummary` only stores `total_minutes` and `session_count` for the current week. FR-001/FR-002/FR-003 require comparison values. Rather than creating a separate comparison struct alongside the existing one, extending the single struct keeps the data model clean. The existing stat cards that consumed `WeeklySummary` are being replaced (FR-006), so there are no backward-compatibility concerns.

**Alternatives considered**:
- Keep old `WeeklySummary` and add a new `WeeklyComparison` struct alongside: Rejected — the old struct becomes orphaned once the stat cards are removed.
- Store raw data and compute comparison in the shell: Rejected — violates pure core principle.

## Decision 4: Neglected items computation approach

**Decision**: Compute neglected items by scanning all sessions within a 14-day lookback window, collecting practised item IDs, then comparing against the full items list. Items never practised sort to the top; remaining items sort by days-since-last-practice descending.

**Rationale**: This is a straightforward set-difference operation. The existing `compute_top_items()` already iterates over all session entries by item_id, providing a known-working pattern. A 14-day lookback is specified in FR-004 and confirmed during clarification.

**Alternatives considered**:
- Use `ItemPracticeSummary` from the model: Could work for session counts but doesn't track the *most recent* practice date directly. Computing from raw sessions is simpler and more reliable.
- Pre-compute in the API: Rejected — no new API endpoints, all from existing data (spec assumption).

## Decision 5: Score changes computation approach

**Decision**: Compute score changes by finding items whose latest score this week differs from their latest score in all prior sessions. Cap at 5 items, sorted by largest absolute delta.

**Rationale**: The existing `compute_score_trends()` already collects all scored entries by item. The new function follows the same data access pattern but compares this-week vs pre-this-week latest scores rather than building a full history. Items scored for the first time this week are shown with a "new" indicator (spec assumption).

**Alternatives considered**:
- Reuse `ItemScoreTrend` data: The existing trend is for display visualisation (score dots), not comparison. A dedicated computation is cleaner.

## Decision 6: UI structure — new components

**Decision**: Create three new components: `WeekComparisonRow`, `NeglectedItemsList`, and `ScoreChangesList`. These compose inside an existing `Card` component on the analytics page.

**Rationale**: Each section has distinct rendering logic and can be hidden independently (FR-007). Separate components follow the component library pattern established in the codebase. The existing `StatCard` is not reusable here because the comparison metric has a fundamentally different layout (value + directional indicator + comparison text).

**Alternatives considered**:
- Extend `StatCard` with comparison props: Rejected — the layout is too different (two-line value with arrow vs single value with subtitle). Would add complexity to StatCard without benefit.
- Inline everything in `AnalyticsDashboard`: Rejected — violates component library principle (Constitution III).

## Decision 7: Stat card replacement strategy

**Decision**: Remove the "This Week" and "Sessions" stat cards from the 3-column grid. Keep only "Streak" as a single stat card. Place the new weekly summary card between the streak card and the practice history chart.

**Rationale**: Confirmed in clarification session — the new weekly summary makes the existing time and sessions stat cards redundant since it shows the same data with richer context (comparison). The streak card remains because streak logic is separate from weekly comparison.

**Alternatives considered**:
- Keep all three stat cards and add the comparison below: Rejected by user during clarification (option B chosen — replace).

## Existing Patterns Documented

### Analytics Data Flow
```
model.sessions + model.items
    → compute_analytics(sessions, items, today)  [pure, in analytics.rs]
        → AnalyticsView { weekly_summary, streak, daily_totals, ... }
            → ViewModel.analytics = Some(analytics)  [in app.rs view()]
                → <AnalyticsDashboard analytics=analytics />  [in analytics.rs view]
```

### Key Types Available
- `PracticeSession.started_at: DateTime<Utc>` — used for week bucketing via `.date_naive().iso_week()`
- `PracticeSession.entries: Vec<SetlistEntry>` — each has `item_id`, `item_title`, `score: Option<u8>`
- `PracticeSession.total_duration_secs: u64` — for time aggregation
- `Item.id: String`, `Item.title: String` — for neglected items matching
- `NaiveDate.iso_week()` — already used for current week computation

### Test Helpers Available
- `make_session(id, date, total_secs, entries)` — creates a PracticeSession
- `make_entry(item_id, title, item_type, duration_secs, score)` — creates a SetlistEntry
- Both in `analytics.rs` test module, ready for reuse

### Components Available
- `Card` — container
- `StatCard` — for streak (retained)
- `PageHeading` — already on analytics page
- Design tokens: `text-primary`, `text-secondary`, `text-muted`, `text-faint`, `text-accent-text`, `text-success-text`
