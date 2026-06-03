# Quickstart: Session Week Strip Navigator

**Feature**: 154-session-week-strip
**Branch**: `154-session-week-strip`

## Prerequisites

- Rust stable (1.89.0+)
- trunk 0.21.x
- Tailwind CSS v4 standalone CLI
- Running API server (or local dev with auth disabled)

## Setup

```bash
# Ensure you're on the feature branch
git checkout 154-session-week-strip

# Build and serve the web app
cd crates/intrada-web
trunk serve
```

The app will be available at `http://localhost:8080`.

## Verification Steps

### V1: Week Strip Renders (FR-001, FR-002, FR-003)

1. Navigate to `/sessions`
2. **Verify**: A horizontal week strip is visible below the page heading
3. **Verify**: The strip shows seven days labeled M, T, W, T, F, S, S with date numbers
4. **Verify**: The displayed week is the current week (check dates match this week's Mon–Sun)
5. **Verify**: Days that have recorded sessions show a small dot indicator below the date number

### V2: Day Selection & Auto-Select (FR-004, FR-005, FR-012, FR-013)

1. Navigate to `/sessions`
2. **Verify**: A day is auto-selected (highlighted) — either today if it has sessions, or the most recent day with sessions in the current week
3. **Verify**: Session cards for the selected day are displayed below the strip
4. Click a different day that has sessions (dot visible)
5. **Verify**: The selected day highlight moves and session cards update to show that day's sessions
6. Click a day without sessions (no dot)
7. **Verify**: An empty state message appears (e.g. "No sessions on this day")

### V3: Session Card Content (FR-006, FR-007)

1. Select a day with sessions
2. **Verify**: Each session card shows: start time, total duration, items practised with status icons, confidence scores, tempo (if applicable), rep targets (if applicable), intention (if set), and notes (if set)
3. **Verify**: Sessions within the day are ordered chronologically (earliest first)
4. Click/tap a session card
5. **Verify**: Navigation to the session detail/review page occurs

### V4: Week Navigation - Arrows (FR-008, FR-010)

1. Click the left arrow (◄) in the week strip header
2. **Verify**: The strip updates to show the previous week's dates
3. **Verify**: The month/year label updates correctly
4. **Verify**: Session dot indicators reflect the previous week's data
5. **Verify**: A day with sessions is auto-selected in the new week (or Monday if no sessions)
6. Click the right arrow (►)
7. **Verify**: The strip returns to the original week

### V5: Week Navigation - Swipe (FR-009)

*Test on a mobile device or using browser dev tools touch simulation:*

1. Swipe left on the week strip
2. **Verify**: Navigates to the next week
3. Swipe right on the week strip
4. **Verify**: Navigates to the previous week

### V6: Month Label (Clarification: dual month label)

1. Navigate to a week that spans two months (e.g. last week of February / first week of March)
2. **Verify**: The month label shows both months (e.g. "Feb – Mar 2026")
3. Navigate to a week within a single month
4. **Verify**: The month label shows just one month (e.g. "March 2026")

### V7: Loading States (FR-010)

1. Navigate to a different week using arrows
2. **Verify**: The week strip updates immediately with new dates
3. **Verify**: The session cards area shows a brief loading skeleton before sessions appear

### V8: Show All Sessions (FR-011)

1. On the `/sessions` page, scroll below the session cards
2. **Verify**: A "Show all sessions" link is visible
3. Click the link
4. **Verify**: The full chronological session list is displayed (same format as the original sessions page)
5. **Verify**: Navigation back to the week view is possible

### V9: Delete Session (FR-014)

1. Select a day with sessions
2. Click "Delete" on a session card
3. **Verify**: Confirmation prompt appears
4. Confirm the deletion
5. **Verify**: Session is removed and the day's session list updates
6. **Verify**: If the day now has no sessions, the dot indicator disappears from the week strip

### V10: New Session Button (FR-015)

1. Navigate to `/sessions`
2. **Verify**: The "New Session" button/link is still visible and accessible at the top of the page
3. Click it
4. **Verify**: Navigates to `/sessions/new`

### V11: Empty States (Edge Cases)

1. If you have no sessions at all:
   - **Verify**: Week strip shows current week with no dots
   - **Verify**: Empty state message encourages starting a practice session
2. If a day has many sessions:
   - **Verify**: Sessions stack vertically and the session area scrolls
   - **Verify**: The week strip stays in place (does not scroll with sessions)

### V12: Responsive Layout

1. Resize browser to mobile width (<640px)
   - **Verify**: Week strip spans full width
   - **Verify**: Day cells are evenly distributed
   - **Verify**: Session cards are full-width
2. Resize to desktop width (≥640px)
   - **Verify**: Week strip sits within content area with comfortable spacing
   - **Verify**: Arrow buttons are visible

## Quality Checks

```bash
# Run all workspace tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

All three must pass before merging.
