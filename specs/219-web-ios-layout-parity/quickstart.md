# Quickstart: Web — Adopt iOS Layout Patterns

**Feature**: `219-web-ios-layout-parity`
**Date**: 2026-03-21

## Prerequisites

- Rust 1.89.0+ with `wasm32-unknown-unknown` target
- Trunk 0.21.x
- Node.js (for E2E tests)
- Running API server (`cargo run -p intrada-api`)

## Verification Steps

### 1. Desktop Split-View Library

1. Run `just dev` to start the web app
2. Open the library at `http://localhost:8080/library` in a desktop browser (≥768px viewport)
3. **Verify**: Sidebar list on the left shows compact rows (title, subtitle, badge)
4. **Verify**: First item is auto-selected and its detail is shown in the right pane
5. Click a different item in the sidebar
6. **Verify**: Detail pane updates without page reload; sidebar highlights the selected row
7. **Verify**: Browser URL updates to `/library/{item-id}`
8. Click "Edit" in the detail pane
9. **Verify**: Edit form loads in the detail pane (no full-page navigation)

### 2. Mobile Library (Regression)

1. Resize browser to <768px (or use DevTools mobile viewport)
2. Navigate to `/library`
3. **Verify**: Full-page list view with compact rows (no sidebar)
4. Tap an item
5. **Verify**: Navigates to full-page detail view
6. Tap back
7. **Verify**: Returns to list view

### 3. Desktop Session Builder

1. Navigate to `/sessions/new` on desktop
2. **Verify**: Split-view with library list on left, setlist panel on right
3. Click 3 library items
4. **Verify**: Each clicked item shows accent left bar + check icon; setlist panel shows 3 entries
5. Click a selected item again
6. **Verify**: Item is removed from setlist; row returns to unselected state
7. In the setlist panel, drag an entry to reorder
8. **Verify**: Entry moves to new position
9. Click an entry to expand
10. **Verify**: Duration, intention, and rep fields are revealed
11. Click "Start Session"
12. **Verify**: Session starts (transitions to active session view)

### 4. Mobile Session Builder

1. Navigate to `/sessions/new` on mobile viewport
2. **Verify**: Full-screen library list
3. Tap 2 items
4. **Verify**: Sticky bottom bar shows "2 items · X min" and "Start Session" button
5. Tap the bottom bar summary area
6. **Verify**: Slide-up sheet opens with full setlist (reorder, entry details, intention)
7. Dismiss the sheet (tap backdrop or swipe down)
8. **Verify**: Returns to library list with bottom bar still showing

### 5. Search & Filter (Session Builder)

1. Open session builder
2. Type a search query in the search field
3. **Verify**: Library list filters to matching items
4. Tap a filter tab (Pieces / Exercises)
5. **Verify**: List shows only that type
6. Clear search
7. **Verify**: Full list restored

### 6. E2E Test Suite

```bash
npx playwright test
```

**Verify**: All existing tests pass. No regressions in library, session, navigation, or analytics flows.
