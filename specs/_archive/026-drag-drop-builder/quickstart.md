# Quickstart Verification: Drag-and-Drop Session Builder

**Feature**: 026-drag-drop-builder
**Date**: 2026-02-18

## Prerequisites

- Feature branch `026-drag-drop-builder` checked out and rebased onto latest `main` (which includes 025-reusable-routines for FR-010)
- `cargo fmt --check` passes
- `cargo clippy -- -D warnings` passes
- `cargo test` passes (all existing + new tests)
- Dev server running (`trunk serve`)

## Verification Steps

### V1: Drag Handle Visible (FR-002)

1. Navigate to Sessions → New Session
2. Add 3+ items to the setlist
3. **Verify**: Each setlist entry shows a grip icon (drag handle) at the leftmost position
4. **Verify**: The drag handle has a minimum touch-target area of 44x44px (inspect element)
5. **Verify**: Up/down arrow buttons are still visible alongside the drag handle (FR-012)

### V2: Drag-and-Drop Reorder on Desktop (FR-001, FR-003, FR-004)

1. Add items "A", "B", "C", "D" to the setlist (in that order)
2. Click and hold the drag handle on item "C"
3. Drag upward past item "A"
4. **Verify**: A coloured drop indicator line appears between the top of the list and item "A"
5. Release the mouse
6. **Verify**: The setlist now reads "C", "A", "B", "D"
7. **Verify**: Position numbers update to 1, 2, 3, 4

### V3: Drag Cancel (FR-009)

1. With items "C", "A", "B", "D" in the setlist
2. Click and hold the drag handle on item "B"
3. Drag the pointer outside the setlist area (e.g., into the library section) and release
4. **Verify**: The setlist order is unchanged: "C", "A", "B", "D"

### V4: Tap Library Row to Add (FR-005, FR-006)

1. In the Library Items section, click anywhere on a library item row (not specifically on the "+ Add" text)
2. **Verify**: The item is added to the setlist
3. **Verify**: The "+ Add" text is still visible on the row
4. **Verify**: On desktop, the row shows `cursor: pointer` on hover

### V5: Touch Drag Handle Only (FR-007) — Mobile/Responsive

1. Open the app in a mobile viewport (or use browser dev tools responsive mode)
2. Add 4+ items to the setlist
3. Touch and swipe on the entry body (outside the drag handle)
4. **Verify**: The page scrolls normally — no drag initiated
5. Touch and hold the drag handle, then drag to a new position
6. **Verify**: The entry reorders correctly

### V6: Reduced Motion (FR-011)

1. Enable `prefers-reduced-motion: reduce` in browser settings (or dev tools)
2. Perform a drag-and-drop reorder
3. **Verify**: Items snap immediately to new positions with no transition animations

### V7: Routine Edit Page (FR-010)

1. Navigate to Routines → select a routine → Edit
2. **Verify**: Routine entries have drag handles and up/down arrow buttons
3. Drag an entry to a new position
4. **Verify**: The entry reorders in the local list
5. Save changes
6. **Verify**: The new order is persisted

### V8: Single Item Setlist (Edge Case)

1. Add exactly one item to the setlist
2. **Verify**: The drag handle is visible
3. Attempt to drag it
4. **Verify**: No error, no crash — the item stays in place

### V9: Existing Arrow Buttons (FR-012)

1. Add 3+ items to the setlist
2. Use the up/down arrow buttons to reorder an entry
3. **Verify**: Arrow button reordering still works as before alongside the drag handles

### V10: All Tests Pass (SC-006)

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

All existing tests continue to pass. No core logic changes required.
