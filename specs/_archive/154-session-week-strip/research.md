# Research: Session Week Strip Navigator

**Feature**: 154-session-week-strip
**Date**: 2026-03-04

## R1: Date Handling & Week Calculation in WASM

### Decision
Use chrono's `NaiveDate`, `Datelike`, and `IsoWeek` for all week calculations, following the existing patterns in `analytics.rs` and `app.rs`.

### Rationale
- chrono 0.4 with `serde` feature is already a workspace dependency used across all three crates
- `NaiveDate`, `Datelike`, `IsoWeek`, and `Weekday` are all pure-computation types that work in WASM without additional feature flags
- The codebase already uses the exact pattern needed: `session.started_at.date_naive()` → `NaiveDate` → `.iso_week()` in both `analytics.rs` and `app.rs`
- ISO 8601 week convention (Monday start) matches the spec requirement (FR-001: Monday through Sunday)

### Key Patterns Already in Production
- **DateTime to date**: `session.started_at.date_naive()` → `NaiveDate`
- **ISO week**: `date.iso_week()` → `IsoWeek` with `.year()` and `.week()`
- **Week start from ISO week**: `NaiveDate::from_isoywd_opt(year, week, Weekday::Mon)`
- **Date arithmetic**: `date + chrono::Duration::days(n)`
- **Date range filtering**: `session_date >= week_start && session_date < week_end`
- **Date grouping**: `HashMap<NaiveDate, Vec<...>>` pattern from `compute_daily_totals`

### Alternatives Considered
- **js-sys Date**: Would work but adds an unnecessary JS interop boundary for pure date math. chrono is already available and Rust-native.
- **Manual week math**: Too error-prone for ISO 8601 week numbering edge cases (year boundaries, etc.). chrono handles this correctly.

## R2: Session Data for Week Filtering

### Decision
Filter sessions client-side from `ViewModel.sessions` in the Leptos shell. No changes to intrada-core or intrada-api.

### Rationale
- `ViewModel.sessions: Vec<PracticeSessionView>` already contains all completed sessions, sorted newest-first by `finished_at`
- Each `PracticeSessionView.started_at` is an RFC3339 string that can be parsed with `DateTime::parse_from_rfc3339` (same as existing helpers in `helpers.rs`)
- The analytics module already performs per-week session filtering on the full dataset — the pattern is proven at scale
- Filtering a few hundred sessions by date is well within the 16ms frame budget
- Adding API-side week filtering would add unnecessary complexity for a dataset that's already in memory

### Data Flow
```
ViewModel.sessions (Vec<PracticeSessionView>, sorted newest-first)
    ↓ parse started_at from RFC3339
    ↓ convert to NaiveDate via date_naive()
    ↓ group by NaiveDate into HashMap<NaiveDate, Vec<PracticeSessionView>>
    ↓ filter by current week range (week_start..week_start+7)
    ↓
WeekStrip: knows which days have sessions (dot indicators)
SessionCards: shows sessions for selected day
```

### Alternatives Considered
- **API endpoint for week-filtered sessions**: Over-engineered for the dataset size. Would add latency for every week navigation.
- **Core-side filtering via new Event/Effect**: Would pollute the core with UI navigation concerns. Week/day selection is ephemeral UI state per the architecture's state boundary rules.

## R3: Touch/Swipe Gesture Support

### Decision
Use `PointerEvent` (already enabled in web-sys) for swipe detection rather than raw `TouchEvent`. Fall back to arrow buttons as the primary interaction.

### Rationale
- `PointerEvent` is already available — web-sys has `"PointerEvent"` feature enabled and the codebase uses it for drag-and-drop in `use_drag_reorder`
- Pointer events unify mouse and touch — a single handler covers both desktop drag and mobile swipe
- The swipe gesture is simple: track `pointerdown` → `pointermove` → `pointerup`, measure horizontal delta, trigger week navigation if delta exceeds threshold
- The existing `DragHandle` component already demonstrates the pattern: pointer capture, movement threshold (5px), pointer events on window level

### Implementation Approach
- Track `pointerdown` position on the week strip element
- On `pointerup`, calculate horizontal delta
- If |deltaX| > 50px and |deltaX| > |deltaY| (horizontal bias), trigger prev/next week
- No additional web-sys features needed — `PointerEvent` already gives us `client_x()` and `client_y()`

### Alternatives Considered
- **TouchEvent API**: Would require adding `"TouchEvent"`, `"Touch"`, `"TouchList"` web-sys features. More specific to touch screens; doesn't cover mouse/trackpad.
- **Third-party gesture library**: Against spec assumptions ("basic touch event handling rather than a third-party gesture library").
- **CSS scroll-snap**: Doesn't provide the discrete week-by-week navigation needed; continuous scrolling is wrong for this UX.

## R4: Week Strip State Management

### Decision
All week navigation state (current week offset, selected day) lives in Leptos signals — pure UI state, not domain state.

### Rationale
- Per the architecture's state boundary rules: "UI state that has no meaning outside the current view stays in Leptos signals"
- The selected week and selected day are ephemeral — they reset on page navigation and have no domain meaning
- No need to persist week selection across sessions or share it with core
- This matches the existing pattern: form field values, tab selection, and drag state all use Leptos signals

### State Shape
```
week_offset: RwSignal<i32>        // 0 = current week, -1 = last week, +1 = next week
selected_date: RwSignal<Option<NaiveDate>>  // Currently selected day, or None for auto-select
```

### Alternatives Considered
- **Crux model state**: Would violate architecture principle — "Don't inflate the Crux model with ephemeral UI concerns"
- **URL query params**: Over-complicated for simple temporal navigation. Deep-linking to a specific week is not a spec requirement.

## R5: "Show All Sessions" Route

### Decision
Create a separate route `/sessions/all` that renders the existing flat session list.

### Rationale
- The spec (FR-011) requires a "Show all sessions" link below the week view
- A separate route is cleaner than toggling view modes in-place with signals
- The existing `SessionsListView` component can be moved to the new route with minimal changes
- The `/sessions` route gets the new week strip view
- Back navigation works naturally with browser history

### Alternatives Considered
- **In-place toggle**: Would require managing show/hide state and conditionally rendering two complex views in one component. More complex, less URL-addressable.
- **Anchor link to section**: Doesn't match the "link" interaction pattern in the spec.
